use serenity::all::{
    ActionRowComponent, ButtonStyle, ChannelType, CommandInteraction, ComponentInteractionDataKind,
    CreateActionRow, CreateButton, CreateEmbed, CreateInputText, CreateInteractionResponse,
    CreateModal, CreateSelectMenu, CreateSelectMenuKind, InputTextStyle,
};

use crate::{
    commands::config::{ConfigError, ConfigStage, EMBED_COLOR},
    models::{
        command::{CommandContext, CommandContextReply},
        handler::Handler,
        response::{Response, ResponseError},
    },
};

use super::XP_TITLE;

pub struct LevelUpEnable;
#[async_trait::async_trait]
impl ConfigStage for LevelUpEnable {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let level_up_configuration = if let Some(level_up_configuration) = sqlx::query!(
            "SELECT enabled FROM xp_level_up_messages WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_optional(&handler.main_database)
        .await?
        {
            level_up_configuration.enabled
        } else {
            sqlx::query!(
                "INSERT INTO xp_level_up_messages (guild_id) VALUES ($1)",
                ctx.guild.id.get() as i64
            )
            .execute(&handler.main_database)
            .await?;
            return Ok(Some(0));
        };
        let level_up_configuration_string = if level_up_configuration {
            "**Enabled**"
        } else {
            "**Disabled**"
        };

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(if level_up_configuration {
                                "Would you like to configure or disable level up messages?".to_string()
                            } else {
                                format!("Would you like to enable level up messages?\n\nYour current setting is: {level_up_configuration_string}")
                            })
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("enable")
                            .label(if level_up_configuration {
                                "Configure"
                            } else {
                                "Enable"
                            })
                            .style(ButtonStyle::Success),
                        CreateButton::new("disable")
                            .label("Disable")
                            .style(ButtonStyle::Danger),
                        CreateButton::new("skip")
                            .label("Skip")
                            .style(ButtonStyle::Secondary),
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));

        if let Some(interaction) = collector.await {
            match interaction.data.custom_id.as_str() {
                "skip" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;
                    return Ok(Some(4));
                }
                "enable" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

                    sqlx::query!(
                        "UPDATE xp_level_up_messages SET enabled = true WHERE guild_id = $1",
                        ctx.guild.id.get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                "disable" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

                    sqlx::query!(
                        "UPDATE xp_level_up_messages SET enabled = false WHERE guild_id = $1",
                        ctx.guild.id.get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
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

pub struct LevelUpDm;
#[async_trait::async_trait]
impl ConfigStage for LevelUpDm {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let level_up_configuration = sqlx::query!(
            "SELECT dm_message FROM xp_level_up_messages WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?
        .dm_message;
        let level_up_configuration_string = if level_up_configuration {
            "**DMs**"
        } else {
            "**Channel**"
        };

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!("Would you like to send level up messages in DMs or in a channel?\n\nYour current setting is: {level_up_configuration_string}"))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("enable")
                            .label("DMs")
                            .style(ButtonStyle::Primary),
                        CreateButton::new("disable")
                            .label("Channel")
                            .style(ButtonStyle::Primary),
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));

        if let Some(interaction) = collector.await {
            match interaction.data.custom_id.as_str() {
                "enable" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

                    sqlx::query!(
                        "UPDATE xp_level_up_messages SET dm_message = true, channel = NULL WHERE guild_id = $1",
                        ctx.guild.id.get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(Some(2));
                }
                "disable" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

                    sqlx::query!(
                        "UPDATE xp_level_up_messages SET dm_message = false WHERE guild_id = $1",
                        ctx.guild.id.get() as i64
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

pub struct LevelUpChannel;
#[async_trait::async_trait]
impl ConfigStage for LevelUpChannel {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let level_up_configuration = sqlx::query!(
            "SELECT channel FROM xp_level_up_messages WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?
        .channel;
        let level_up_configuration_string = match level_up_configuration {
            Some(channel) => format!("Messages will be sent to <#{channel}>"),
            None => "Messages will be sent to the channel spoken in".to_string(),
        };

        let help_text = r#"> Level up messages can be sent in one of two ways:
> - To a specific channel - this can be done by selecting a channel from the dropdown
> - To the channel where the user sent their message - this can be done by clicking the "Channel spoken in" button"#;

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!("Where would you like to send level up messages?\n\n{help_text}\n\nYour current setting is: **{level_up_configuration_string}**"))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![
                        CreateActionRow::SelectMenu(CreateSelectMenu::new(
                            "level_up_channel",
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
                        CreateActionRow::Buttons(vec![CreateButton::new("user")
                            .label("Channel spoken in")
                            .style(ButtonStyle::Primary)]),
                    ]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));

        if let Some(interaction) = collector.await {
            match interaction.data.custom_id.as_str() {
                "user" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

                    sqlx::query!(
                        "UPDATE xp_level_up_messages SET channel = NULL WHERE guild_id = $1",
                        ctx.guild.id.get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                "level_up_channel" => {
                    if let ComponentInteractionDataKind::ChannelSelect { values } =
                        &interaction.data.kind
                    {
                        interaction
                            .create_response(
                                &ctx.ctx.http,
                                serenity::builder::CreateInteractionResponse::Acknowledge,
                            )
                            .await?;

                        let channel = values.first().unwrap();
                        sqlx::query!(
                            "UPDATE xp_level_up_messages SET channel = $1 WHERE guild_id = $2",
                            channel.get() as i64,
                            ctx.guild.id.get() as i64
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

pub struct LevelUpMessage;
#[async_trait::async_trait]
impl ConfigStage for LevelUpMessage {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let level_up_configuration = sqlx::query!(
            "SELECT message FROM xp_level_up_messages WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?
        .message;
        let level_up_configuration_string = match &level_up_configuration {
            Some(message) => {
                format!("\n```{message}```")
            }
            None => "\n`No message`".to_string(),
        };

        let mut buttons = vec![CreateButton::new("change")
            .label("Change")
            .style(ButtonStyle::Success)];
        if level_up_configuration.is_some() {
            buttons.push(
                CreateButton::new("skip")
                    .label("Skip")
                    .style(ButtonStyle::Secondary),
            );
        }

        let help_text = format!(
            r#"> The following placeholders can be placed in your message:
> {} - This will become the user's name (e.g. {})
> {} - This will mention the user (e.g. <@{}>)
> {} - This will become the user's level (e.g. 100)"#,
            "{user.name}",
            cmd.user.name,
            "{user.mention}",
            cmd.user.id.get(),
            "{user.level}",
        );

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!("What would you like your level up messages to say?\n\n{help_text}\n\nYour current setting is: **{level_up_configuration_string}**"))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(buttons)]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));

        if let Some(interaction) = collector.await {
            match interaction.data.custom_id.as_str() {
                "change" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            CreateInteractionResponse::Modal(
                                CreateModal::new("level_up_message_modal", "Level Up Message")
                                    .components(vec![CreateActionRow::InputText(
                                        CreateInputText::new(
                                            InputTextStyle::Paragraph,
                                            "Message",
                                            "message",
                                        )
                                        .placeholder("Recommended: A message that mentions the user and their level")
                                        .required(true),
                                    )]),
                            ),
                        )
                        .await?;

                    let modal_collector = message
                        .await_modal_interaction(&ctx.ctx)
                        .author_id(cmd.user.id)
                        .timeout(std::time::Duration::new(60, 0));

                    if let Some(interaction) = modal_collector.await {
                        interaction
                            .create_response(
                                &ctx.ctx.http,
                                serenity::builder::CreateInteractionResponse::Acknowledge,
                            )
                            .await?;

                        if let ActionRowComponent::InputText(text) =
                            &interaction.data.components[0].components[0]
                        {
                            let Ok(message) = text.value.as_ref().unwrap().parse::<String>() else {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid message",
                                        Some("Please enter a valid message.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            };
                            if message.is_empty() {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid message",
                                        Some("Please enter a valid message.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            }

                            sqlx::query!(
                                "UPDATE xp_level_up_messages SET message = $2 WHERE guild_id = $1",
                                ctx.guild.id.get() as i64,
                                message
                            )
                            .execute(&handler.main_database)
                            .await?;
                        }
                        return Ok(None);
                    }
                }
                "skip" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
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
