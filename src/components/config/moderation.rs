use std::sync::atomic::Ordering;

use serde_json::json;
use serenity::all::{
    ActionRowComponent, ButtonStyle, ComponentInteraction, ComponentInteractionDataKind,
    CreateActionRow, CreateButton, CreateEmbed, CreateInputText, CreateInteractionResponse,
    CreateModal, CreateSelectMenu, CreateSelectMenuKind, InputTextStyle, Permissions,
};

use crate::{
    components::config::{ConfigStage, EMBED_COLOR, advance_to},
    models::{
        bot::Bot,
        context::{Context, ContextComponentReplies, ContextReply},
        duration::Duration,
        interactions::InteractionBuilder,
        response::{Response, ResponseError, ResponseResult},
        role::Role,
        user::User,
    },
};

const MODERATION_TITLE: &str = "Configuration - Moderation";

pub struct MuteRole;
#[async_trait::async_trait]
impl ConfigStage for MuteRole {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "mute_role")
    }

    async fn router(&self, ctx: &Context<'_>, component: &ComponentInteraction) -> ResponseResult {
        let Context::Populated(context) = ctx else {
            return Err(ResponseError::Execution(
                "Failed to obtain context information".to_string(),
                Some("Please let the developers know of this error!".to_string()),
            ));
        };

        if sqlx::query!(
            "SELECT guild_id FROM moderation_configuration WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .fetch_optional(Bot::global().postgres())
        .await?
        .is_none()
        {
            sqlx::query!(
                "INSERT INTO moderation_configuration (guild_id) VALUES ($1)",
                context.guild.as_i64()
            )
            .execute(Bot::global().postgres())
            .await?;
        }

        let mute_role = sqlx::query!(
            "SELECT mute_role FROM moderation_configuration WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .mute_role;

        let user = User::from(component.user.id);
        let interactions = vec![
            InteractionBuilder::new(
                "config".to_string(),
                "selected_mute_role".to_string(),
                user,
                json!({"category": "moderation", "step": "selected_mute_role"}),
                Some(InteractionBuilder::one_hour_expiry()),
            )
            .build(),
            InteractionBuilder::new(
                "config".to_string(),
                "skip".to_string(),
                user,
                json!({"category": "moderation", "step": "default_strike_duration"}),
                Some(InteractionBuilder::one_hour_expiry()),
            )
            .build(),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.clone())
            .await?;

        ctx.acknowledge(component).await?;
        ctx.reply(component, Response::new().embed(
            CreateEmbed::new()
                .title(MODERATION_TITLE)
                // TODO: Improve and add descriptions
                .description(format!("You can add a role that Reaper will use to mute users.\nThe current mute role is: {}", match mute_role {
                    Some(role) => format!("<@&{role}>"),
                    None => "None".to_string(),
                }))
                .color(EMBED_COLOR),
        )
        .components(vec![
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    interactions[0].id,
                    CreateSelectMenuKind::Role { default_roles: None }
                )),
            CreateActionRow::Buttons(vec![
                CreateButton::new(interactions[1].id)
                    .label("Skip")
                    .style(ButtonStyle::Secondary),
            ])
        ])).await.map(|_| ())
    }
}

pub struct SelectedMuteRole;
#[async_trait::async_trait]
impl ConfigStage for SelectedMuteRole {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "selected_mute_role")
    }

    async fn router(&self, ctx: &Context<'_>, component: &ComponentInteraction) -> ResponseResult {
        let Context::Populated(context) = ctx else {
            return Err(ResponseError::Execution(
                "Failed to obtain context information".to_string(),
                Some("Please let the developers know of this error!".to_string()),
            ));
        };

        let ComponentInteractionDataKind::RoleSelect { values } = &component.data.kind else {
            return Err(ResponseError::Execution(
                "Invalid interaction type".to_string(),
                Some("Please let the developers know of this error!".to_string()),
            ));
        };

        let role = values.first().ok_or_else(|| {
            ResponseError::Execution(
                "No role selected".to_string(),
                Some("Please select a role.".to_string()),
            )
        })?;
        let role = Role::from(*role);

        let role_position = match context.partial_guild.roles.get(&role.as_serenity()) {
            Some(role) => {
                if role.permissions.contains(Permissions::ADMINISTRATOR) {
                    u16::MAX - 1
                } else {
                    role.position
                }
            }
            None => {
                return Err(ResponseError::Execution(
                    "Invalid role".to_string(),
                    Some(
                        "Please select a role that does not have administrator permissions"
                            .to_string(),
                    ),
                ));
            }
        };

        if role_position >= context.highest_role {
            return Err(ResponseError::Execution(
                "Invalid role".to_string(),
                Some("Please select a role that is lower than your highest role".to_string()),
            ));
        }

        // TODO: Configure role for user?

        ctx.acknowledge(component).await?;
        sqlx::query!(
            "UPDATE moderation_configuration SET mute_role = $1 WHERE guild_id = $2",
            role.as_i64(),
            context.guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await?;

        advance_to(DefaultStrikeDuration, ctx, component).await
    }
}

