use serenity::{
    all::{ButtonStyle, ChannelType, CommandInteraction, ComponentInteractionDataKind},
    builder::{CreateActionRow, CreateButton, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind},
};

use crate::models::{
    command::{CommandContext, CommandContextReply},
    handler::Handler,
    response::{Response, ResponseError},
};

use super::{ConfigError, ConfigStage, EMBED_COLOR};

const LOGGING_TITLE: &str = "Configuration - Logging";

pub struct LoggingChannelMultipleVoice;
#[async_trait::async_trait]
impl ConfigStage for LoggingChannelMultipleVoice {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let is_logging = sqlx::query!(
            "SELECT log_voice FROM logging_configuration WHERE guild_id = $1",
            cmd.guild_id.unwrap().get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?;

        if !is_logging.log_voice {
            return Ok(None);
        }

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(LOGGING_TITLE)
                            .description(
                                "Please enter the channel you would like to log voice actions to.",
                            )
                            .color(EMBED_COLOR),
                    )
                    .components(vec![
                        CreateActionRow::SelectMenu(CreateSelectMenu::new(
                            "log_voice_channel",
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
                        CreateActionRow::Buttons(vec![CreateButton::new("cancel")
                            .label("Cancel")
                            .style(ButtonStyle::Danger)]),
                    ]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));
        if let Some(interaction) = collector.await {
            interaction
                .create_response(
                    &ctx.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;
            match interaction.data.custom_id.as_str() {
                "log_voice_channel" => {
                    if let ComponentInteractionDataKind::ChannelSelect { values } =
                        interaction.data.kind
                    {
                        let channel = values.first().unwrap();
                        sqlx::query!(
                            "UPDATE logging_configuration SET log_voice_channel = $1 WHERE guild_id = $2",
                            channel.get() as i64,
                            cmd.guild_id.unwrap().get() as i64
                        )
                        .execute(&handler.main_database)
                        .await?;
                        return Ok(None);
                    }
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    });
                }
                "cancel" => {
                    return Ok(None);
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct LoggingChannelMultipleMessages;
#[async_trait::async_trait]
impl ConfigStage for LoggingChannelMultipleMessages {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let is_logging = sqlx::query!(
            "SELECT log_messages FROM logging_configuration WHERE guild_id = $1",
            cmd.guild_id.unwrap().get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?;

        if !is_logging.log_messages {
            return Ok(None);
        }

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(LOGGING_TITLE)
                            .description(
                                "Please enter the channel you would like to log messages to.",
                            )
                            .color(EMBED_COLOR),
                    )
                    .components(vec![
                        CreateActionRow::SelectMenu(CreateSelectMenu::new(
                            "log_message_channel",
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
                        CreateActionRow::Buttons(vec![CreateButton::new("cancel")
                            .label("Cancel")
                            .style(ButtonStyle::Danger)]),
                    ]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));
        if let Some(interaction) = collector.await {
            interaction
                .create_response(
                    &ctx.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;
            match interaction.data.custom_id.as_str() {
                "log_message_channel" => {
                    if let ComponentInteractionDataKind::ChannelSelect { values } =
                        interaction.data.kind
                    {
                        let channel = values.first().unwrap();
                        sqlx::query!(
                            "UPDATE logging_configuration SET log_message_channel = $1 WHERE guild_id = $2",
                            channel.get() as i64,
                            cmd.guild_id.unwrap().get() as i64
                        )
                        .execute(&handler.main_database)
                        .await?;
                        return Ok(None);
                    }
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    });
                }
                "cancel" => {
                    return Ok(Some(2));
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct LoggingChannelMultipleActions;
#[async_trait::async_trait]
impl ConfigStage for LoggingChannelMultipleActions {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        sqlx::query!(
            "UPDATE logging_configuration SET log_channel = null WHERE guild_id = $1",
            cmd.guild_id.unwrap().get() as i64
        )
        .execute(&handler.main_database)
        .await?;

        let is_logging = sqlx::query!(
            "SELECT log_actions FROM logging_configuration WHERE guild_id = $1",
            cmd.guild_id.unwrap().get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?;

        if !is_logging.log_actions {
            return Ok(None);
        }

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(LOGGING_TITLE)
                            .description(
                                "Please enter the channel you would like to log actions to.",
                            )
                            .color(EMBED_COLOR),
                    )
                    .components(vec![
                        CreateActionRow::SelectMenu(CreateSelectMenu::new(
                            "log_action_channel",
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
                        CreateActionRow::Buttons(vec![CreateButton::new("cancel")
                            .label("Cancel")
                            .style(ButtonStyle::Danger)]),
                    ]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));
        if let Some(interaction) = collector.await {
            interaction
                .create_response(
                    &ctx.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;
            match interaction.data.custom_id.as_str() {
                "log_action_channel" => {
                    if let ComponentInteractionDataKind::ChannelSelect { values } =
                        interaction.data.kind
                    {
                        let channel = values.first().unwrap();
                        sqlx::query!(
                            "UPDATE logging_configuration SET log_action_channel = $1 WHERE guild_id = $2",
                            channel.get() as i64,
                            cmd.guild_id.unwrap().get() as i64
                        )
                        .execute(&handler.main_database)
                        .await?;
                        return Ok(None);
                    }
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    });
                }
                "cancel" => {
                    return Ok(Some(3));
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct LoggingChannelSingle;
#[async_trait::async_trait]
impl ConfigStage for LoggingChannelSingle {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(LOGGING_TITLE)
                            .description("Please enter the channel you would like to log to.")
                            .color(EMBED_COLOR),
                    )
                    .components(vec![
                        CreateActionRow::SelectMenu(CreateSelectMenu::new(
                            "log_channel",
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
                        CreateActionRow::Buttons(vec![CreateButton::new("cancel")
                            .label("Cancel")
                            .style(ButtonStyle::Danger)]),
                    ]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));
        if let Some(interaction) = collector.await {
            interaction
                .create_response(
                    &ctx.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;
            match interaction.data.custom_id.as_str() {
                "log_channel" => {
                    if let ComponentInteractionDataKind::ChannelSelect { values } =
                        interaction.data.kind
                    {
                        let channel = values.first().unwrap();
                        sqlx::query!(
                            "UPDATE logging_configuration SET log_channel = $1, log_action_channel = null, log_message_channel = null, log_voice_channel = null WHERE guild_id = $2",
                            channel.get() as i64,
                            cmd.guild_id.unwrap().get() as i64
                        )
                        .execute(&handler.main_database)
                        .await?;
                        return Ok(Some(4));
                    }
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    });
                }
                "cancel" => {
                    return Ok(Some(4));
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct LoggingChannelEnter;
#[async_trait::async_trait]
impl ConfigStage for LoggingChannelEnter {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let log_options = sqlx::query!(
            "SELECT log_actions, log_messages, log_voice FROM logging_configuration WHERE guild_id = $1"
            , cmd.guild_id.unwrap().get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?;
        if !log_options.log_actions && !log_options.log_messages && !log_options.log_voice {
            return Ok(Some(4));
        }

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(LOGGING_TITLE)
                            .description(
                                "Would you like to log into one channnel or multiple channels?",
                            )
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("one")
                            .label("One")
                            .style(ButtonStyle::Primary),
                        CreateButton::new("multiple")
                            .label("Multiple")
                            .style(ButtonStyle::Primary),
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));
        if let Some(interaction) = collector.await {
            interaction
                .create_response(
                    &ctx.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;
            match interaction.data.custom_id.as_str() {
                "one" => {
                    return Ok(None);
                }
                "multiple" => {
                    return Ok(Some(2));
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct LoggingLogVoice;
#[async_trait::async_trait]
impl ConfigStage for LoggingLogVoice {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(LOGGING_TITLE)
                            .description("Would you like to log when a user joins, leaves or moves voice channels?")
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("yes")
                            .label("Yes")
                            .style(ButtonStyle::Success),
                        CreateButton::new("no")
                            .label("No")
                            .style(ButtonStyle::Secondary),
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));
        if let Some(interaction) = collector.await {
            interaction
                .create_response(
                    &ctx.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;
            match interaction.data.custom_id.as_str() {
                "yes" => {
                    sqlx::query!(
                        "UPDATE logging_configuration SET log_voice = true WHERE guild_id = $1",
                        cmd.guild_id.unwrap().get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                "no" => {
                    sqlx::query!(
                        "UPDATE logging_configuration SET log_voice = false WHERE guild_id = $1",
                        cmd.guild_id.unwrap().get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct LoggingLogMessages;
#[async_trait::async_trait]
impl ConfigStage for LoggingLogMessages {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(LOGGING_TITLE)
                            .description("Would you like to log message edits and deletions?")
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("yes")
                            .label("Yes")
                            .style(ButtonStyle::Success),
                        CreateButton::new("no")
                            .label("No")
                            .style(ButtonStyle::Secondary),
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));
        if let Some(interaction) = collector.await {
            interaction
                .create_response(
                    &ctx.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;
            match interaction.data.custom_id.as_str() {
                "yes" => {
                    sqlx::query!(
                        "UPDATE logging_configuration SET log_messages = true WHERE guild_id = $1",
                        cmd.guild_id.unwrap().get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                "no" => {
                    sqlx::query!(
                        "UPDATE logging_configuration SET log_messages = false WHERE guild_id = $1",
                        cmd.guild_id.unwrap().get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct LoggingLogActions;
#[async_trait::async_trait]
impl ConfigStage for LoggingLogActions {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(LOGGING_TITLE)
                            .description("Would you like to log actions?")
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("yes")
                            .label("Yes")
                            .style(ButtonStyle::Success),
                        CreateButton::new("no")
                            .label("No")
                            .style(ButtonStyle::Secondary),
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));
        if let Some(interaction) = collector.await {
            interaction
                .create_response(
                    &ctx.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;
            match interaction.data.custom_id.as_str() {
                "yes" => {
                    sqlx::query!(
                        "UPDATE logging_configuration SET log_actions = true WHERE guild_id = $1",
                        cmd.guild_id.unwrap().get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                "no" => {
                    sqlx::query!(
                        "UPDATE logging_configuration SET log_actions = false WHERE guild_id = $1",
                        cmd.guild_id.unwrap().get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct LoggingEnter;
#[async_trait::async_trait]
impl ConfigStage for LoggingEnter {
    async fn execute(
        &self,
        _handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(LOGGING_TITLE)
                            .description("Would you like to configure logging?")
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("yes")
                            .label("Yes")
                            .style(ButtonStyle::Success),
                        CreateButton::new("no")
                            .label("No")
                            .style(ButtonStyle::Secondary),
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));
        if let Some(interaction) = collector.await {
            interaction
                .create_response(
                    &ctx.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;
            match interaction.data.custom_id.as_str() {
                "yes" => {
                    return Ok(None);
                }
                "no" => {
                    return Ok(Some(9));
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}
