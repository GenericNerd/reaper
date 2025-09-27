use std::{collections::BTreeMap, time::Duration as StdDuration};

use humantime::format_duration;
use serenity::all::{
    ActionRowComponent, ButtonStyle, ChannelType, ComponentInteractionDataKind, CreateActionRow,
    CreateButton, CreateEmbed, CreateInputText, CreateModal, CreateSelectMenu,
    CreateSelectMenuKind, InputTextStyle, ReactionType,
};

use crate::{
    components::config::{
        Complete, ConfigEntry, ConfigStage, EMBED_COLOR, EditMode, advance_to, boards::BoardsEnter,
        interaction_builder,
    },
    models::{
        bot::Bot,
        channel::Channel,
        context::Context,
        duration::Duration,
        interactions::{
            Interaction, InteractionBuilder,
            config::{
                BoardsStage, ChannelMultiplier, ConfigInteraction, Reward, RoleMultiplier, XPStage,
            },
        },
        response::{
            ExecutionError, InputError, InternalError, Response, ResponseError, ResponseResult,
        },
        role::Role,
        user::User,
    },
};

const XP_TITLE: &str = "Configuration - Levelling";

fn xp_interaction_builder(user: User, stage: XPStage, single_category: bool) -> InteractionBuilder {
    interaction_builder(user, ConfigInteraction::Xp { stage }, single_category)
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
            interaction_builder(
                context.user,
                ConfigInteraction::Boards {
                    stage: BoardsStage::Enter,
                },
                data.1,
            )
            .build(),
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

        sqlx::query!(
            "INSERT INTO xp_configuration (guild_id) VALUES ($1) ON CONFLICT (guild_id) DO NOTHING",
            context.guild.as_i64(),
        )
        .execute(Bot::global().postgres())
        .await?;
        sqlx::query!(
            "INSERT INTO xp_level_up_messages (guild_id) VALUES ($1) ON CONFLICT (guild_id) DO NOTHING",
            context.guild.as_i64(),
        )
        .execute(Bot::global().postgres())
        .await?;

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
        let ConfigInteraction::Xp { stage } = data.0 else {
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

            let min_xp = if let ActionRowComponent::InputText(text) =
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
                    xp_value
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
            } else {
                return Err(ResponseError::Execution(ExecutionError::Internal(
                    InternalError::InvalidInteractionType,
                )));
            };

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

                if min_xp > max_xp {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidMinXP,
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
        let help_text = r"When earning XP, how much time should pass before someone can gain XP again?

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
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let max_level = sqlx::query!(
            "SELECT max_level FROM xp_configuration WHERE guild_id = $1",
            context.guild.as_i64(),
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .max_level;

        let current_setting = match max_level {
            Some(max_level) => format!("**Level {max_level}**"),
            None => "**No limit**".to_string(),
        };

        let help_text = r"You can set a maximum level that members can reach.

> - Once members hit this level, they will stop earning XP.
> - This can be useful to cap progression and prevent infinite grinding.
> - If you don't want a limit, you can click the `No limit` button.";

        let interactions = [
            xp_interaction_builder(
                context.user,
                XPStage::ChangeMaxLevel { is_limited: true },
                data.1,
            )
            .build(),
            xp_interaction_builder(
                context.user,
                XPStage::ChangeMaxLevel { is_limited: false },
                data.1,
            )
            .build(),
            xp_interaction_builder(context.user, XPStage::StackRewards, data.1).build(),
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
                                "{help_text}\n\nThe current limit is: {current_setting}"
                            ))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(interactions[0].id.to_string())
                            .label(if max_level.is_some() {
                                "Change maximum"
                            } else {
                                "Set maximum"
                            })
                            .style(ButtonStyle::Primary),
                        CreateButton::new(interactions[1].id.to_string())
                            .label("No limit")
                            .style(ButtonStyle::Danger),
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
pub struct ChangeMaxLevel;
#[async_trait::async_trait]
impl ConfigStage for ChangeMaxLevel {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "change_max_level")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "max_level"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::ChangeMaxLevel { is_limited } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        if !is_limited {
            sqlx::query!(
                "UPDATE xp_configuration SET max_level = null WHERE guild_id = $1",
                context.guild.as_i64()
            )
            .execute(Bot::global().postgres())
            .await?;

            return advance_to(StackRewards, ctx, entry, data).await;
        }

        entry
            .modal(
                ctx,
                CreateModal::new("max_level_modal", "Maximum Level").components(vec![
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, "Maximum Level", "max_level")
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
                let Ok(max_level) = text.value.as_ref().unwrap().parse::<i32>().map_err(|_| {
                    ResponseError::Execution(ExecutionError::Input(InputError::InvalidMaxLevel))
                }) else {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidMaxLevel,
                    )));
                };

                if max_level <= 0 {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidMaxLevel,
                    )));
                }

                sqlx::query!(
                    "UPDATE xp_configuration SET max_level = $1 WHERE guild_id = $2",
                    max_level,
                    context.guild.as_i64()
                )
                .execute(Bot::global().postgres())
                .await?;

                return advance_to(StackRewards, ctx, entry, data).await;
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
pub struct StackRewards;
#[async_trait::async_trait]
impl ConfigStage for StackRewards {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "stack_rewards")
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

        let stacking = sqlx::query!(
            "SELECT stack_rewards FROM xp_configuration WHERE guild_id = $1",
            context.guild.as_i64(),
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .stack_rewards;

        let current_setting = if stacking {
            "**Enabled**"
        } else {
            "**Disabled**"
        };

        let help_text = r"When members level up, you can reward them with special roles.

> - By default, when a member earns a new reward role, their previous reward is replaced.
> - Enabling stacking rewards lets members **keep all the roles** they earn as they level up.
> - Stackable rewards are useful if you want leveling to feel like progression (e.g. keeping Bronze, Silver, and Gold).
> - Non-stacking rewards are useful if you only want members to hold the **highest role** they've unlocked.";

        let interactions = [
            xp_interaction_builder(
                context.user,
                XPStage::ChangeStackRewards { is_enabled: true },
                data.1,
            )
            .build(),
            xp_interaction_builder(
                context.user,
                XPStage::ChangeStackRewards { is_enabled: false },
                data.1,
            )
            .build(),
            xp_interaction_builder(context.user, XPStage::StackMultipliers, data.1).build(),
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
                                "{help_text}\n\nStacking rewards is currently: {current_setting}"
                            ))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(interactions[0].id.to_string())
                            .label("Stacking")
                            .style(ButtonStyle::Primary),
                        CreateButton::new(interactions[1].id.to_string())
                            .label("No stacking")
                            .style(ButtonStyle::Danger),
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
pub struct ChangeStackRewards;
#[async_trait::async_trait]
impl ConfigStage for ChangeStackRewards {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "change_stack_rewards")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "stack_rewards"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::ChangeStackRewards { is_enabled } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        sqlx::query!(
            "UPDATE xp_configuration SET stack_rewards = $1 WHERE guild_id = $2",
            is_enabled,
            context.guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await?;

        advance_to(StackMultipliers, ctx, entry, data).await
    }
}

#[derive(Debug)]
pub struct StackMultipliers;
#[async_trait::async_trait]
impl ConfigStage for StackMultipliers {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "stack_multipliers")
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

        let stacking = sqlx::query!(
            "SELECT stack_multipliers FROM xp_configuration WHERE guild_id = $1",
            context.guild.as_i64(),
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .stack_multipliers;

        let current_setting = if stacking {
            "**Enabled**"
        } else {
            "**Disabled**"
        };

        let help_text = r"You can configure how XP boosts are applied from roles and channels.

> - Boosts increase the amount of XP a member earns. They can come from special roles or specific channels.
> - By default, boosts are **stacked together**. For example, a user with a role giving +20% XP chatting in a channel with +10% XP will receive a total +30% XP.
> - If you disable stacking, only the **highest single boost** will apply. In the same example, the user would receive only the +20% XP boost.";

        let interactions = [
            xp_interaction_builder(
                context.user,
                XPStage::ChangeStackMultipliers { is_enabled: true },
                data.1,
            )
            .build(),
            xp_interaction_builder(
                context.user,
                XPStage::ChangeStackMultipliers { is_enabled: false },
                data.1,
            )
            .build(),
            xp_interaction_builder(context.user, XPStage::MultiplierCap, data.1).build(),
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
                                "{help_text}\n\nStacking multipliers is currently: {current_setting}"
                            ))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(interactions[0].id.to_string())
                            .label("Stacking")
                            .style(ButtonStyle::Primary),
                        CreateButton::new(interactions[1].id.to_string())
                            .label("No stacking")
                            .style(ButtonStyle::Danger),
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
pub struct ChangeStackMultipliers;
#[async_trait::async_trait]
impl ConfigStage for ChangeStackMultipliers {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "change_stack_multipliers")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "stack_multipliers"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::ChangeStackMultipliers { is_enabled } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        sqlx::query!(
            "UPDATE xp_configuration SET stack_multipliers = $1 WHERE guild_id = $2",
            is_enabled,
            context.guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await?;

        advance_to(MultiplierCap, ctx, entry, data).await
    }
}

