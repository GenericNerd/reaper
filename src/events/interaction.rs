use std::{collections::HashSet, sync::atomic::AtomicBool};

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use serenity::all::{
    CommandInteraction, ComponentInteraction, Context as SerenityContext, Interaction,
    ModalInteraction, PartialGuild, Permissions,
};
use tracing::{error, trace};
use uuid::Uuid;

use crate::{
    events::EventRouter,
    models::{
        bot::Bot,
        interactions::{
            context::{Context, PopulatedContext},
            responder::Responder,
        },
        permission::Permission,
        response::{ReaperError, ResponseError, ResponseResult},
        serenity::{guild::Guild, member::Member, role::Role, user::User},
    },
};

impl EventRouter {
    #[tracing::instrument(skip(self))]
    async fn check_feature_flags(&self, feature_flags: &[String]) -> ResponseResult<bool> {
        use crate::schema::global_kills::dsl::*;

        Ok(global_kills
            .filter(feature.eq_any(feature_flags))
            .filter(active.eq(false))
            .count()
            .get_result::<i64>(&mut Bot::instance().postgres())?
            == 0)
    }

    #[tracing::instrument(skip(self, guild), fields(guild.id = guild.as_i64()))]
    async fn check_guild_kill(&self, guild: &Guild) -> ResponseResult<bool> {
        use crate::schema::guild_kills::dsl::*;

        Ok(guild_kills
            .filter(guild_id.eq(guild.as_i64()))
            .count()
            .get_result::<i64>(&mut Bot::instance().postgres())?
            == 0)
    }

    #[tracing::instrument(skip(self, user), fields(user.id = user.as_i64()))]
    async fn check_user_kill(&self, user: &User) -> ResponseResult<bool> {
        use crate::schema::user_kills::dsl::*;

        Ok(user_kills
            .filter(user_id.eq(user.as_i64()))
            .count()
            .get_result::<i64>(&mut Bot::instance().postgres())?
            == 0)
    }

    #[tracing::instrument(skip(self, ctx, guild), fields(guild.id = guild.as_i64()))]
    async fn get_partial_guild(
        &self,
        ctx: &SerenityContext,
        guild: &Guild,
    ) -> ResponseResult<PartialGuild> {
        if let Some(guild) = guild.as_serenity_id().to_guild_cached(&ctx.cache) {
            return Ok(PartialGuild::from(guild.clone()));
        }
        trace!("Cache miss, fetching guild from API");

        ctx.http
            .get_guild(guild.as_serenity_id())
            .await
            .map_err(ResponseError::from)
    }

    #[tracing::instrument(skip(self, partial_guild, member), fields(guild.id = member.guild.as_i64(), user.id = member.user.as_i64()))]
    async fn compute_permissions(
        &self,
        partial_guild: PartialGuild,
        member: &Member,
    ) -> ResponseResult<(HashSet<Permission>, u16)> {
        let mut highest_role = 0;
        let user_permissions = if partial_guild.owner_id == member.user.as_serenity_id() {
            highest_role = u16::MAX;
            HashSet::from(Permission::ALL)
        } else {
            let mut user_permissions = HashSet::new();
            for permission in Permission::get_user(&member.guild, &member.user).await {
                user_permissions.insert(permission);
            }

            for role in &member.roles {
                if let Some(serenity_role) = partial_guild.roles.get(&role.as_serenity_id()) {
                    if serenity_role.position > highest_role {
                        highest_role = serenity_role.position;
                    }
                    if serenity_role
                        .permissions
                        .contains(Permissions::ADMINISTRATOR)
                    {
                        highest_role = u16::MAX - 1;
                        user_permissions.extend(Permission::ALL);
                        break;
                    }
                    if serenity_role
                        .permissions
                        .contains(Permissions::KICK_MEMBERS)
                    {
                        user_permissions.insert(Permission::ModerationKick);
                    }
                    if serenity_role.permissions.contains(Permissions::BAN_MEMBERS) {
                        user_permissions.insert(Permission::ModerationBan);
                    }
                    if serenity_role
                        .permissions
                        .contains(Permissions::MANAGE_MESSAGES)
                    {
                        user_permissions.insert(Permission::ModerationPurge);
                    }

                    user_permissions.extend(Permission::get_role(&member.guild, &role).await);
                }
            }

            user_permissions.extend(
                Permission::get_role(&member.guild, &Role::from(member.guild.as_u64())).await,
            );

            user_permissions
        };

        Ok((user_permissions, highest_role))
    }

    #[tracing::instrument(skip(self, ctx, user, member), fields(user.id = user.as_i64()))]
    async fn generate_context(
        &self,
        ctx: SerenityContext,
        user: User,
        member: Option<Member>,
    ) -> ResponseResult<Context> {
        if let Some(member) = member {
            let partial_guild = self.get_partial_guild(&ctx, &member.guild).await?;
            let (user_permissions, highest_role) =
                self.compute_permissions(partial_guild, &member).await?;
            Ok(Context::Populated(PopulatedContext {
                serenity_context: ctx,
                has_responded: AtomicBool::new(false),
                user,
                user_permissions: user_permissions.into_iter().collect(),
                highest_role,
                guild: Some(member.guild),
            }))
        } else {
            Ok(Context::Populated(PopulatedContext {
                serenity_context: ctx,
                has_responded: AtomicBool::new(false),
                user,
                user_permissions: vec![],
                highest_role: 0,
                guild: None,
            }))
        }
    }

