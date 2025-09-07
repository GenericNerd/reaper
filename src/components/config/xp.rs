use std::time::Duration as StdDuration;

use humantime::format_duration;
use serenity::all::{
    ActionRowComponent, ButtonStyle, CreateActionRow, CreateButton, CreateEmbed, CreateInputText,
    CreateModal, InputTextStyle,
};

use crate::{
    components::config::{ConfigEntry, ConfigStage, EMBED_COLOR, advance_to, interaction_builder},
    models::{
        bot::Bot,
        context::Context,
        duration::Duration,
        interactions::{
            InteractionBuilder,
            config::{ConfigInteraction, XPStage},
        },
        response::{
            ExecutionError, InputError, InternalError, Response, ResponseError, ResponseResult,
        },
        user::User,
    },
};

const XP_TITLE: &str = "Configuration - Levelling";

fn xp_interaction_builder(user: User, stage: XPStage, single_category: bool) -> InteractionBuilder {
    interaction_builder(user, ConfigInteraction::XP { stage }, single_category)
}

#[derive(Debug)]
pub struct XPEnter;
#[async_trait::async_trait]
impl ConfigStage for XPEnter {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "enter")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        None
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let interactions = [
            xp_interaction_builder(context.user, XPStage::RandomOrSet, data.1).build(),
            interaction_builder(context.user, ConfigInteraction::Complete, data.1).build(),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.to_vec())
            .await?;

        entry
            .reply(
                ctx,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description("Would you like to configure levelling?")
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(interactions[0].id.to_string())
                            .style(ButtonStyle::Success)
                            .label("Yes"),
                        CreateButton::new(interactions[1].id.to_string())
                            .style(ButtonStyle::Secondary)
                            .label("No"),
                    ])]),
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct RandomOrSet;
#[async_trait::async_trait]
impl ConfigStage for RandomOrSet {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "random_or_set")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        None
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;

        if sqlx::query!(
            "SELECT guild_id FROM xp_configuration WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .fetch_optional(Bot::global().postgres())
        .await?
        .is_none()
        {
            sqlx::query!(
                "INSERT INTO xp_configuration (guild_id) VALUES ($1)",
                context.guild.as_i64()
            )
            .execute(Bot::global().postgres())
            .await?;
        }

        let xp_configuration = sqlx::query!(
            "SELECT min_xp_per_message, max_xp_per_message, set_xp_per_message FROM xp_configuration WHERE guild_id = $1",
            context.guild.as_i64(),
        )
        .fetch_one(Bot::global().postgres())
        .await?;

        let interactions = [
            xp_interaction_builder(
                context.user,
                XPStage::SelectedRandomOrSet { is_random: true },
                data.1,
            )
            .build(),
            xp_interaction_builder(
                context.user,
                XPStage::SelectedRandomOrSet { is_random: false },
                data.1,
            )
            .build(),
            xp_interaction_builder(context.user, XPStage::MessageCooldown, data.1).build(),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.to_vec())
            .await?;

        let current_setting = match xp_configuration.set_xp_per_message {
            Some(set_xp) => format!("**{set_xp}** XP per message"),
            None => format!(
                "Between **{}** to **{}** XP per message",
                xp_configuration.min_xp_per_message.unwrap(),
                xp_configuration.max_xp_per_message.unwrap()
            ),
        };

        let help_text = r"When earning XP, would you like to set a fixed amount of XP per message, or a random amount between a minimum and maximum?

> Reaper can award XP to members when they send messages.
> This XP contributes to their level and progression in your server.
> 
> You can choose how much XP is given per message and how:
> - **Fixed Amount** → Every message gives the same amount of XP (e.g. 10 XP).
> - **Random Amount** → Every message gives a random amount of XP between a minimum and maximum (e.g. from 5 to 15 XP).
> 
> A cooldown will be configured later to prevent earning XP too quickly. Messages sent during this cooldown will not count towards XP.";

        entry
            .reply(
                ctx,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!(
                                "{help_text}\n\nYour current setting is: {current_setting}"
                            ))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(interactions[0].id.to_string())
                            .label("Random")
                            .style(ButtonStyle::Primary),
                        CreateButton::new(interactions[1].id.to_string())
                            .label("Specific")
                            .style(ButtonStyle::Primary),
                        CreateButton::new(interactions[2].id.to_string())
                            .label("Skip")
                            .style(ButtonStyle::Secondary),
                    ])]),
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct SelectedRandomOrSet;
#[async_trait::async_trait]
impl ConfigStage for SelectedRandomOrSet {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "selected_random_or_set")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "random_or_set"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let ConfigInteraction::XP { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::SelectedRandomOrSet { is_random } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let context = ctx.get_populated_context()?;

