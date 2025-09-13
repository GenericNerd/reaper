use serenity::all::{
    ButtonStyle, ChannelType, ComponentInteractionDataKind, CreateActionRow, CreateButton,
    CreateEmbed, CreateSelectMenu, CreateSelectMenuKind,
};

use crate::{
    components::config::{
        Complete, ConfigEntry, ConfigStage, EMBED_COLOR, advance_to, interaction_builder,
        xp::XPEnter,
    },
    models::{
        bot::Bot,
        channel::Channel,
        context::Context,
        interactions::{
            Interaction, InteractionBuilder,
            config::{ConfigInteraction, LogCategory, LoggingStage, XPStage},
        },
        response::{
            ExecutionError, InputError, InternalError, Response, ResponseError, ResponseResult,
        },
        user::User,
    },
};

const LOGGING_TITLE: &str = "Configuration - Logging";

fn logging_interaction_builder(
    user: User,
    stage: LoggingStage,
    single_category: bool,
) -> InteractionBuilder {
    interaction_builder(user, ConfigInteraction::Logging { stage }, single_category)
}

#[derive(Debug)]
pub struct LoggingEnter;
#[async_trait::async_trait]
impl ConfigStage for LoggingEnter {
    fn key(&self) -> (&'static str, &'static str) {
        ("logging", "enter")
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
            logging_interaction_builder(
                context.user,
                LoggingStage::Categories {
                    actions: None,
                    messages: None,
                    voice: None,
                },
                data.1,
            )
            .build(),
            interaction_builder(
                context.user,
                ConfigInteraction::XP {
                    stage: XPStage::Enter,
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
                            .title(LOGGING_TITLE)
                            .description("Would you like to configure logging?")
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

#[derive(Clone, Copy)]
struct LoggingSettings {
    log_actions: bool,
    log_messages: bool,
    log_voice: bool,
}

#[derive(Debug)]
pub struct Categories;

impl Categories {
    fn interactions(
        user: User,
        settings: LoggingSettings,
        single_category: bool,
    ) -> Vec<Interaction> {
        [
            logging_interaction_builder(
                user,
                LoggingStage::Categories {
                    actions: Some(!settings.log_actions),
                    messages: Some(settings.log_messages),
                    voice: Some(settings.log_voice),
                },
                single_category,
            )
            .build(),
            logging_interaction_builder(
                user,
                LoggingStage::Categories {
                    actions: Some(settings.log_actions),
                    messages: Some(!settings.log_messages),
                    voice: Some(settings.log_voice),
                },
                single_category,
            )
            .build(),
            logging_interaction_builder(
                user,
                LoggingStage::Categories {
                    actions: Some(settings.log_actions),
                    messages: Some(settings.log_messages),
                    voice: Some(!settings.log_voice),
                },
                single_category,
            )
            .build(),
            logging_interaction_builder(
                user,
                LoggingStage::SubmitCategories {
                    actions: settings.log_actions,
                    messages: settings.log_messages,
                    voice: settings.log_voice,
                },
                single_category,
            )
            .build(),
            logging_interaction_builder(
                user,
                LoggingStage::Categories {
                    actions: None,
                    messages: None,
                    voice: None,
                },
                single_category,
            )
            .build(),
        ]
        .to_vec()
    }

    async fn create_response(
        &self,
        user: User,
        settings: LoggingSettings,
        single_category: bool,
    ) -> Result<Response, ResponseError> {
        let help_text = r"You can configure what Reaper should log using the buttons below.

> Reaper can log important events in your server to help staff review activity and enforce rules.
> 
> You can enable or disable each type of logging independently:
> - **Actions Logging** → Records moderation actions such as strikes, mutes, kicks, and bans.
> - **Message Logging** → Records message edits, deletions, and bulk deletions.
> - **Voice Logging** → Records when members join, leave or move between voice channels.
> 
> Toggle each category below to enable or disable it.";

        let interactions = Categories::interactions(user, settings, single_category);
        let components = vec![
            CreateActionRow::Buttons(vec![
                CreateButton::new(interactions[0].id.to_string())
                    .style(ButtonStyle::Secondary)
                    .label("Actions")
                    .emoji(if settings.log_actions { '❌' } else { '✅' }),
                CreateButton::new(interactions[1].id.to_string())
                    .style(ButtonStyle::Secondary)
                    .label("Messages")
                    .emoji(if settings.log_messages { '❌' } else { '✅' }),
                CreateButton::new(interactions[2].id.to_string())
                    .style(ButtonStyle::Secondary)
                    .label("Voice")
                    .emoji(if settings.log_voice { '❌' } else { '✅' }),
            ]),
            CreateActionRow::Buttons(vec![
                CreateButton::new(interactions[3].id.to_string())
                    .style(ButtonStyle::Success)
                    .label("Submit"),
                CreateButton::new(interactions[4].id.to_string())
                    .style(ButtonStyle::Danger)
                    .label("Revert"),
            ]),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions)
            .await?;

        Ok(Response::new()
            .embed(
                CreateEmbed::new()
                    .title(LOGGING_TITLE)
                    .description(help_text)
                    .color(EMBED_COLOR)
                    .fields(vec![
                        (
                            "Actions Logging",
                            if settings.log_actions {
                                "Enabled ✅"
                            } else {
                                "Disabled ❌"
                            }
                            .to_string(),
                            true,
                        ),
                        (
                            "Message Logging",
                            if settings.log_messages {
                                "Enabled ✅"
                            } else {
                                "Disabled ❌"
                            }
                            .to_string(),
                            true,
                        ),
                        (
                            "Voice Logging",
                            if settings.log_voice {
                                "Enabled ✅"
                            } else {
                                "Disabled ❌"
                            }
                            .to_string(),
                            true,
                        ),
                    ]),
            )
            .components(components))
    }
}

#[async_trait::async_trait]
impl ConfigStage for Categories {
    fn key(&self) -> (&'static str, &'static str) {
        ("logging", "categories")
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
        let ConfigInteraction::Logging { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let LoggingStage::Categories {
            actions,
            messages,
            voice,
        } = stage
        else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let context = ctx.get_populated_context()?;

        sqlx::query!(
            "INSERT INTO logging_configuration (guild_id) VALUES ($1) ON CONFLICT (guild_id) DO NOTHING",
            context.guild.as_i64(),
        )
        .execute(Bot::global().postgres())
        .await?;

        // Fetch all logging settings in one go if any are missing
        let settings = sqlx::query!(
            "SELECT log_actions, log_messages, log_voice \
                FROM logging_configuration \
                WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .fetch_one(Bot::global().postgres())
        .await?;

        let log_actions = actions.unwrap_or(settings.log_actions);
        let log_messages = messages.unwrap_or(settings.log_messages);
        let log_voice = voice.unwrap_or(settings.log_voice);

        let response = self
            .create_response(
                context.user,
                LoggingSettings {
                    log_actions,
                    log_messages,
                    log_voice,
                },
                data.1,
            )
            .await?;

        entry.reply(ctx, response).await.map(|_| ())
    }
}

#[derive(Debug)]
pub struct SubmitCategories;
#[async_trait::async_trait]
impl ConfigStage for SubmitCategories {
    fn key(&self) -> (&'static str, &'static str) {
        ("logging", "submit_categories")
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
        let ConfigInteraction::Logging { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let LoggingStage::SubmitCategories {
            actions,
            messages,
            voice,
        } = stage
        else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let context = ctx.get_populated_context()?;

        sqlx::query!(
            "UPDATE logging_configuration SET log_actions = $1, log_messages = $2, log_voice = $3 WHERE guild_id = $4",
            actions,
            messages,
            voice,
            context.guild.as_i64(),
        )
        .execute(Bot::global().postgres())
        .await?;

        advance_to(OneOrMultiple, ctx, entry, data).await
    }
}

#[derive(Debug)]
pub struct OneOrMultiple;
#[async_trait::async_trait]
impl ConfigStage for OneOrMultiple {
    fn key(&self) -> (&'static str, &'static str) {
        ("logging", "one_or_multiple")
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
            "SELECT log_actions, log_messages, log_voice FROM logging_configuration WHERE guild_id = $1",
            context.guild.as_i64(),
        )
        .fetch_one(Bot::global().postgres())
        .await?;

        if !row.log_actions && !row.log_messages && !row.log_voice {
            if data.1 {
                return advance_to(Complete, ctx, entry, data).await;
            }
            return advance_to(XPEnter, ctx, entry, data).await;
        }

        let interactions = [
            logging_interaction_builder(context.user, LoggingStage::SingleLogChannel, data.1)
                .build(),
            logging_interaction_builder(
                context.user,
                LoggingStage::MultipleLogChannels {
                    category: LogCategory::Actions,
                },
                data.1,
            )
            .build(),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.to_vec())
            .await?;

        let help_text = r"Would you like to log to one channel or to multiple channels?

> Reaper can split up the logging into different channels to make it easier to view and manage.";

        entry
            .reply(
                ctx,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(LOGGING_TITLE)
                            .description(help_text)
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(interactions[0].id.to_string())
                            .label("One channel")
                            .style(ButtonStyle::Primary),
                        CreateButton::new(interactions[1].id.to_string())
                            .label("Multiple channels")
                            .style(ButtonStyle::Primary),
                    ])]),
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct SingleLogChannel;
#[async_trait::async_trait]
impl ConfigStage for SingleLogChannel {
    fn key(&self) -> (&'static str, &'static str) {
        ("logging", "single_log_channel")
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
            logging_interaction_builder(context.user, LoggingStage::SubmitSingleLogChannel, data.1)
                .build(),
            logging_interaction_builder(context.user, LoggingStage::OneOrMultiple, data.1).build(),
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
                            .title(LOGGING_TITLE)
                            .description("Which channel would you like Reaper to send logs to?")
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
                                .label("Cancel")
                                .style(ButtonStyle::Danger),
                        ]),
                    ]),
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct SubmitSingleLogChannel;
#[async_trait::async_trait]
impl ConfigStage for SubmitSingleLogChannel {
    fn key(&self) -> (&'static str, &'static str) {
        ("logging", "submit_single_log_channel")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("logging", "single_log_channel"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;

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
            "UPDATE logging_configuration SET log_channel = $1, log_action_channel = null, log_message_channel = null, log_voice_channel = null WHERE guild_id = $2",
            channel.as_i64(),
            context.guild.as_i64(),
        )
        .execute(Bot::global().postgres())
        .await?;

        if data.1 {
            advance_to(Complete, ctx, entry, data).await
        } else {
            advance_to(XPEnter, ctx, entry, data).await
        }
    }
}

#[derive(Debug)]
pub struct MultipleLogChannels;
#[async_trait::async_trait]
impl ConfigStage for MultipleLogChannels {
    fn key(&self) -> (&'static str, &'static str) {
        ("logging", "multiple_log_channels")
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
        let ConfigInteraction::Logging { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let category = match stage {
            LoggingStage::MultipleLogChannels { category } => category,
            LoggingStage::SubmitMultipleLogChannels { category } => category,
            _ => {
                return Err(ResponseError::Execution(ExecutionError::Internal(
                    InternalError::InvalidInteractionType,
                )));
            }
        };

        let context = ctx.get_populated_context()?;

        let guild_settings = sqlx::query!(
            "SELECT log_actions, log_messages, log_voice FROM logging_configuration WHERE guild_id = $1",
            context.guild.as_i64(),
        )
        .fetch_one(Bot::global().postgres())
        .await?;

        match category {
            LogCategory::Actions => {
                if !guild_settings.log_actions {
                    return advance_to(
                        MultipleLogChannels,
                        ctx,
                        entry,
                        (
                            &ConfigInteraction::Logging {
                                stage: LoggingStage::MultipleLogChannels {
                                    category: LogCategory::Messages,
                                },
                            },
                            data.1,
                        ),
                    )
                    .await;
                }
            }
            LogCategory::Messages => {
                if !guild_settings.log_messages {
                    return advance_to(
                        MultipleLogChannels,
                        ctx,
                        entry,
                        (
                            &ConfigInteraction::Logging {
                                stage: LoggingStage::MultipleLogChannels {
                                    category: LogCategory::Voice,
                                },
                            },
                            data.1,
                        ),
                    )
                    .await;
                }
            }
            LogCategory::Voice => {
                if !guild_settings.log_voice {
                    if data.1 {
                        return advance_to(Complete, ctx, entry, data).await;
                    }
                    return advance_to(XPEnter, ctx, entry, data).await;
                }
            }
        }

        let help_text = format!(
            "Which channel should Reaper send {} logs to?\n\n> {}",
            match category {
                LogCategory::Actions => "moderation action",
                LogCategory::Messages => "messages events",
                LogCategory::Voice => "voice channel events",
            },
            match category {
                LogCategory::Actions =>
                    "**Actions Logging** → Records moderation actions such as strikes, mutes, kicks, and bans.",
                LogCategory::Messages =>
                    "**Message Logging** → Records message edits, deletions, and bulk deletions.",
                LogCategory::Voice =>
                    "**Voice Logging** → Records when members join, leave or move between voice channels.",
            }
        );

        let interactions = vec![
            logging_interaction_builder(
                context.user,
                LoggingStage::SubmitMultipleLogChannels {
                    category: category.clone(),
                },
                data.1,
            )
            .build(),
            match category {
                LogCategory::Actions => logging_interaction_builder(
                    context.user,
                    LoggingStage::MultipleLogChannels {
                        category: LogCategory::Messages,
                    },
                    data.1,
                )
                .build(),
                LogCategory::Messages => logging_interaction_builder(
                    context.user,
                    LoggingStage::MultipleLogChannels {
                        category: LogCategory::Voice,
                    },
                    data.1,
                )
                .build(),
                LogCategory::Voice => {
                    if data.1 {
                        interaction_builder(context.user, ConfigInteraction::Complete, data.1)
                            .build()
                    } else {
                        interaction_builder(
                            context.user,
                            ConfigInteraction::XP {
                                stage: XPStage::Enter,
                            },
                            data.1,
                        )
                        .build()
                    }
                }
            },
            logging_interaction_builder(context.user, LoggingStage::OneOrMultiple, data.1).build(),
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
                            .title(LOGGING_TITLE)
                            .description(help_text)
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
                                .label("Skip")
                                .style(ButtonStyle::Secondary),
                            CreateButton::new(interactions[2].id.to_string())
                                .label("Cancel")
                                .style(ButtonStyle::Danger),
                        ]),
                    ]),
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct SubmitMultipleLogChannels;
#[async_trait::async_trait]
impl ConfigStage for SubmitMultipleLogChannels {
    fn key(&self) -> (&'static str, &'static str) {
        ("logging", "submit_multiple_log_channels")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("logging", "multiple_log_channels"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Logging { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let category = match stage {
            LoggingStage::SubmitMultipleLogChannels { category } => category,
            _ => {
                return Err(ResponseError::Execution(ExecutionError::Internal(
                    InternalError::InvalidInteractionType,
                )));
            }
        };

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

        let mut query = sqlx::QueryBuilder::<sqlx::Postgres>::new(
            "UPDATE logging_configuration SET log_channel = null, ",
        );
        query.push(match category {
            LogCategory::Actions => "log_action_channel",
            LogCategory::Messages => "log_message_channel",
            LogCategory::Voice => "log_voice_channel",
        });
        query.push(" = ");
        query.push_bind(channel.as_i64());
        query.push(" WHERE guild_id = ");
        query.push_bind(context.guild.as_i64());

        query.build().execute(Bot::global().postgres()).await?;

        match category {
            LogCategory::Actions => {
                advance_to(
                    MultipleLogChannels,
                    ctx,
                    entry,
                    (
                        &ConfigInteraction::Logging {
                            stage: LoggingStage::MultipleLogChannels {
                                category: LogCategory::Messages,
                            },
                        },
                        data.1,
                    ),
                )
                .await
            }
            LogCategory::Messages => {
                advance_to(
                    MultipleLogChannels,
                    ctx,
                    entry,
                    (
                        &ConfigInteraction::Logging {
                            stage: LoggingStage::MultipleLogChannels {
                                category: LogCategory::Voice,
                            },
                        },
                        data.1,
                    ),
                )
                .await
            }
            LogCategory::Voice => {
                if data.1 {
                    advance_to(Complete, ctx, entry, data).await
                } else {
                    advance_to(XPEnter, ctx, entry, data).await
                }
            }
        }
    }
}