#[derive(Debug)]
pub struct MultiplierCap;
#[async_trait::async_trait]
impl ConfigStage for MultiplierCap {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "multiplier_cap")
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

        let multiplier_cap = sqlx::query!(
            "SELECT multiplier_cap FROM xp_configuration WHERE guild_id = $1",
            context.guild.as_i64(),
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .multiplier_cap;

        let current_setting = if let Some(cap) = multiplier_cap {
            format!("**Capped at {cap}%**")
        } else {
            "**Uncapped**".to_string()
        };

        let help_text = r"You can set a cap on how much XP boost a user can receive in total.

> - XP boosts come from roles and channels, and they increase the amount of XP a member earns.
> - By default, multiple boosts can stack without limit (e.g. a role with +20% XP and a channel with +10% XP would give +30% total).
> - Setting an XP boost cap defines the maximum bonus a member can get.
> - For example, with a 25% cap, the same user would receive only +25% even though their total stacked boost was +30%.";

        let interactions = [
            xp_interaction_builder(
                context.user,
                XPStage::ChangeMultiplierCap { is_limited: true },
                data.1,
            )
            .build(),
            xp_interaction_builder(
                context.user,
                XPStage::ChangeMultiplierCap { is_limited: false },
                data.1,
            )
            .build(),
            xp_interaction_builder(context.user, XPStage::ResetXpOnLeave, data.1).build(),
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
                                "{help_text}\n\nMultipliers are: {current_setting}"
                            ))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(interactions[0].id.to_string())
                            .label("Limited")
                            .style(ButtonStyle::Primary),
                        CreateButton::new(interactions[1].id.to_string())
                            .label("No cap")
                            .style(ButtonStyle::Danger),
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
pub struct ChangeMultiplierCap;
#[async_trait::async_trait]
impl ConfigStage for ChangeMultiplierCap {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "change_multiplier_cap")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "multiplier_cap"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::ChangeMultiplierCap { is_limited } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        if !*is_limited {
            sqlx::query!(
                "UPDATE xp_configuration SET multiplier_cap = NULL WHERE guild_id = $1",
                context.guild.as_i64()
            )
            .execute(Bot::global().postgres())
            .await?;
            return advance_to(ResetXpOnLeave, ctx, entry, data).await;
        }

        entry
            .modal(
                ctx,
                CreateModal::new("multiplier_cap_modal", "Multiplier Cap").components(vec![
                    CreateActionRow::InputText(
                        CreateInputText::new(
                            InputTextStyle::Short,
                            "Multiplier Cap (in %)",
                            "multiplier_cap",
                        )
                        .placeholder("Recommended: 50")
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
                        InputError::InvalidMultiplierCap,
                    )));
                }
                let Ok(multiplier_cap) =
                    text.value.as_ref().unwrap().parse::<i32>().map_err(|_| {
                        ResponseError::Execution(ExecutionError::Input(
                            InputError::InvalidMultiplierCap,
                        ))
                    })
                else {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidMultiplierCap,
                    )));
                };

                if multiplier_cap <= 0 {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidMultiplierCap,
                    )));
                }

                sqlx::query!(
                    "UPDATE xp_configuration SET multiplier_cap = $1 WHERE guild_id = $2",
                    (multiplier_cap as f32 / 100.0) + 1.0,
                    context.guild.as_i64()
                )
                .execute(Bot::global().postgres())
                .await?;

                return advance_to(ResetXpOnLeave, ctx, entry, data).await;
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
pub struct ResetXpOnLeave;
#[async_trait::async_trait]
impl ConfigStage for ResetXpOnLeave {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "reset_xp_on_leave")
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

        let reset_xp_on_leave = sqlx::query!(
            "SELECT reset_level_on_leave FROM xp_configuration WHERE guild_id = $1",
            context.guild.as_i64(),
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .reset_level_on_leave;

        let current_setting = if reset_xp_on_leave {
            "**Enabled**"
        } else {
            "**Disabled**"
        };

        let help_text = r"Would you like to reset a member's level when they leave the server?

> - If enabled, leaving the server will clear all of a member's XP and levels.
> - If disabled, members will keep their XP and level if they rejoin.";

        let interactions = [
            xp_interaction_builder(
                context.user,
                XPStage::ChangeResetXpOnLeave { is_enabled: true },
                data.1,
            )
            .build(),
            xp_interaction_builder(
                context.user,
                XPStage::ChangeResetXpOnLeave { is_enabled: false },
                data.1,
            )
            .build(),
            xp_interaction_builder(context.user, XPStage::LevelUpMessages, data.1).build(),
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
                            .description(format!("{help_text}\n\nCurrently: {current_setting}"))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(interactions[0].id.to_string())
                            .label("Enable")
                            .style(ButtonStyle::Primary),
                        CreateButton::new(interactions[1].id.to_string())
                            .label("Disable")
                            .style(ButtonStyle::Danger),
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
pub struct ChangeResetXpOnLeave;
#[async_trait::async_trait]
impl ConfigStage for ChangeResetXpOnLeave {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "change_reset_xp_on_leave")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "reset_xp_on_leave"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::ChangeResetXpOnLeave { is_enabled } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        sqlx::query!(
            "UPDATE xp_configuration SET reset_level_on_leave = $1 WHERE guild_id = $2",
            is_enabled,
            context.guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await?;

        advance_to(LevelUpMessages, ctx, entry, data).await
    }
}

#[derive(Debug)]
pub struct LevelUpMessages;
#[async_trait::async_trait]
impl ConfigStage for LevelUpMessages {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "level_up_messages")
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

        let enabled = sqlx::query!(
            "SELECT enabled FROM xp_level_up_messages WHERE guild_id = $1",
            context.guild.as_i64(),
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .enabled;

        let current_setting = if enabled {
            "**Enabled**"
        } else {
            "**Disabled**"
        };

        let help_text = if enabled {
            r"Would you like to configure or disable level up messages?

> - Currently, level up messages are enabled and will be sent when members reach a new level.
> - You can configure the message text or channel if you'd like to customize them.
> - If you disable them, members will still level up but no messages will be sent."
        } else {
            r"Would you like to enable level up messages?

> - Level up messages are sent when members gain a new level from XP.
> - They can be customized to fit your server's style.
> - If disabled, members will still level up but no messages will be sent."
        };

        let interactions = [
            xp_interaction_builder(
                context.user,
                XPStage::ChangeLevelUpMessages { is_enabled: true },
                data.1,
            )
            .build(),
            xp_interaction_builder(
                context.user,
                XPStage::ChangeLevelUpMessages { is_enabled: false },
                data.1,
            )
            .build(),
            xp_interaction_builder(context.user, XPStage::RewardsEnter, data.1).build(),
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
                            .description(format!("{help_text}\n\nCurrently: {current_setting}"))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(interactions[0].id.to_string())
                            .label("Enable")
                            .style(ButtonStyle::Primary),
                        CreateButton::new(interactions[1].id.to_string())
                            .label("Disable")
                            .style(ButtonStyle::Danger),
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
pub struct ChangeLevelUpMessages;
#[async_trait::async_trait]
impl ConfigStage for ChangeLevelUpMessages {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "change_level_up_messages")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "level_up_messages"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::ChangeLevelUpMessages { is_enabled } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        sqlx::query!(
            "UPDATE xp_level_up_messages SET enabled = $1 WHERE guild_id = $2",
            is_enabled,
            context.guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await?;

        if *is_enabled {
            advance_to(DmOnLevelUp, ctx, entry, data).await
        } else {
            advance_to(RewardsEnter, ctx, entry, data).await
        }
    }
}

#[derive(Debug)]
pub struct DmOnLevelUp;
#[async_trait::async_trait]
impl ConfigStage for DmOnLevelUp {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "dm_on_level_up")
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

        let row = sqlx::query!(
            "SELECT dm_message, channel FROM xp_level_up_messages WHERE guild_id = $1",
            context.guild.as_i64(),
        )
        .fetch_one(Bot::global().postgres())
        .await?;

        let current_setting = if row.dm_message {
            "**In DMs**".to_string()
        } else {
            let channel = row.channel;
            if let Some(channel_id) = channel {
                format!("**In <#{channel_id}>**")
            } else {
                "**In the spoken channel**".to_string()
            }
        };

        let help_text = r"Would you like to send level up messages in DMs or in a channel?

> - **DMs** → The member will receive their level up message privately.
> - **Channel** → Messages will be posted in the server where everyone can see them.
-# (If you choose channel, you'll also select whether to send them in the channel the user spoke in, or always to one specific channel.)";

        let interactions = [
            xp_interaction_builder(
                context.user,
                XPStage::ChangeDmOnLevelUp { is_enabled: true },
                data.1,
            )
            .build(),
            xp_interaction_builder(
                context.user,
                XPStage::ChangeDmOnLevelUp { is_enabled: false },
                data.1,
            )
            .build(),
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
                            .description(format!("{help_text}\n\nCurrently: {current_setting}"))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(interactions[0].id.to_string())
                            .label("In DMs")
                            .style(ButtonStyle::Primary),
                        CreateButton::new(interactions[1].id.to_string())
                            .label("In Channel")
                            .style(ButtonStyle::Primary),
                    ])]),
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct ChangeDmOnLevelUp;
#[async_trait::async_trait]
impl ConfigStage for ChangeDmOnLevelUp {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "change_dm_on_level_up")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "dm_on_level_up"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::ChangeDmOnLevelUp { is_enabled } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        sqlx::query!(
            "UPDATE xp_level_up_messages SET dm_message = $1 WHERE guild_id = $2",
            is_enabled,
            context.guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await?;

        if *is_enabled {
            advance_to(LevelUpMessage, ctx, entry, data).await
        } else {
            advance_to(LevelUpChannel, ctx, entry, data).await
        }
    }
}

