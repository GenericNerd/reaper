use std::{fmt::Debug, time::Duration};

use dashmap::DashMap;
use serenity::all::{CommandInteraction, ComponentInteraction, CreateEmbed, CreateModal, Message};
use tracing::debug;

use crate::{
    components::Component,
    models::{
        context::{Context, ContextComponentReplies, ContextReply},
        interactions::{
            Interaction, InteractionBuilder, InteractionKind, config::ConfigInteraction,
        },
        permissions::Permission,
        response::{ExecutionError, InternalError, Response, ResponseError, ResponseResult},
        user::User,
    },
};

const EMBED_COLOR: u32 = 0x5539CC;

mod logging;
mod moderation;
mod xp;

pub enum ConfigEntry {
    Command(CommandInteraction),
    Component(ComponentInteraction),
}

impl ConfigEntry {
    async fn reply(&self, ctx: &Context<'_>, response: Response) -> Result<Message, ResponseError> {
        match self {
            ConfigEntry::Command(command) => ctx.reply(command, response).await,
            ConfigEntry::Component(component) => ctx.reply(component, response).await,
        }
    }

    async fn modal(&self, ctx: &Context<'_>, modal: CreateModal) -> ResponseResult {
        match self {
            ConfigEntry::Command(_) => Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            ))),
            ConfigEntry::Component(component) => ctx.modal(component, modal).await,
        }
    }

    fn component(&self) -> Result<ComponentInteraction, ResponseError> {
        match self {
            ConfigEntry::Command(_) => Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            ))),
            ConfigEntry::Component(component) => Ok(component.clone()),
        }
    }
}

#[async_trait::async_trait]
trait ConfigStage: Debug + Send + Sync {
    fn key(&self) -> (&'static str, &'static str);
    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult;
    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)>;
}

#[derive(Debug)]
struct Complete;
#[async_trait::async_trait]
impl ConfigStage for Complete {
    fn key(&self) -> (&'static str, &'static str) {
        ("complete", "complete")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        None
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        _data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let message = entry
            .reply(
                ctx,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title("Configuration Complete!")
                            .color(0x00ff00),
                    )
                    .components(vec![]),
            )
            .await?;

        tokio::time::sleep(Duration::new(5, 0)).await;

        message
            .delete(ctx.get_populated_context()?.ctx.http.clone())
            .await
            .map_err(|err| ResponseError::Serenity(Box::new(err)))
    }
}

#[derive(Debug)]
pub struct Config {
    handlers: DashMap<(String, String), Box<dyn ConfigStage>>,
}

impl Config {
    pub fn new() -> Self {
        let handlers: [Box<dyn ConfigStage>; 68] = [
            Box::new(moderation::ModerationEnter),
            Box::new(moderation::Footer),
            Box::new(moderation::ChangeFooter),
            Box::new(moderation::MuteRole),
            Box::new(moderation::SelectedMuteRole),
            Box::new(moderation::DefaultStrikeDuration),
            Box::new(moderation::ChangeDefaultStrikeDuration),
            Box::new(moderation::Escalations),
            Box::new(moderation::AddEscalation),
            Box::new(moderation::RemoveEscalation),
            Box::new(moderation::SubmitEscalations),
            Box::new(logging::LoggingEnter),
            Box::new(logging::Categories),
            Box::new(logging::SubmitCategories),
            Box::new(logging::OneOrMultiple),
            Box::new(logging::SingleLogChannel),
            Box::new(logging::SubmitSingleLogChannel),
            Box::new(logging::MultipleLogChannels),
            Box::new(logging::SubmitMultipleLogChannels),
            Box::new(xp::XPEnter),
            Box::new(xp::RandomOrSet),
            Box::new(xp::SelectedRandomOrSet),
            Box::new(xp::MessageCooldown),
            Box::new(xp::ChangeMessageCooldown),
            Box::new(xp::MaxLevel),
            Box::new(xp::ChangeMaxLevel),
            Box::new(xp::StackRewards),
            Box::new(xp::ChangeStackRewards),
            Box::new(xp::StackMultipliers),
            Box::new(xp::ChangeStackMultipliers),
            Box::new(xp::MultiplierCap),
            Box::new(xp::ChangeMultiplierCap),
            Box::new(xp::ResetXpOnLeave),
            Box::new(xp::ChangeResetXpOnLeave),
            Box::new(xp::LevelUpMessages),
            Box::new(xp::ChangeLevelUpMessages),
            Box::new(xp::DmOnLevelUp),
            Box::new(xp::ChangeDmOnLevelUp),
            Box::new(xp::LevelUpChannel),
            Box::new(xp::ChangeLevelUpChannel),
            Box::new(xp::LevelUpMessage),
            Box::new(xp::ChangeLevelUpMessage),
            Box::new(xp::RewardsEnter),
            Box::new(xp::Rewards),
            Box::new(xp::AddReward),
            Box::new(xp::RemoveReward),
            Box::new(xp::SaveRewards),
            Box::new(xp::RoleMultiplierEnter),
            Box::new(xp::RoleMultipliers),
            Box::new(xp::AddRoleMultiplier),
            Box::new(xp::RemoveRoleMultiplier),
            Box::new(xp::SaveRoleMultipliers),
            Box::new(xp::ChannelMultiplierEnter),
            Box::new(xp::ChannelMultipliers),
            Box::new(xp::AddChannelMultiplier),
            Box::new(xp::RemoveChannelMultiplier),
            Box::new(xp::SaveChannelMultipliers),
            Box::new(xp::RoleBlacklistEnter),
            Box::new(xp::RoleBlacklists),
            Box::new(xp::AddRoleBlacklist),
            Box::new(xp::RemoveRoleBlacklist),
            Box::new(xp::SaveRoleBlacklist),
            Box::new(xp::ChannelBlacklistEnter),
            Box::new(xp::ChannelBlacklists),
            Box::new(xp::AddChannelBlacklist),
            Box::new(xp::RemoveChannelBlacklist),
            Box::new(xp::SaveChannelBlacklist),
            Box::new(Complete),
        ];

        Self {
            handlers: handlers
                .into_iter()
                .map(|h| ((h.key().0.to_string(), h.key().1.to_string()), h))
                .collect::<DashMap<_, _>>(),
        }
    }