        entry
            .modal(
                ctx,
                CreateModal::new(
                    "xp_modal",
                    if *is_random {
                        "Random XP Configuration"
                    } else {
                        "Set XP Configuration"
                    },
                )
                .components(if *is_random {
                    vec![
                        CreateActionRow::InputText(
                            CreateInputText::new(
                                InputTextStyle::Short,
                                "Minimum XP Gain",
                                "min_xp",
                            )
                            .placeholder("Recommended: 15")
                            .required(true),
                        ),
                        CreateActionRow::InputText(
                            CreateInputText::new(
                                InputTextStyle::Short,
                                "Maximum XP Gain",
                                "max_xp",
                            )
                            .placeholder("Recommended: 40")
                            .required(true),
                        ),
                    ]
                } else {
                    vec![CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, "Set XP", "set_xp")
                            .placeholder("Recommended: 25")
                            .required(true),
                    )]
                }),
            )
            .await?;

        let message = entry.component()?.get_response(&context.ctx.http).await?;

        let modal_collector = message
            .await_modal_interaction(context.ctx)
            .author_id(context.user.as_serenity_id())
            .timeout(std::time::Duration::new(300, 0));

        if let Some(modal_interaction) = modal_collector.await {
            modal_interaction
                .create_response(
                    &context.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;

            if let ActionRowComponent::InputText(text) =
                &modal_interaction.data.components[0].components[0]
            {
                let Ok(xp_value) = text.value.as_ref().unwrap().parse::<i32>().map_err(|_| {
                    ResponseError::Execution(ExecutionError::Input(InputError::InvalidNumber))
                }) else {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidNumber,
                    )));
                };

                if xp_value < 0 {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidNumber,
                    )));
                }

                if *is_random {
                    sqlx::query!(
                        "UPDATE xp_configuration SET min_xp_per_message = $1, set_xp_per_message = null WHERE guild_id = $2",
                        xp_value,
                        context.guild.as_i64(),
                    )
                    .execute(Bot::global().postgres())
                    .await?;
                } else {
                    sqlx::query!(
                        "UPDATE xp_configuration SET set_xp_per_message = $1, min_xp_per_message = null, max_xp_per_message = null WHERE guild_id = $2",
                        xp_value,
                        context.guild.as_i64(),
                    )
                    .execute(Bot::global().postgres())
                    .await?;

                    return advance_to(MessageCooldown, ctx, entry, data).await;
                }
            }

            if let ActionRowComponent::InputText(text) =
                &modal_interaction.data.components[1].components[0]
            {
                let Ok(max_xp) = text.value.as_ref().unwrap().parse::<i32>().map_err(|_| {
                    ResponseError::Execution(ExecutionError::Input(InputError::InvalidNumber))
                }) else {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidNumber,
                    )));
                };

                if max_xp < 0 {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidNumber,
                    )));
                }

                sqlx::query!(
                    "UPDATE xp_configuration SET max_xp_per_message = $1 WHERE guild_id = $2",
                    max_xp,
                    context.guild.as_i64(),
                )
                .execute(Bot::global().postgres())
                .await?;

                return advance_to(MessageCooldown, ctx, entry, data).await;
            }
        }

        Err(ResponseError::Execution(ExecutionError::Input(
            InputError::Timeout {
                duration: "5 minutes".to_string(),
            },
        )))
    }
}

#[derive(Debug)]
pub struct MessageCooldown;
#[async_trait::async_trait]
impl ConfigStage for MessageCooldown {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "message_cooldown")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        None
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let cooldown = sqlx::query!(
            "SELECT message_cooldown FROM xp_configuration WHERE guild_id = $1",
            context.guild.as_i64(),
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .message_cooldown;

        let cooldown_duration =
            format_duration(StdDuration::from_secs(cooldown as u64)).to_string();
        let help_text = r"When earning XP, how long should pass before someone can gain XP again?

> The cooldown prevents spam and ensures XP is earned fairly.
> 
> You can choose how long the cooldown should be:
> - For example, a cooldown of 30 seconds means a member can only gain XP once every 30 seconds, no matter how many messages they send.
> - Shorter cooldowns reward more frequent activity, while longer cooldowns slow down progression.
> 
> Messages sent during the cooldown will not give XP.";

        let interactions = [
            xp_interaction_builder(context.user, XPStage::ChangeMessageCooldown, data.1).build(),
            xp_interaction_builder(context.user, XPStage::MaxLevel, data.1).build(),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.to_vec())
            .await?;

        entry
            .reply(
                ctx,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!(
                                "{help_text}\n\nThe current cooldown is: **{cooldown_duration}**"
                            ))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(interactions[0].id.to_string())
                            .label("Change")
                            .style(ButtonStyle::Primary),
                        CreateButton::new(interactions[1].id.to_string())
                            .label("Skip")
                            .style(ButtonStyle::Secondary),
                    ])]),
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct ChangeMessageCooldown;
#[async_trait::async_trait]
impl ConfigStage for ChangeMessageCooldown {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "change_message_cooldown")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "message_cooldown"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;

        entry
            .modal(
                ctx,
                CreateModal::new("message_cooldown_modal", "Message Cooldown").components(vec![
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, "Duration", "message_cooldown")
                            .placeholder("1m")
                            .required(true),
                    ),
                ]),
            )
            .await?;

        let message = entry.component()?.get_response(&context.ctx.http).await?;

        let modal_collector = message
            .await_modal_interaction(context.ctx)
            .author_id(context.user.as_serenity_id())
            .timeout(std::time::Duration::new(300, 0));

        if let Some(modal_interaction) = modal_collector.await {
            modal_interaction
                .create_response(
                    &context.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;

            if let ActionRowComponent::InputText(text) =
                &modal_interaction.data.components[0].components[0]
            {
                let value = text.value.clone().unwrap();
                if value.is_empty() {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidDuration,
                    )));
                }
                let Some(duration) = Duration::new(value.as_str()).in_seconds() else {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidDuration,
                    )));
                };
                if duration < 0 {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidDuration,
                    )));
                }

                sqlx::query!(
                    "UPDATE xp_configuration SET message_cooldown = $1 WHERE guild_id = $2",
                    duration as i32,
                    context.guild.as_i64()
                )
                .execute(Bot::global().postgres())
                .await?;

                return advance_to(MaxLevel, ctx, entry, data).await;
            }
        }

        Err(ResponseError::Execution(ExecutionError::Input(
            InputError::Timeout {
                duration: "5 minutes".to_string(),
            },
        )))
    }
}

#[derive(Debug)]
pub struct MaxLevel;
#[async_trait::async_trait]
impl ConfigStage for MaxLevel {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "max_level")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        None
    }

    async fn router(
        &self,
        _ctx: &Context<'_>,
        _entry: &ConfigEntry,
        _data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        unimplemented!()
    }
}