#[derive(Debug)]
pub struct LevelUpChannel;
#[async_trait::async_trait]
impl ConfigStage for LevelUpChannel {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "level_up_channel")
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

        let channel = sqlx::query!(
            "SELECT channel FROM xp_level_up_messages WHERE guild_id = $1",
            context.guild.as_i64(),
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .channel;

        let current_setting = if let Some(channel_id) = channel {
            format!("Messages will be sent to **<#{channel_id}>**")
        } else {
            "Messages will be sent to the channel spoken in".to_string()
        };

        let help_text = r"You can choose how channel level up messages should be sent.

> - **Specific Channel** → Always send level up messages to one fixed channel (you'll choose this channel from the dropdown).
> - **Channel Spoken In** → Send the level up message directly in the channel where the user gained their level.";

        let interactions = [
            xp_interaction_builder(context.user, XPStage::ChangeLevelUpChannel, data.1).build(),
            xp_interaction_builder(context.user, XPStage::ChangeLevelUpChannel, data.1).build(),
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
                            .description(format!("{help_text}\n\nCurrently: {current_setting}"))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![
                        CreateActionRow::SelectMenu(CreateSelectMenu::new(
                            interactions[0].id.to_string(),
                            CreateSelectMenuKind::Channel {
                                channel_types: Some(vec![
                                    ChannelType::Text,
                                    ChannelType::Forum,
                                    ChannelType::PublicThread,
                                    ChannelType::PrivateThread,
                                ]),
                                default_channels: None,
                            },
                        )),
                        CreateActionRow::Buttons(vec![
                            CreateButton::new(interactions[1].id.to_string())
                                .label("Channel Spoken In")
                                .style(ButtonStyle::Primary),
                        ]),
                    ]),
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct ChangeLevelUpChannel;
#[async_trait::async_trait]
impl ConfigStage for ChangeLevelUpChannel {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "change_level_up_channel")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "level_up_channel"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;

        if let ComponentInteractionDataKind::Button = &entry.component()?.data.kind {
            sqlx::query!(
                "UPDATE xp_level_up_messages SET channel = NULL WHERE guild_id = $1",
                context.guild.as_i64()
            )
            .execute(Bot::global().postgres())
            .await?;
            return advance_to(LevelUpMessage, ctx, entry, data).await;
        }

        let ComponentInteractionDataKind::ChannelSelect { values } = &entry.component()?.data.kind
        else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let channel = values.first().ok_or_else(|| {
            ResponseError::Execution(ExecutionError::Input(InputError::NoChannelSelected))
        })?;
        let channel = Channel::from(*channel);

        sqlx::query!(
            "UPDATE xp_level_up_messages SET channel = $1 WHERE guild_id = $2",
            channel.as_i64(),
            context.guild.as_i64(),
        )
        .execute(Bot::global().postgres())
        .await?;

        advance_to(LevelUpMessage, ctx, entry, data).await
    }
}