    async fn run_checks(
        &self,
        feature_flags: &[String],
        guild: Option<&Guild>,
        user: &User,
    ) -> ResponseResult<()> {
        if !self.check_feature_flags(feature_flags).await? {
            return Err(ResponseError::Reaper(ReaperError::FeatureFlagDisabled));
        }

        if let Some(guild) = guild {
            if !self.check_guild_kill(guild).await? {
                return Err(ResponseError::Reaper(ReaperError::GuildKilled));
            }
        }

        if !self.check_user_kill(user).await? {
            return Err(ResponseError::Reaper(ReaperError::UserKilled));
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx, command))]
    async fn handle_command(
        &self,
        ctx: SerenityContext,
        command: &CommandInteraction,
    ) -> ResponseResult<()> {
        let handler = Bot::instance()
            .commands()
            .get(command.data.name.as_str())
            .ok_or(ResponseError::Reaper(ReaperError::CommandNotFound))?;

        let user = User::from(command.user.id);
        self.run_checks(
            &["commands".to_string(), format!("commands.{}", handler.id())],
            command.guild_id.map(|id| Guild::from(id)).as_ref(),
            &user,
        )
        .await?;

        let context = self
            .generate_context(
                ctx,
                user,
                command
                    .member
                    .as_ref()
                    .map(|member| Member::from(member.clone())),
            )
            .await?;

        let result = handler.execute(&context, command).await;
        if let Err(ref err) = result {
            let _ = command.error_message(&context, err).await;
        }
        result
    }

    fn extract_state(&self, custom_id: String) -> ResponseResult<(String, Uuid)> {
        let parts = custom_id.splitn(2, ':').collect::<Vec<_>>();

        if parts.len() != 2 {
            return Err(ResponseError::Reaper(ReaperError::InvalidCustomId));
        }

        Ok((
            parts[0].to_string(),
            Uuid::parse_str(parts[1])
                .map_err(|_| ResponseError::Reaper(ReaperError::InvalidCustomId))?,
        ))
    }

    #[tracing::instrument(skip(self, ctx, component))]
    async fn handle_component(
        &self,
        ctx: SerenityContext,
        component: &ComponentInteraction,
    ) -> ResponseResult<()> {
        let (handler_id, state_id) = self.extract_state(component.data.custom_id.clone())?;

        let handler = Bot::instance()
            .components()
            .get(&handler_id)
            .ok_or(ResponseError::Reaper(ReaperError::ComponentNotFound))?;

        let user = User::from(component.user.id);
        self.run_checks(
            &[
                "components".to_string(),
                format!("components.{}", handler.id()),
            ],
            component.guild_id.map(|id| Guild::from(id)).as_ref(),
            &user,
        )
        .await?;

        let context = self
            .generate_context(
                ctx,
                user,
                component
                    .member
                    .as_ref()
                    .map(|member| Member::from(member.clone())),
            )
            .await?;

        let state = Bot::instance()
            .state_store()
            .get(state_id)
            .await
            .ok_or(ResponseError::Reaper(ReaperError::StateNotFound))?;

        let result = handler.execute(&context, component, state.state).await;
        if let Err(ref err) = result {
            let _ = component.error_message(&context, err).await;
        }
        result
    }

    #[tracing::instrument(skip(self, ctx, modal))]
    async fn handle_modal(
        &self,
        ctx: SerenityContext,
        modal: &ModalInteraction,
    ) -> ResponseResult<()> {
        let (handler_id, state_id) = self.extract_state(modal.data.custom_id.clone())?;

        let handler = Bot::instance()
            .modals()
            .get(&handler_id)
            .ok_or(ResponseError::Reaper(ReaperError::ModalNotFound))?;

        let user = User::from(modal.user.id);
        self.run_checks(
            &["modals".to_string(), format!("modals.{}", handler.id())],
            modal.guild_id.map(|id| Guild::from(id)).as_ref(),
            &user,
        )
        .await?;

        let context = self
            .generate_context(
                ctx,
                user,
                modal
                    .member
                    .as_ref()
                    .map(|member| Member::from(member.clone())),
            )
            .await?;

        let state = Bot::instance()
            .state_store()
            .get(state_id)
            .await
            .ok_or(ResponseError::Reaper(ReaperError::StateNotFound))?;

        let result = handler.execute(&context, modal, state.state).await;
        if let Err(ref err) = result {
            let _ = modal.error_message(&context, err).await;
        }
        result
    }

    #[tracing::instrument(skip(self, ctx, interaction))]
    pub async fn on_interaction(&self, ctx: SerenityContext, interaction: Interaction) {
        let result = match interaction {
            Interaction::Command(cmd) => self.handle_command(ctx, &cmd).await,
            Interaction::Component(comp) => self.handle_component(ctx, &comp).await,
            Interaction::Modal(modal) => self.handle_modal(ctx, &modal).await,
            _ => return,
        };

        if let Err(err) = result {
            error!(error = %err, "Error handling interaction");
        }
    }
}