pub struct DefaultStrikeDuration;
#[async_trait::async_trait]
impl ConfigStage for DefaultStrikeDuration {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "default_strike_duration")
    }

    async fn router(&self, ctx: &Context<'_>, component: &ComponentInteraction) -> ResponseResult {
        let Context::Populated(context) = ctx else {
            return Err(ResponseError::Execution(
                "Failed to obtain context information".to_string(),
                Some("Please let the developers know of this error!".to_string()),
            ));
        };

        let default_strike_duration = sqlx::query!(
            "SELECT default_strike_duration FROM moderation_configuration WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .default_strike_duration;

        let help_text = r"> This refers to how long it takes for a strike to expire.
> Strikes™️ are Reaper's method of punishing users for breaking a rule, like the warns used in other bots!
> When a strike expires, they will not count towards strike escalations. Strike escalations allow you to automatically action against users for breaking a rule.
> You will be able to configure strike escalations later in the config.";

        let user = User::from(component.user.id);
        let interactions = vec![
            InteractionBuilder::new(
                "config".to_string(),
                "change".to_string(),
                user,
                json!({"category": "moderation", "step": "change_default_strike_duration"}),
                Some(InteractionBuilder::one_hour_expiry()),
            )
            .build(),
            InteractionBuilder::new(
                "config".to_string(),
                "skip".to_string(),
                user,
                json!({"category": "moderation", "step": "escalations"}),
                Some(InteractionBuilder::one_hour_expiry()),
            )
            .build(),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.clone())
            .await?;

        ctx.acknowledge(component).await?;
        ctx.reply(
            component,
            Response::new()
                .embed(
                    CreateEmbed::new()
                        .title(MODERATION_TITLE)
                        .description(format!(
                                "What should be the default strike duration?\n\n{help_text}\n\nYour current setting is: **{}**",
                                default_strike_duration.as_deref().unwrap_or("30d")
                            ))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![
                        CreateActionRow::Buttons(vec![
                            CreateButton::new(interactions[0].id)
                                .label("Change")
                                .style(ButtonStyle::Success),
                            CreateButton::new(interactions[1].id)
                                .label("Skip")
                                .style(ButtonStyle::Secondary)
                        ]),
                    ]),
            )
            .await
            .map(|_| ())
    }
}

pub struct ChangeDefaultStrikeDuration;
#[async_trait::async_trait]
impl ConfigStage for ChangeDefaultStrikeDuration {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "change_default_strike_duration")
    }

    async fn router(&self, ctx: &Context<'_>, component: &ComponentInteraction) -> ResponseResult {
        let Context::Populated(context) = ctx else {
            return Err(ResponseError::Execution(
                "Failed to obtain context information".to_string(),
                Some("Please let the developers know of this error!".to_string()),
            ));
        };

        component
            .create_response(
                &context.ctx.http,
                CreateInteractionResponse::Modal(
                    CreateModal::new("default_strike_duration_modal", "Default Strike Duration")
                        .components(vec![CreateActionRow::InputText(
                            CreateInputText::new(
                                InputTextStyle::Short,
                                "Duration",
                                "default_strike_duration",
                            )
                            .placeholder("30d")
                            .required(true),
                        )]),
                ),
            )
            .await?;

        let user = User::from(component.user.id);
        let message = component.get_response(&context.ctx.http).await?;

        let modal_collector = message
            .await_modal_interaction(&context.ctx)
            .author_id(user.as_serenity_id())
            .timeout(std::time::Duration::new(300, 0));

        if let Some(modal_interaction) = modal_collector.await {
            modal_interaction
                .create_response(
                    &context.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;
            context.has_responded.store(true, Ordering::Relaxed);

            if let ActionRowComponent::InputText(text) =
                &modal_interaction.data.components[0].components[0]
            {
                let value = text.value.clone().unwrap();
                if value.is_empty() {
                    return Err(ResponseError::Execution(
                        "Invalid duration".to_string(),
                        Some("Please enter a valid duration.".to_string()),
                    ));
                }
                let duration = Duration::new(value.as_str()).to_timestamp().unwrap();
                if duration < time::OffsetDateTime::now_utc() {
                    return Err(ResponseError::Execution(
                        "Invalid duration".to_string(),
                        Some("Please enter a valid duration.".to_string()),
                    ));
                }

                sqlx::query!(
                    "UPDATE moderation_configuration SET default_strike_duration = $1 WHERE guild_id = $2",
                    value,
                    context.guild.as_i64()
                ).execute(Bot::global().postgres()).await?;

                return advance_to(Escalations, ctx, component).await;
            }
        }

        Err(ResponseError::Execution(
            "Timeout!".to_string(),
            Some(
                "We didn't receive the duration from you within 5 minutes. Feel free to try again"
                    .to_string(),
            ),
        ))
    }
}

pub struct Escalations;
#[async_trait::async_trait]
impl ConfigStage for Escalations {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "escalations")
    }

    async fn router(&self, ctx: &Context<'_>, component: &ComponentInteraction) -> ResponseResult {
        ctx.reply(component, Response::new().content("Escalations"))
            .await
            .map(|_| ())
    }
}