#[derive(Debug)]
pub struct LevelUpMessage;
#[async_trait::async_trait]
impl ConfigStage for LevelUpMessage {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "level_up_message")
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

        let message = sqlx::query!(
            "SELECT message FROM xp_level_up_messages WHERE guild_id = $1",
            context.guild.as_i64(),
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .message;
        let has_message = message.is_some();

        let help_text = format!(
            r"What would you like your level up message to say?

> You can include the following placeholders in your message:
> - `{{user.name}}` → The user's name (e.g. `{}`)
> - `{{user.mention}}` → Mentions the user (e.g. <@{}>)
> - `{{user.level}}` → The user's new level (e.g. 100)",
            entry.component()?.user.name,
            context.user.as_i64(),
        );

        let interactions = if has_message {
            vec![
                xp_interaction_builder(context.user, XPStage::ChangeLevelUpMessage, data.1).build(),
                xp_interaction_builder(context.user, XPStage::RewardsEnter, data.1).build(),
            ]
        } else {
            vec![
                xp_interaction_builder(context.user, XPStage::ChangeLevelUpMessage, data.1).build(),
            ]
        };

        Bot::global()
            .interaction_state()
            .register(interactions.clone())
            .await?;

        entry
            .reply(
                ctx,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!(
                                "{help_text}\n\nCurrent message:\n{}",
                                match message {
                                    Some(msg) => format!("```{msg}```"),
                                    None => "**No message**".to_string(),
                                }
                            ))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(if has_message {
                        vec![
                            CreateButton::new(interactions[0].id.to_string())
                                .label("Change")
                                .style(ButtonStyle::Primary),
                            CreateButton::new(interactions[1].id.to_string())
                                .label("Skip")
                                .style(ButtonStyle::Secondary),
                        ]
                    } else {
                        vec![
                            CreateButton::new(interactions[0].id.to_string())
                                .label("Change")
                                .style(ButtonStyle::Primary),
                        ]
                    })]),
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct ChangeLevelUpMessage;
#[async_trait::async_trait]
impl ConfigStage for ChangeLevelUpMessage {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "change_level_up_message")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "level_up_message"))
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
                CreateModal::new("level_up_message_modal", "Level Up Message").components(vec![
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, "Message", "message")
                            .placeholder(
                                "Recommended: {user.name} leveled up to level {user.level}",
                            )
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

                sqlx::query!(
                    "UPDATE xp_level_up_messages SET message = $1 WHERE guild_id = $2",
                    value,
                    context.guild.as_i64(),
                )
                .execute(Bot::global().postgres())
                .await?;

                return advance_to(RewardsEnter, ctx, entry, data).await;
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
pub struct RewardsEnter;
#[async_trait::async_trait]
impl ConfigStage for RewardsEnter {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "rewards_enter")
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
            xp_interaction_builder(
                context.user,
                XPStage::Rewards {
                    page: 0,
                    rewards: None,
                },
                data.1,
            )
            .build(),
            xp_interaction_builder(context.user, XPStage::RoleMultiplierEnter, data.1).build(),
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
                            .description("Would you like to configure level rewards?")
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
pub struct Rewards;

impl Rewards {
    const MAX_REWARDS_PER_PAGE: usize = 25;
    fn max_pages(reward_count: usize) -> usize {
        ((reward_count as f64 / Rewards::MAX_REWARDS_PER_PAGE as f64).ceil() as usize).max(1)
    }

    fn generate_interactions(
        user: User,
        single_category: bool,
        rewards: &[Reward],
        current_page: usize,
        max_pages: usize,
        edit_mode: Option<EditMode>,
    ) -> (Vec<Interaction>, Vec<CreateActionRow>) {
        let mut components = vec![];
        let mut interactions = vec![];

        if let Some(edit_mode) = edit_mode {
            interactions.append(&mut vec![
                xp_interaction_builder(
                    user,
                    match edit_mode {
                        EditMode::Add => XPStage::AddReward {
                            page: current_page,
                            rewards: rewards.to_owned(),
                        },
                        EditMode::Remove => XPStage::RemoveReward {
                            page: current_page,
                            rewards: rewards.to_owned(),
                        },
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::Rewards {
                        page: current_page,
                        rewards: Some(rewards.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::Rewards {
                        page: current_page,
                        rewards: None,
                    },
                    single_category,
                )
                .build(),
            ]);
            components.append(&mut vec![
                CreateActionRow::SelectMenu(CreateSelectMenu::new(
                    interactions[0].id.to_string(),
                    CreateSelectMenuKind::Role {
                        default_roles: None,
                    },
                )),
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[1].id.to_string())
                        .style(ButtonStyle::Secondary)
                        .label("Cancel edit"),
                    CreateButton::new(interactions[2].id.to_string())
                        .style(ButtonStyle::Danger)
                        .label("Revert"),
                ]),
            ]);
        } else {
            interactions.append(&mut vec![
                xp_interaction_builder(
                    user,
                    XPStage::Rewards {
                        page: current_page - 1,
                        rewards: Some(rewards.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::Rewards {
                        page: current_page + 1,
                        rewards: Some(rewards.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::AddReward {
                        page: current_page,
                        rewards: rewards.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::RemoveReward {
                        page: current_page,
                        rewards: rewards.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::SaveRewards {
                        rewards: rewards.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::Rewards {
                        page: current_page,
                        rewards: None,
                    },
                    single_category,
                )
                .build(),
            ]);
            components.append(&mut vec![
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[0].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("◀".to_string()))
                        .disabled(current_page == 0),
                    CreateButton::new(interactions[1].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("▶".to_string()))
                        .disabled((current_page + 1) == max_pages),
                    CreateButton::new(interactions[2].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("➕".to_string())),
                    CreateButton::new(interactions[3].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("➖".to_string()))
                        .disabled(rewards.is_empty()),
                    CreateButton::new(interactions[4].id.to_string())
                        .emoji('✅')
                        .style(ButtonStyle::Success),
                ]),
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[5].id.to_string())
                        .style(ButtonStyle::Danger)
                        .label("Revert"),
                ]),
            ]);
        }

        (interactions, components)
    }

    async fn generate_message(
        user: User,
        single_category: bool,
        rewards: &Vec<Reward>,
        current_page: usize,
        edit_mode: Option<EditMode>,
    ) -> Result<Response, ResponseError> {
        let mut rewards_by_level: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
        for reward in rewards {
            if let std::collections::btree_map::Entry::Vacant(e) =
                rewards_by_level.entry(reward.level)
            {
                e.insert(vec![reward.role]);
            } else {
                rewards_by_level
                    .get_mut(&reward.level)
                    .unwrap()
                    .push(reward.role);
            }
        }

        let fields = rewards_by_level
            .iter()
            .map(|(level, roles)| {
                (
                    format!("Level {level}"),
                    roles
                        .iter()
                        .map(|role| format!("<@&{role}>"))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    true,
                )
            })
            .collect::<Vec<_>>();

        let start = current_page * Rewards::MAX_REWARDS_PER_PAGE;
        let mut end = start + Rewards::MAX_REWARDS_PER_PAGE;
        if end > fields.len() {
            end = fields.len();
        }
        let fields = fields[start..end].to_vec();
        let max_pages = Rewards::max_pages(rewards_by_level.len());
        let (interactions, components) = Self::generate_interactions(
            user,
            single_category,
            rewards,
            current_page,
            max_pages,
            edit_mode,
        );

        Bot::global()
            .interaction_state()
            .register(interactions.clone())
            .await?;

        Ok(Response::new()
            .embed(
                CreateEmbed::new()
                    .title(format!(
                        "Level Rewards - Page {}/{}",
                        current_page + 1,
                        max_pages
                    ))
                    .fields(fields)
                    .color(EMBED_COLOR),
            )
            .components(components))
    }
}

#[async_trait::async_trait]
impl ConfigStage for Rewards {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "rewards")
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
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::Rewards { page, rewards } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let rewards = match rewards {
            Some(rewards) => rewards,
            None => {
                &sqlx::query_as!(
                    Reward,
                    "SELECT role, level FROM xp_rewards WHERE guild_id = $1",
                    context.guild.as_i64()
                )
                .fetch_all(Bot::global().postgres())
                .await?
            }
        };

        entry
            .reply(
                ctx,
                Rewards::generate_message(context.user, data.1, rewards, *page, None).await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct AddReward;
#[async_trait::async_trait]
impl ConfigStage for AddReward {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "add_reward")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "rewards"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::AddReward { page, rewards } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let mut rewards = rewards.clone();

        if let ComponentInteractionDataKind::RoleSelect { values } = &entry.component()?.data.kind {
            let role = values.first().ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoRoleSelected))
            })?;
            let role = Role::from(*role);

            entry
                .modal(
                    ctx,
                    CreateModal::new("reward_modal", "Add Reward").components(vec![
                        CreateActionRow::InputText(
                            CreateInputText::new(InputTextStyle::Short, "Level", "level")
                                .placeholder("30")
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
                    let Ok(level) = text.value.as_ref().unwrap().parse::<i64>().map_err(|_| {
                        ResponseError::Execution(ExecutionError::Input(
                            InputError::InvalidStrikeCount,
                        ))
                    }) else {
                        return Err(ResponseError::Execution(ExecutionError::Input(
                            InputError::InvalidNumber,
                        )));
                    };

                    rewards.push(Reward {
                        role: role.as_i64(),
                        level,
                    });

                    return advance_to(
                        Rewards,
                        ctx,
                        entry,
                        (
                            &ConfigInteraction::Xp {
                                stage: XPStage::Rewards {
                                    page: *page,
                                    rewards: Some(rewards.clone()),
                                },
                            },
                            data.1,
                        ),
                    )
                    .await;
                }
                return Err(ResponseError::Execution(ExecutionError::Internal(
                    InternalError::InvalidInteractionType,
                )));
            }

            return Err(ResponseError::Execution(ExecutionError::Input(
                InputError::Timeout {
                    duration: "5 minutes".to_string(),
                },
            )));
        }

        entry
            .reply(
                ctx,
                Rewards::generate_message(
                    context.user,
                    data.1,
                    &rewards,
                    *page,
                    Some(EditMode::Add),
                )
                .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct RemoveReward;
#[async_trait::async_trait]
impl ConfigStage for RemoveReward {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "remove_reward")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "rewards"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::RemoveReward { page, rewards } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let mut rewards = rewards.clone();

        if let ComponentInteractionDataKind::RoleSelect { values } = &entry.component()?.data.kind {
            let role = values.first().ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoRoleSelected))
            })?;
            let role = Role::from(*role);

            rewards.retain(|r| r.role != role.as_i64());

            return advance_to(
                Rewards,
                ctx,
                entry,
                (
                    &ConfigInteraction::Xp {
                        stage: XPStage::Rewards {
                            page: *page,
                            rewards: Some(rewards),
                        },
                    },
                    data.1,
                ),
            )
            .await;
        }

        entry
            .reply(
                ctx,
                Rewards::generate_message(
                    context.user,
                    data.1,
                    &rewards,
                    *page,
                    Some(EditMode::Remove),
                )
                .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct SaveRewards;
#[async_trait::async_trait]
impl ConfigStage for SaveRewards {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "save_rewards")
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
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::SaveRewards { rewards } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let mut tx = Bot::global().postgres().begin().await?;
        sqlx::query!(
            "DELETE FROM xp_rewards WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .execute(&mut *tx)
        .await?;

        for reward in rewards {
            sqlx::query!(
                "INSERT INTO xp_rewards (guild_id, role, level) VALUES ($1, $2, $3)",
                context.guild.as_i64(),
                reward.role,
                reward.level,
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        advance_to(RoleMultiplierEnter, ctx, entry, data).await
    }
}

#[derive(Debug)]
pub struct RoleMultiplierEnter;
#[async_trait::async_trait]
impl ConfigStage for RoleMultiplierEnter {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "role_multiplier_enter")
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
            xp_interaction_builder(
                context.user,
                XPStage::RoleMultipliers {
                    page: 0,
                    multipliers: None,
                },
                data.1,
            )
            .build(),
            xp_interaction_builder(context.user, XPStage::ChannelMultiplierEnter, data.1).build(),
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
                            .description("Would you like to configure role boosts?")
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
pub struct RoleMultipliers;

impl RoleMultipliers {
    const MAX_MULTIPLIERS_PER_PAGE: usize = 25;
    fn max_pages(multiplier_count: usize) -> usize {
        ((multiplier_count as f64 / RoleMultipliers::MAX_MULTIPLIERS_PER_PAGE as f64).ceil()
            as usize)
            .max(1)
    }

    fn generate_interactions(
        user: User,
        single_category: bool,
        multipliers: &[RoleMultiplier],
        current_page: usize,
        max_pages: usize,
        edit_mode: Option<EditMode>,
    ) -> (Vec<Interaction>, Vec<CreateActionRow>) {
        let mut components = vec![];
        let mut interactions = vec![];

        if let Some(edit_mode) = edit_mode {
            interactions.append(&mut vec![
                xp_interaction_builder(
                    user,
                    match edit_mode {
                        EditMode::Add => XPStage::AddRoleMultiplier {
                            page: current_page,
                            multipliers: multipliers.to_owned(),
                        },
                        EditMode::Remove => XPStage::RemoveRoleMultiplier {
                            page: current_page,
                            multipliers: multipliers.to_owned(),
                        },
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::RoleMultipliers {
                        page: current_page,
                        multipliers: Some(multipliers.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::RoleMultipliers {
                        page: current_page,
                        multipliers: None,
                    },
                    single_category,
                )
                .build(),
            ]);
            components.append(&mut vec![
                CreateActionRow::SelectMenu(CreateSelectMenu::new(
                    interactions[0].id.to_string(),
                    CreateSelectMenuKind::Role {
                        default_roles: None,
                    },
                )),
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[1].id.to_string())
                        .style(ButtonStyle::Secondary)
                        .label("Cancel edit"),
                    CreateButton::new(interactions[2].id.to_string())
                        .style(ButtonStyle::Danger)
                        .label("Revert"),
                ]),
            ]);
        } else {
            interactions.append(&mut vec![
                xp_interaction_builder(
                    user,
                    XPStage::RoleMultipliers {
                        page: current_page - 1,
                        multipliers: Some(multipliers.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::RoleMultipliers {
                        page: current_page + 1,
                        multipliers: Some(multipliers.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::AddRoleMultiplier {
                        page: current_page,
                        multipliers: multipliers.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::RemoveRoleMultiplier {
                        page: current_page,
                        multipliers: multipliers.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::SaveRoleMultipliers {
                        multipliers: multipliers.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::RoleMultipliers {
                        page: current_page,
                        multipliers: None,
                    },
                    single_category,
                )
                .build(),
            ]);
            components.append(&mut vec![
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[0].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("◀".to_string()))
                        .disabled(current_page == 0),
                    CreateButton::new(interactions[1].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("▶".to_string()))
                        .disabled((current_page + 1) == max_pages),
                    CreateButton::new(interactions[2].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("➕".to_string())),
                    CreateButton::new(interactions[3].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("➖".to_string()))
                        .disabled(multipliers.is_empty()),
                    CreateButton::new(interactions[4].id.to_string())
                        .emoji('✅')
                        .style(ButtonStyle::Success),
                ]),
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[5].id.to_string())
                        .style(ButtonStyle::Danger)
                        .label("Revert"),
                ]),
            ]);
        }

        (interactions, components)
    }

    async fn generate_message(
        user: User,
        single_category: bool,
        multipliers: &Vec<RoleMultiplier>,
        current_page: usize,
        edit_mode: Option<EditMode>,
    ) -> Result<Response, ResponseError> {
        let mut multipliers_by_mult: BTreeMap<i32, Vec<i64>> = BTreeMap::new();
        for multiplier in multipliers {
            let mult_int = ((multiplier.multiplier - 1.0) * 100.0).round() as i32;

            if let std::collections::btree_map::Entry::Vacant(e) =
                multipliers_by_mult.entry(mult_int)
            {
                e.insert(vec![multiplier.role]);
            } else {
                multipliers_by_mult
                    .get_mut(&mult_int)
                    .unwrap()
                    .push(multiplier.role);
            }
        }

        let fields = multipliers_by_mult
            .iter()
            .map(|(mult, roles)| {
                (
                    format!("{mult}% boost"),
                    roles
                        .iter()
                        .map(|role| format!("<@&{role}>"))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    true,
                )
            })
            .collect::<Vec<_>>();

        let start = current_page * RoleMultipliers::MAX_MULTIPLIERS_PER_PAGE;
        let mut end = start + RoleMultipliers::MAX_MULTIPLIERS_PER_PAGE;
        if end > fields.len() {
            end = fields.len();
        }
        let fields = fields[start..end].to_vec();
        let max_pages = RoleMultipliers::max_pages(multipliers_by_mult.len());
        let (interactions, components) = Self::generate_interactions(
            user,
            single_category,
            multipliers,
            current_page,
            max_pages,
            edit_mode,
        );

        Bot::global()
            .interaction_state()
            .register(interactions.clone())
            .await?;

        Ok(Response::new()
            .embed(
                CreateEmbed::new()
                    .title(format!(
                        "Role Boosts - Page {}/{}",
                        current_page + 1,
                        max_pages
                    ))
                    .fields(fields)
                    .color(EMBED_COLOR),
            )
            .components(components))
    }
}

#[async_trait::async_trait]
impl ConfigStage for RoleMultipliers {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "role_multipliers")
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
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::RoleMultipliers { page, multipliers } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let multipliers = match multipliers {
            Some(multipliers) => multipliers,
            None => {
                &sqlx::query_as!(
                    RoleMultiplier,
                    "SELECT role, multiplier FROM xp_role_multipliers WHERE guild_id = $1",
                    context.guild.as_i64()
                )
                .fetch_all(Bot::global().postgres())
                .await?
            }
        };

        entry
            .reply(
                ctx,
                RoleMultipliers::generate_message(context.user, data.1, multipliers, *page, None)
                    .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct AddRoleMultiplier;
#[async_trait::async_trait]
impl ConfigStage for AddRoleMultiplier {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "add_role_multiplier")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "role_multipliers"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::AddRoleMultiplier { page, multipliers } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let mut multipliers = multipliers.clone();

        if let ComponentInteractionDataKind::RoleSelect { values } = &entry.component()?.data.kind {
            let role = values.first().ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoRoleSelected))
            })?;
            let role = Role::from(*role);

            entry
                .modal(
                    ctx,
                    CreateModal::new("role_multiplier_modal", "Add Boost").components(vec![
                        CreateActionRow::InputText(
                            CreateInputText::new(
                                InputTextStyle::Short,
                                "Boost (in %)",
                                "multiplier",
                            )
                            .placeholder("20")
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
                    let Ok(multiplier) =
                        text.value.as_ref().unwrap().parse::<i32>().map_err(|_| {
                            ResponseError::Execution(ExecutionError::Input(
                                InputError::InvalidStrikeCount,
                            ))
                        })
                    else {
                        return Err(ResponseError::Execution(ExecutionError::Input(
                            InputError::InvalidNumber,
                        )));
                    };

                    if multiplier <= 0 {
                        return Err(ResponseError::Execution(ExecutionError::Input(
                            InputError::InvalidNumber,
                        )));
                    }

                    multipliers.push(RoleMultiplier {
                        role: role.as_i64(),
                        multiplier: (multiplier as f64 / 100.0) + 1.0,
                    });

                    return advance_to(
                        RoleMultipliers,
                        ctx,
                        entry,
                        (
                            &ConfigInteraction::Xp {
                                stage: XPStage::RoleMultipliers {
                                    page: *page,
                                    multipliers: Some(multipliers.clone()),
                                },
                            },
                            data.1,
                        ),
                    )
                    .await;
                }
                return Err(ResponseError::Execution(ExecutionError::Internal(
                    InternalError::InvalidInteractionType,
                )));
            }

            return Err(ResponseError::Execution(ExecutionError::Input(
                InputError::Timeout {
                    duration: "5 minutes".to_string(),
                },
            )));
        }

        entry
            .reply(
                ctx,
                RoleMultipliers::generate_message(
                    context.user,
                    data.1,
                    &multipliers,
                    *page,
                    Some(EditMode::Add),
                )
                .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct RemoveRoleMultiplier;
#[async_trait::async_trait]
impl ConfigStage for RemoveRoleMultiplier {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "remove_role_multiplier")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "role_multipliers"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::RemoveRoleMultiplier { page, multipliers } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let mut multipliers = multipliers.clone();

        if let ComponentInteractionDataKind::RoleSelect { values } = &entry.component()?.data.kind {
            let role = values.first().ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoRoleSelected))
            })?;
            let role = Role::from(*role);

            multipliers.retain(|r| r.role != role.as_i64());

            return advance_to(
                RoleMultipliers,
                ctx,
                entry,
                (
                    &ConfigInteraction::Xp {
                        stage: XPStage::RoleMultipliers {
                            page: *page,
                            multipliers: Some(multipliers),
                        },
                    },
                    data.1,
                ),
            )
            .await;
        }

        entry
            .reply(
                ctx,
                RoleMultipliers::generate_message(
                    context.user,
                    data.1,
                    &multipliers,
                    *page,
                    Some(EditMode::Remove),
                )
                .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct SaveRoleMultipliers;
#[async_trait::async_trait]
impl ConfigStage for SaveRoleMultipliers {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "save_role_multipliers")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "role_multipliers"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::SaveRoleMultipliers { multipliers } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let mut tx = Bot::global().postgres().begin().await?;
        sqlx::query!(
            "DELETE FROM xp_role_multipliers WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .execute(&mut *tx)
        .await?;

        for m in multipliers {
            sqlx::query!(
                "INSERT INTO xp_role_multipliers (guild_id, role, multiplier) VALUES ($1, $2, $3)",
                context.guild.as_i64(),
                m.role,
                m.multiplier as f32,
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        advance_to(
            ChannelMultiplierEnter,
            ctx,
            entry,
            (
                &ConfigInteraction::Xp {
                    stage: XPStage::ChannelMultipliers {
                        page: 0,
                        multipliers: None,
                    },
                },
                data.1,
            ),
        )
        .await
    }
}

#[derive(Debug)]
pub struct ChannelMultiplierEnter;
#[async_trait::async_trait]
impl ConfigStage for ChannelMultiplierEnter {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "channel_multiplier_enter")
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
            xp_interaction_builder(
                context.user,
                XPStage::ChannelMultipliers {
                    page: 0,
                    multipliers: None,
                },
                data.1,
            )
            .build(),
            xp_interaction_builder(context.user, XPStage::RoleBlacklistEnter, data.1).build(),
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
                            .description("Would you like to configure channel boosts?")
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
pub struct ChannelMultipliers;

impl ChannelMultipliers {
    const MAX_MULTIPLIERS_PER_PAGE: usize = 25;
    fn max_pages(multiplier_count: usize) -> usize {
        ((multiplier_count as f64 / ChannelMultipliers::MAX_MULTIPLIERS_PER_PAGE as f64).ceil()
            as usize)
            .max(1)
    }

    fn generate_interactions(
        user: User,
        single_category: bool,
        multipliers: &[ChannelMultiplier],
        current_page: usize,
        max_pages: usize,
        edit_mode: Option<EditMode>,
    ) -> (Vec<Interaction>, Vec<CreateActionRow>) {
        let mut components = vec![];
        let mut interactions = vec![];

        if let Some(edit_mode) = edit_mode {
            interactions.append(&mut vec![
                xp_interaction_builder(
                    user,
                    match edit_mode {
                        EditMode::Add => XPStage::AddChannelMultiplier {
                            page: current_page,
                            multipliers: multipliers.to_owned(),
                        },
                        EditMode::Remove => XPStage::RemoveChannelMultiplier {
                            page: current_page,
                            multipliers: multipliers.to_owned(),
                        },
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::ChannelMultipliers {
                        page: current_page,
                        multipliers: Some(multipliers.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::ChannelMultipliers {
                        page: current_page,
                        multipliers: None,
                    },
                    single_category,
                )
                .build(),
            ]);
            components.append(&mut vec![
                CreateActionRow::SelectMenu(CreateSelectMenu::new(
                    interactions[0].id.to_string(),
                    CreateSelectMenuKind::Channel {
                        channel_types: Some(vec![
                            ChannelType::Text,
                            ChannelType::Forum,
                            ChannelType::PublicThread,
                            ChannelType::PrivateThread,
                        ]),
                        default_channels: None,
                    },
                )),
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[1].id.to_string())
                        .style(ButtonStyle::Secondary)
                        .label("Cancel edit"),
                    CreateButton::new(interactions[2].id.to_string())
                        .style(ButtonStyle::Danger)
                        .label("Revert"),
                ]),
            ]);
        } else {
            interactions.append(&mut vec![
                xp_interaction_builder(
                    user,
                    XPStage::ChannelMultipliers {
                        page: current_page - 1,
                        multipliers: Some(multipliers.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::ChannelMultipliers {
                        page: current_page + 1,
                        multipliers: Some(multipliers.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::AddChannelMultiplier {
                        page: current_page,
                        multipliers: multipliers.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::RemoveChannelMultiplier {
                        page: current_page,
                        multipliers: multipliers.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::SaveChannelMultipliers {
                        multipliers: multipliers.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::ChannelMultipliers {
                        page: current_page,
                        multipliers: None,
                    },
                    single_category,
                )
                .build(),
            ]);
            components.append(&mut vec![
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[0].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("◀".to_string()))
                        .disabled(current_page == 0),
                    CreateButton::new(interactions[1].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("▶".to_string()))
                        .disabled((current_page + 1) == max_pages),
                    CreateButton::new(interactions[2].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("➕".to_string())),
                    CreateButton::new(interactions[3].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("➖".to_string()))
                        .disabled(multipliers.is_empty()),
                    CreateButton::new(interactions[4].id.to_string())
                        .emoji('✅')
                        .style(ButtonStyle::Success),
                ]),
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[5].id.to_string())
                        .style(ButtonStyle::Danger)
                        .label("Revert"),
                ]),
            ]);
        }

        (interactions, components)
    }

    async fn generate_message(
        user: User,
        single_category: bool,
        multipliers: &Vec<ChannelMultiplier>,
        current_page: usize,
        edit_mode: Option<EditMode>,
    ) -> Result<Response, ResponseError> {
        let mut multipliers_by_mult: BTreeMap<i32, Vec<i64>> = BTreeMap::new();
        for multiplier in multipliers {
            let mult_int = ((multiplier.multiplier - 1.0) * 100.0).round() as i32;

            if let std::collections::btree_map::Entry::Vacant(e) =
                multipliers_by_mult.entry(mult_int)
            {
                e.insert(vec![multiplier.channel]);
            } else {
                multipliers_by_mult
                    .get_mut(&mult_int)
                    .unwrap()
                    .push(multiplier.channel);
            }
        }

        let fields = multipliers_by_mult
            .iter()
            .map(|(mult, channels)| {
                (
                    format!("{mult}% boost"),
                    channels
                        .iter()
                        .map(|ch| format!("<#{ch}>"))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    true,
                )
            })
            .collect::<Vec<_>>();

        let start = current_page * ChannelMultipliers::MAX_MULTIPLIERS_PER_PAGE;
        let mut end = start + ChannelMultipliers::MAX_MULTIPLIERS_PER_PAGE;
        if end > fields.len() {
            end = fields.len();
        }
        let fields = fields[start..end].to_vec();
        let max_pages = ChannelMultipliers::max_pages(multipliers_by_mult.len());
        let (interactions, components) = Self::generate_interactions(
            user,
            single_category,
            multipliers,
            current_page,
            max_pages,
            edit_mode,
        );

        Bot::global()
            .interaction_state()
            .register(interactions.clone())
            .await?;

        Ok(Response::new()
            .embed(
                CreateEmbed::new()
                    .title(format!(
                        "Channel Boosts - Page {}/{}",
                        current_page + 1,
                        max_pages
                    ))
                    .fields(fields)
                    .color(EMBED_COLOR),
            )
            .components(components))
    }
}

#[async_trait::async_trait]
impl ConfigStage for ChannelMultipliers {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "channel_multipliers")
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
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::ChannelMultipliers { page, multipliers } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let multipliers =
            match multipliers {
                Some(multipliers) => multipliers,
                None => &sqlx::query_as!(
                    ChannelMultiplier,
                    "SELECT channel, multiplier FROM xp_channel_multipliers WHERE guild_id = $1",
                    context.guild.as_i64()
                )
                .fetch_all(Bot::global().postgres())
                .await?,
            };

        entry
            .reply(
                ctx,
                ChannelMultipliers::generate_message(
                    context.user,
                    data.1,
                    multipliers,
                    *page,
                    None,
                )
                .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct AddChannelMultiplier;
#[async_trait::async_trait]
impl ConfigStage for AddChannelMultiplier {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "add_channel_multiplier")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "channel_multipliers"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::AddChannelMultiplier { page, multipliers } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let mut multipliers = multipliers.clone();

        if let ComponentInteractionDataKind::ChannelSelect { values } =
            &entry.component()?.data.kind
        {
            let channel = values.first().ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoChannelSelected))
            })?;
            let channel = Channel::from(*channel);

            entry
                .modal(
                    ctx,
                    CreateModal::new("channel_multiplier_modal", "Add Boost").components(vec![
                        CreateActionRow::InputText(
                            CreateInputText::new(
                                InputTextStyle::Short,
                                "Boost (in %)",
                                "multiplier",
                            )
                            .placeholder("20")
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
                    let Ok(multiplier) =
                        text.value.as_ref().unwrap().parse::<i32>().map_err(|_| {
                            ResponseError::Execution(ExecutionError::Input(
                                InputError::InvalidStrikeCount,
                            ))
                        })
                    else {
                        return Err(ResponseError::Execution(ExecutionError::Input(
                            InputError::InvalidNumber,
                        )));
                    };

                    if multiplier <= 0 {
                        return Err(ResponseError::Execution(ExecutionError::Input(
                            InputError::InvalidNumber,
                        )));
                    }

                    multipliers.push(ChannelMultiplier {
                        channel: channel.as_i64(),
                        multiplier: (multiplier as f64 / 100.0) + 1.0,
                    });

                    return advance_to(
                        ChannelMultipliers,
                        ctx,
                        entry,
                        (
                            &ConfigInteraction::Xp {
                                stage: XPStage::ChannelMultipliers {
                                    page: *page,
                                    multipliers: Some(multipliers.clone()),
                                },
                            },
                            data.1,
                        ),
                    )
                    .await;
                }
                return Err(ResponseError::Execution(ExecutionError::Internal(
                    InternalError::InvalidInteractionType,
                )));
            }

            return Err(ResponseError::Execution(ExecutionError::Input(
                InputError::Timeout {
                    duration: "5 minutes".to_string(),
                },
            )));
        }

        entry
            .reply(
                ctx,
                ChannelMultipliers::generate_message(
                    context.user,
                    data.1,
                    &multipliers,
                    *page,
                    Some(EditMode::Add),
                )
                .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct RemoveChannelMultiplier;
#[async_trait::async_trait]
impl ConfigStage for RemoveChannelMultiplier {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "remove_channel_multiplier")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "channel_multipliers"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::RemoveChannelMultiplier { page, multipliers } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let mut multipliers = multipliers.clone();

        if let ComponentInteractionDataKind::ChannelSelect { values } =
            &entry.component()?.data.kind
        {
            let channel = values.first().ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoChannelSelected))
            })?;
            let channel = Channel::from(*channel);

            multipliers.retain(|r| r.channel != channel.as_i64());

            return advance_to(
                ChannelMultipliers,
                ctx,
                entry,
                (
                    &ConfigInteraction::Xp {
                        stage: XPStage::ChannelMultipliers {
                            page: *page,
                            multipliers: Some(multipliers),
                        },
                    },
                    data.1,
                ),
            )
            .await;
        }

        entry
            .reply(
                ctx,
                ChannelMultipliers::generate_message(
                    context.user,
                    data.1,
                    &multipliers,
                    *page,
                    Some(EditMode::Remove),
                )
                .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct SaveChannelMultipliers;
#[async_trait::async_trait]
impl ConfigStage for SaveChannelMultipliers {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "save_channel_multipliers")
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
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::SaveChannelMultipliers { multipliers } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let mut tx = Bot::global().postgres().begin().await?;
        sqlx::query!(
            "DELETE FROM xp_channel_multipliers WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .execute(&mut *tx)
        .await?;

        for m in multipliers {
            sqlx::query!(
                "INSERT INTO xp_channel_multipliers (guild_id, channel, multiplier) VALUES ($1, $2, $3)",
                context.guild.as_i64(),
                m.channel,
                m.multiplier as f32,
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        advance_to(RoleBlacklistEnter, ctx, entry, data).await
    }
}

#[derive(Debug)]
pub struct RoleBlacklistEnter;
#[async_trait::async_trait]
impl ConfigStage for RoleBlacklistEnter {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "role_blacklist_enter")
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
            xp_interaction_builder(
                context.user,
                XPStage::RoleBlacklists {
                    page: 0,
                    blacklists: None,
                },
                data.1,
            )
            .build(),
            xp_interaction_builder(context.user, XPStage::ChannelBlacklistEnter, data.1).build(),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.to_vec())
            .await?;

        let help_text = "> Blacklisting a role will prevent anyone with that role from earning XP";

        entry
            .reply(
                ctx,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!(
                                "Would you like to configure role blacklists?\n\n{help_text}"
                            ))
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
pub struct RoleBlacklists;

impl RoleBlacklists {
    const MAX_BLACKLISTS_PER_PAGE: usize = 25;

    fn max_pages(blacklisted_roles_count: usize) -> usize {
        ((blacklisted_roles_count as f64 / RoleBlacklists::MAX_BLACKLISTS_PER_PAGE as f64).ceil()
            as usize)
            .max(1)
    }

    fn generate_interactions(
        user: User,
        single_category: bool,
        blacklists: &[i64],
        current_page: usize,
        max_pages: usize,
        edit_mode: Option<EditMode>,
    ) -> (Vec<Interaction>, Vec<CreateActionRow>) {
        let mut components = vec![];
        let mut interactions = vec![];

        if let Some(edit_mode) = edit_mode {
            interactions.append(&mut vec![
                xp_interaction_builder(
                    user,
                    match edit_mode {
                        EditMode::Add => XPStage::AddRoleBlacklist {
                            page: current_page,
                            blacklists: blacklists.to_owned(),
                        },
                        EditMode::Remove => XPStage::RemoveRoleBlacklist {
                            page: current_page,
                            blacklists: blacklists.to_owned(),
                        },
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::RoleBlacklists {
                        page: current_page,
                        blacklists: Some(blacklists.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::RoleBlacklists {
                        page: current_page,
                        blacklists: None,
                    },
                    single_category,
                )
                .build(),
            ]);
            components.append(&mut vec![
                CreateActionRow::SelectMenu(CreateSelectMenu::new(
                    interactions[0].id.to_string(),
                    CreateSelectMenuKind::Role {
                        default_roles: None,
                    },
                )),
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[1].id.to_string())
                        .style(ButtonStyle::Secondary)
                        .label("Cancel edit"),
                    CreateButton::new(interactions[2].id.to_string())
                        .style(ButtonStyle::Danger)
                        .label("Revert"),
                ]),
            ]);
        } else {
            interactions.append(&mut vec![
                xp_interaction_builder(
                    user,
                    XPStage::RoleBlacklists {
                        page: current_page - 1,
                        blacklists: Some(blacklists.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::RoleBlacklists {
                        page: current_page + 1,
                        blacklists: Some(blacklists.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::AddRoleBlacklist {
                        page: current_page,
                        blacklists: blacklists.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::RemoveRoleBlacklist {
                        page: current_page,
                        blacklists: blacklists.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::SaveRoleBlacklist {
                        blacklists: blacklists.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::RoleBlacklists {
                        page: current_page,
                        blacklists: None,
                    },
                    single_category,
                )
                .build(),
            ]);
            components.append(&mut vec![
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[0].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("◀".to_string()))
                        .disabled(current_page == 0),
                    CreateButton::new(interactions[1].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("▶".to_string()))
                        .disabled((current_page + 1) == max_pages),
                    CreateButton::new(interactions[2].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("➕".to_string())),
                    CreateButton::new(interactions[3].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("➖".to_string()))
                        .disabled(blacklists.is_empty()),
                    CreateButton::new(interactions[4].id.to_string())
                        .emoji('✅')
                        .style(ButtonStyle::Success),
                ]),
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[5].id.to_string())
                        .style(ButtonStyle::Danger)
                        .label("Revert"),
                ]),
            ]);
        }

        (interactions, components)
    }

    async fn generate_message(
        user: User,
        single_category: bool,
        blacklists: &[i64],
        current_page: usize,
        edit_mode: Option<EditMode>,
    ) -> Result<Response, ResponseError> {
        let start = current_page * RoleBlacklists::MAX_BLACKLISTS_PER_PAGE;
        let mut end = start + RoleBlacklists::MAX_BLACKLISTS_PER_PAGE;
        if end > blacklists.len() {
            end = blacklists.len();
        }
        let blacklists_to_render = blacklists[start..end].to_vec();
        let max_pages = RoleBlacklists::max_pages(blacklists.len());
        let (interactions, components) = Self::generate_interactions(
            user,
            single_category,
            blacklists,
            current_page,
            max_pages,
            edit_mode,
        );

        Bot::global()
            .interaction_state()
            .register(interactions.clone())
            .await?;

        Ok(Response::new()
            .embed(
                CreateEmbed::new()
                    .title(format!(
                        "Channel Boosts - Page {}/{}",
                        current_page + 1,
                        max_pages
                    ))
                    .description(format!(
                        "The following roles are currently blacklisted:\n\n{}",
                        blacklists_to_render
                            .iter()
                            .map(|role| format!("- <@&{role}>"))
                            .collect::<Vec<_>>()
                            .join("\n")
                    ))
                    .color(EMBED_COLOR),
            )
            .components(components))
    }
}

#[async_trait::async_trait]
impl ConfigStage for RoleBlacklists {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "role_blacklists")
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
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::RoleBlacklists { page, blacklists } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let blacklists = match blacklists {
            Some(blacklists) => blacklists,
            None => &sqlx::query!(
                "SELECT role FROM xp_role_blacklists WHERE guild_id = $1",
                context.guild.as_i64()
            )
            .fetch_all(Bot::global().postgres())
            .await?
            .into_iter()
            .map(|row| row.role)
            .collect::<Vec<_>>(),
        };

        entry
            .reply(
                ctx,
                RoleBlacklists::generate_message(context.user, data.1, blacklists, *page, None)
                    .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct AddRoleBlacklist;
#[async_trait::async_trait]
impl ConfigStage for AddRoleBlacklist {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "add_role_blacklist")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "role_blacklists"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::AddRoleBlacklist { page, blacklists } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let mut blacklists = blacklists.clone();

        if let ComponentInteractionDataKind::RoleSelect { values } = &entry.component()?.data.kind {
            let role = values.first().ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoRoleSelected))
            })?;
            let role = Role::from(*role);

            blacklists.push(role.as_i64());
            return advance_to(
                RoleBlacklists,
                ctx,
                entry,
                (
                    &ConfigInteraction::Xp {
                        stage: XPStage::RoleBlacklists {
                            page: *page,
                            blacklists: Some(blacklists),
                        },
                    },
                    data.1,
                ),
            )
            .await;
        }

        entry
            .reply(
                ctx,
                RoleBlacklists::generate_message(
                    context.user,
                    data.1,
                    &blacklists,
                    *page,
                    Some(EditMode::Add),
                )
                .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct RemoveRoleBlacklist;
#[async_trait::async_trait]
impl ConfigStage for RemoveRoleBlacklist {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "remove_role_blacklist")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "role_blacklists"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::RemoveRoleBlacklist { page, blacklists } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let mut blacklists = blacklists.clone();

        if let ComponentInteractionDataKind::RoleSelect { values } = &entry.component()?.data.kind {
            let role = values.first().ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoRoleSelected))
            })?;
            let role = Role::from(*role);

            blacklists.retain(|r| r != &role.as_i64());
            return advance_to(
                RoleBlacklists,
                ctx,
                entry,
                (
                    &ConfigInteraction::Xp {
                        stage: XPStage::RoleBlacklists {
                            page: *page,
                            blacklists: Some(blacklists),
                        },
                    },
                    data.1,
                ),
            )
            .await;
        }

        entry
            .reply(
                ctx,
                RoleBlacklists::generate_message(
                    context.user,
                    data.1,
                    &blacklists,
                    *page,
                    Some(EditMode::Remove),
                )
                .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct SaveRoleBlacklist;
#[async_trait::async_trait]
impl ConfigStage for SaveRoleBlacklist {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "save_role_blacklist")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "role_blacklists"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::SaveRoleBlacklist { blacklists } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let mut tx = Bot::global().postgres().begin().await?;
        sqlx::query!(
            "DELETE FROM xp_role_blacklists WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .execute(&mut *tx)
        .await?;

        for role in blacklists {
            sqlx::query!(
                "INSERT INTO xp_role_blacklists (guild_id, role) VALUES ($1, $2)",
                context.guild.as_i64(),
                role
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        advance_to(ChannelBlacklistEnter, ctx, entry, data).await
    }
}

#[derive(Debug)]
pub struct ChannelBlacklistEnter;
#[async_trait::async_trait]
impl ConfigStage for ChannelBlacklistEnter {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "channel_blacklist_enter")
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
            xp_interaction_builder(
                context.user,
                XPStage::ChannelBlacklists {
                    page: 0,
                    blacklists: None,
                },
                data.1,
            )
            .build(),
            if data.1 {
                interaction_builder(context.user, ConfigInteraction::Complete, data.1).build()
            } else {
                interaction_builder(
                    context.user,
                    ConfigInteraction::Boards {
                        stage: BoardsStage::Enter,
                    },
                    data.1,
                )
                .build()
            },
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.to_vec())
            .await?;

        let help_text =
            "> Blacklisting a channel will prevent anyone speaking in that channel from earning XP";

        entry
            .reply(
                ctx,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!(
                                "Would you like to configure channel blacklists?\n\n{help_text}"
                            ))
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
pub struct ChannelBlacklists;

impl ChannelBlacklists {
    const MAX_BLACKLISTS_PER_PAGE: usize = 25;

    fn max_pages(blacklisted_roles_count: usize) -> usize {
        ((blacklisted_roles_count as f64 / RoleBlacklists::MAX_BLACKLISTS_PER_PAGE as f64).ceil()
            as usize)
            .max(1)
    }

    fn generate_interactions(
        user: User,
        single_category: bool,
        blacklists: &[i64],
        current_page: usize,
        max_pages: usize,
        edit_mode: Option<EditMode>,
    ) -> (Vec<Interaction>, Vec<CreateActionRow>) {
        let mut components = vec![];
        let mut interactions = vec![];

        if let Some(edit_mode) = edit_mode {
            interactions.append(&mut vec![
                xp_interaction_builder(
                    user,
                    match edit_mode {
                        EditMode::Add => XPStage::AddChannelBlacklist {
                            page: current_page,
                            blacklists: blacklists.to_owned(),
                        },
                        EditMode::Remove => XPStage::RemoveChannelBlacklist {
                            page: current_page,
                            blacklists: blacklists.to_owned(),
                        },
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::ChannelBlacklists {
                        page: current_page,
                        blacklists: Some(blacklists.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::ChannelBlacklists {
                        page: current_page,
                        blacklists: None,
                    },
                    single_category,
                )
                .build(),
            ]);
            components.append(&mut vec![
                CreateActionRow::SelectMenu(CreateSelectMenu::new(
                    interactions[0].id.to_string(),
                    CreateSelectMenuKind::Channel {
                        channel_types: None,
                        default_channels: None,
                    },
                )),
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[1].id.to_string())
                        .style(ButtonStyle::Secondary)
                        .label("Cancel edit"),
                    CreateButton::new(interactions[2].id.to_string())
                        .style(ButtonStyle::Danger)
                        .label("Revert"),
                ]),
            ]);
        } else {
            interactions.append(&mut vec![
                xp_interaction_builder(
                    user,
                    XPStage::ChannelBlacklists {
                        page: current_page - 1,
                        blacklists: Some(blacklists.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::ChannelBlacklists {
                        page: current_page + 1,
                        blacklists: Some(blacklists.to_owned()),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::AddChannelBlacklist {
                        page: current_page,
                        blacklists: blacklists.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::RemoveChannelBlacklist {
                        page: current_page,
                        blacklists: blacklists.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::SaveChannelBlacklist {
                        blacklists: blacklists.to_owned(),
                    },
                    single_category,
                )
                .build(),
                xp_interaction_builder(
                    user,
                    XPStage::ChannelBlacklists {
                        page: current_page,
                        blacklists: None,
                    },
                    single_category,
                )
                .build(),
            ]);
            components.append(&mut vec![
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[0].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("◀".to_string()))
                        .disabled(current_page == 0),
                    CreateButton::new(interactions[1].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("▶".to_string()))
                        .disabled((current_page + 1) == max_pages),
                    CreateButton::new(interactions[2].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("➕".to_string())),
                    CreateButton::new(interactions[3].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("➖".to_string()))
                        .disabled(blacklists.is_empty()),
                    CreateButton::new(interactions[4].id.to_string())
                        .emoji('✅')
                        .style(ButtonStyle::Success),
                ]),
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[5].id.to_string())
                        .style(ButtonStyle::Danger)
                        .label("Revert"),
                ]),
            ]);
        }

        (interactions, components)
    }

    async fn generate_message(
        user: User,
        single_category: bool,
        blacklists: &[i64],
        current_page: usize,
        edit_mode: Option<EditMode>,
    ) -> Result<Response, ResponseError> {
        let start = current_page * ChannelBlacklists::MAX_BLACKLISTS_PER_PAGE;
        let mut end = start + ChannelBlacklists::MAX_BLACKLISTS_PER_PAGE;
        if end > blacklists.len() {
            end = blacklists.len();
        }
        let blacklists_to_render = blacklists[start..end].to_vec();
        let max_pages = ChannelBlacklists::max_pages(blacklists.len());
        let (interactions, components) = Self::generate_interactions(
            user,
            single_category,
            blacklists,
            current_page,
            max_pages,
            edit_mode,
        );

        Bot::global()
            .interaction_state()
            .register(interactions.clone())
            .await?;

        Ok(Response::new()
            .embed(
                CreateEmbed::new()
                    .title(format!(
                        "Channel Boosts - Page {}/{}",
                        current_page + 1,
                        max_pages
                    ))
                    .description(format!(
                        "The following channels are currently blacklisted:\n\n{}",
                        blacklists_to_render
                            .iter()
                            .map(|channel| format!("- <#{channel}>"))
                            .collect::<Vec<_>>()
                            .join("\n")
                    ))
                    .color(EMBED_COLOR),
            )
            .components(components))
    }
}