    pub async fn internal_router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        interaction: &Interaction,
    ) -> ResponseResult {
        // TODO: Remove once more interactions are added
        #[allow(irrefutable_let_patterns)]
        let InteractionKind::Config {
            category,
            single_category,
        } = &interaction.kind
        else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let stage = match &category {
            ConfigInteraction::Moderation { stage } => stage.to_string(),
            ConfigInteraction::Logging { stage } => stage.to_string(),
            ConfigInteraction::XP { stage } => stage.to_string(),
            ConfigInteraction::Complete => "complete".to_string(),
        };
        debug!(
            "Running config category {} stage {}",
            category.to_string(),
            stage
        );

        let mut current_key = (category.to_string(), stage);

        loop {
            let handler = self
                .handlers
                .get(&current_key)
                .ok_or(ResponseError::Execution(ExecutionError::Internal(
                    InternalError::InvalidConfigurationStep,
                )))?;

            match handler
                .router(ctx, entry, (category, *single_category))
                .await
            {
                Ok(res) => return Ok(res),
                Err(err) => {
                    if let ResponseError::Execution(_) = err {
                        match entry {
                            ConfigEntry::Command(command) => {
                                ctx.error_message_ref(command, &err).await?;
                            }
                            ConfigEntry::Component(component) => {
                                ctx.error_message_ref(component, &err).await?;
                            }
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        if let Some((next_cat, next_stage)) = handler.on_error_go_to_stage() {
                            current_key = (next_cat.to_string(), next_stage.to_string());
                            continue;
                        }
                        return Err(err);
                    }
                    return Err(err);
                }
            }
        }
    }
}

async fn advance_to<S: ConfigStage>(
    stage: S,
    ctx: &Context<'_>,
    entry: &ConfigEntry,
    data: (&ConfigInteraction, bool),
) -> ResponseResult {
    stage.router(ctx, entry, data).await
}

fn interaction_builder(
    user: User,
    interaction: ConfigInteraction,
    single_category: bool,
) -> InteractionBuilder {
    InteractionBuilder::new(
        InteractionKind::Config {
            category: interaction,
            single_category,
        },
        user,
        Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
    )
}

#[async_trait::async_trait]
impl Component for Config {
    fn name(&self) -> &'static str {
        "config"
    }

    fn required_permission(&self) -> Option<Permission> {
        Some(Permission::ConfigEdit)
    }

    #[tracing::instrument(skip(ctx, component), fields(interaction_id = interaction.id.to_string()))]
    async fn router(
        &self,
        ctx: &Context<'_>,
        component: &ComponentInteraction,
        interaction: &Interaction,
    ) -> ResponseResult {
        self.internal_router(ctx, &ConfigEntry::Component(component.clone()), interaction)
            .await
    }
}