#[async_trait::async_trait]
impl ConfigStage for ChannelBlacklists {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "channel_blacklists")
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
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::ChannelBlacklists { page, blacklists } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let blacklists = match blacklists {
            Some(blacklists) => blacklists,
            None => &sqlx::query!(
                "SELECT channel FROM xp_channel_blacklists WHERE guild_id = $1",
                context.guild.as_i64()
            )
            .fetch_all(Bot::global().postgres())
            .await?
            .into_iter()
            .map(|row| row.channel)
            .collect::<Vec<_>>(),
        };

        entry
            .reply(
                ctx,
                ChannelBlacklists::generate_message(context.user, data.1, blacklists, *page, None)
                    .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct AddChannelBlacklist;
#[async_trait::async_trait]
impl ConfigStage for AddChannelBlacklist {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "add_channel_blacklist")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "channel_blacklists"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::AddChannelBlacklist { page, blacklists } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let mut blacklists = blacklists.clone();

        if let ComponentInteractionDataKind::ChannelSelect { values } =
            &entry.component()?.data.kind
        {
            let channel = values.first().ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoChannelSelected))
            })?;
            let channel = Channel::from(*channel);

            blacklists.push(channel.as_i64());
            return advance_to(
                ChannelBlacklists,
                ctx,
                entry,
                (
                    &ConfigInteraction::Xp {
                        stage: XPStage::ChannelBlacklists {
                            page: *page,
                            blacklists: Some(blacklists),
                        },
                    },
                    data.1,
                ),
            )
            .await;
        }

        entry
            .reply(
                ctx,
                ChannelBlacklists::generate_message(
                    context.user,
                    data.1,
                    &blacklists,
                    *page,
                    Some(EditMode::Add),
                )
                .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct RemoveChannelBlacklist;
#[async_trait::async_trait]
impl ConfigStage for RemoveChannelBlacklist {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "remove_channel_blacklist")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "channel_blacklists"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::RemoveChannelBlacklist { page, blacklists } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let mut blacklists = blacklists.clone();

        if let ComponentInteractionDataKind::ChannelSelect { values } =
            &entry.component()?.data.kind
        {
            let channel = values.first().ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoChannelSelected))
            })?;
            let channel = Channel::from(*channel);

            blacklists.retain(|c| c != &channel.as_i64());
            return advance_to(
                ChannelBlacklists,
                ctx,
                entry,
                (
                    &ConfigInteraction::Xp {
                        stage: XPStage::ChannelBlacklists {
                            page: *page,
                            blacklists: Some(blacklists),
                        },
                    },
                    data.1,
                ),
            )
            .await;
        }

        entry
            .reply(
                ctx,
                RoleBlacklists::generate_message(
                    context.user,
                    data.1,
                    &blacklists,
                    *page,
                    Some(EditMode::Remove),
                )
                .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct SaveChannelBlacklist;
#[async_trait::async_trait]
impl ConfigStage for SaveChannelBlacklist {
    fn key(&self) -> (&'static str, &'static str) {
        ("xp", "save_channel_blacklist")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("xp", "channel_blacklists"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Xp { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let XPStage::SaveChannelBlacklist { blacklists } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let mut tx = Bot::global().postgres().begin().await?;
        sqlx::query!(
            "DELETE FROM xp_channel_blacklists WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .execute(&mut *tx)
        .await?;

        for channel in blacklists {
            sqlx::query!(
                "INSERT INTO xp_channel_blacklists (guild_id, channel) VALUES ($1, $2)",
                context.guild.as_i64(),
                channel
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        if data.1 {
            advance_to(Complete, ctx, entry, data).await
        } else {
            advance_to(BoardsEnter, ctx, entry, data).await
        }
    }
}
