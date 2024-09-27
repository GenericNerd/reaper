use serenity::{
    all::{
        ButtonStyle, CommandInteraction, ComponentInteractionDataKind, CreateActionRow,
        CreateButton, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, ReactionType,
    },
    futures::StreamExt,
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

pub struct ChannelBlacklistEnter;
#[async_trait::async_trait]
impl ConfigStage for ChannelBlacklistEnter {
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
                            .title(XP_TITLE)
                            .description("Would you like to configure channel blacklists?\n\n> Blacklisting a channel will prevent anyone with the channel from earning XP.")
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("yes")
                            .label("Yes")
                            .style(ButtonStyle::Success),
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
                    return Ok(Some(2));
                }
                "yes" => {
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

pub struct ChannelBlacklistManage;

impl ChannelBlacklistManage {
    const MAX_BLACKLISTED_CHANNELS_PER_PAGE: usize = 25;

    fn max_pages(blacklisted_channels_count: usize) -> usize {
        ((blacklisted_channels_count as f64
            / ChannelBlacklistManage::MAX_BLACKLISTED_CHANNELS_PER_PAGE as f64)
            .ceil() as usize)
            .max(1)
    }

    fn generate_display_message(
        blacklisted_channels: &[i64],
        current_page: usize,
        editing: bool,
    ) -> Response {
        let start = current_page * ChannelBlacklistManage::MAX_BLACKLISTED_CHANNELS_PER_PAGE;
        let mut end = start + ChannelBlacklistManage::MAX_BLACKLISTED_CHANNELS_PER_PAGE;
        if end > blacklisted_channels.len() {
            end = blacklisted_channels.len();
        }
        let blacklisted_channels_to_render = blacklisted_channels[start..end].to_vec();

        let max_pages = ChannelBlacklistManage::max_pages(blacklisted_channels.len());

        let mut components = vec![];
        if editing {
            components.push(CreateActionRow::SelectMenu(CreateSelectMenu::new(
                "pick_channel",
                CreateSelectMenuKind::Channel {
                    channel_types: None,
                    default_channels: None,
                },
            )));
            components.push(CreateActionRow::Buttons(vec![
                CreateButton::new("cancel")
                    .style(ButtonStyle::Secondary)
                    .label("Cancel edit"),
                CreateButton::new("revert")
                    .style(ButtonStyle::Danger)
                    .label("Revert"),
            ]));
        } else {
            components.push(CreateActionRow::Buttons(vec![
                CreateButton::new("previous")
                    .style(ButtonStyle::Primary)
                    .emoji(ReactionType::Unicode("◀".to_string()))
                    .disabled(current_page == 0),
                CreateButton::new("next")
                    .style(ButtonStyle::Primary)
                    .emoji(ReactionType::Unicode("▶".to_string()))
                    .disabled((current_page + 1) == max_pages),
                CreateButton::new("add")
                    .style(ButtonStyle::Primary)
                    .emoji(ReactionType::Unicode("➕".to_string())),
                CreateButton::new("remove")
                    .style(ButtonStyle::Primary)
                    .emoji(ReactionType::Unicode("➖".to_string()))
                    .disabled(blacklisted_channels.is_empty()),
                CreateButton::new("done")
                    .emoji('✅')
                    .style(ButtonStyle::Success),
            ]));
            components.push(CreateActionRow::Buttons(vec![CreateButton::new("revert")
                .style(ButtonStyle::Danger)
                .label("Revert")]));
        }

        let mut embed = CreateEmbed::new().title(format!(
            "Blacklisted Channels - Page {}/{}",
            current_page + 1,
            max_pages
        ));
        if !blacklisted_channels_to_render.is_empty() {
            embed = embed.description(format!(
                "The following channels are currently blacklisted:\n\n{}",
                blacklisted_channels_to_render
                    .iter()
                    .map(|channel| format!("<#{channel}>"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
        embed = embed.color(EMBED_COLOR);

        Response::new().embed(embed).components(components)
    }

    async fn save_blacklisted_channels(
        blacklisted_channels: &[i64],
        handler: &Handler,
        ctx: &CommandContext,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM xp_channel_blacklists WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .execute(&handler.main_database)
        .await?;
        for blacklisted_channel in blacklisted_channels {
            sqlx::query!(
                "INSERT INTO xp_channel_blacklists (guild_id, channel) VALUES ($1, $2)",
                ctx.guild.id.get() as i64,
                blacklisted_channel
            )
            .execute(&handler.main_database)
            .await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl ConfigStage for ChannelBlacklistManage {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        enum EditingMode {
            Add,
            Remove,
        }

        let mut blacklisted_channels = sqlx::query!(
            "SELECT channel FROM xp_channel_blacklists WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_all(&handler.main_database)
        .await?
        .iter()
        .map(|channel| channel.channel)
        .collect::<Vec<_>>();

        let mut current_page = 0;
        let mut editing_mode = None;

        let message = ctx
            .reply_get_message(
                cmd,
                ChannelBlacklistManage::generate_display_message(
                    &blacklisted_channels,
                    current_page,
                    editing_mode.is_some(),
                ),
            )
            .await?;

        let mut collector = message
            .await_component_interactions(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60 * 10, 0))
            .stream();

        while let Some(interaction) = collector.next().await {
            if interaction.data.custom_id.as_str() != "pick_channel" {
                interaction
                    .create_response(
                        &ctx.ctx.http,
                        serenity::builder::CreateInteractionResponse::Acknowledge,
                    )
                    .await?;
            }

            let max_pages = ChannelBlacklistManage::max_pages(blacklisted_channels.len());

            match interaction.data.custom_id.as_str() {
                "pick_channel" => {
                    let Some(ref edit_mode) = editing_mode else {
                        ChannelBlacklistManage::save_blacklisted_channels(
                            &blacklisted_channels,
                            handler,
                            ctx,
                        )
                        .await?;
                        return Err(ConfigError {
                            error: ResponseError::Execution(
                                "Invalid option",
                                Some("Please select a valid option.".to_string()),
                            ),
                            stages_to_skip: None,
                        });
                    };

                    if let ComponentInteractionDataKind::ChannelSelect { values } =
                        &interaction.data.kind
                    {
                        let channel = values.first().ok_or_else(|| {
                            ResponseError::Execution(
                                "No channel selected",
                                Some("Please select a channel.".to_string()),
                            )
                        })?;

                        match edit_mode {
                            EditingMode::Add => {
                                if blacklisted_channels.iter().any(|blacklisted_channel| {
                                    *blacklisted_channel == channel.get() as i64
                                }) {
                                    ChannelBlacklistManage::save_blacklisted_channels(
                                        &blacklisted_channels,
                                        handler,
                                        ctx,
                                    )
                                    .await?;
                                    return Err(ConfigError {
                                        error: ResponseError::Execution(
                                            "Cannot add channel",
                                            Some(
                                                "This channel is already added, you cannot add it again."
                                                    .to_string(),
                                            ),
                                        ),
                                        stages_to_skip: None,
                                    });
                                }

                                let multiplied_channels = sqlx::query!(
                                    "SELECT channel FROM xp_channel_multipliers WHERE guild_id = $1 AND channel = $2",
                                    ctx.guild.id.get() as i64,
                                    channel.get() as i64
                                )
                                .fetch_all(&handler.main_database)
                                .await?
                                .iter()
                                .map(|channel| channel.channel)
                                .collect::<Vec<_>>();

                                if !multiplied_channels.is_empty() {
                                    ChannelBlacklistManage::save_blacklisted_channels(
                                        &blacklisted_channels,
                                        handler,
                                        ctx,
                                    )
                                    .await?;
                                    return Err(ConfigError {
                                        error: ResponseError::Execution(
                                            "Cannot add channel",
                                            Some(
                                                "This channel is currently boosted, you cannot blacklist a boosted channel."
                                                    .to_string(),
                                            ),
                                        ),
                                        stages_to_skip: None,
                                    });
                                }

                                interaction
                                    .create_response(
                                        &ctx.ctx.http,
                                        serenity::builder::CreateInteractionResponse::Acknowledge,
                                    )
                                    .await?;

                                blacklisted_channels.push(channel.get() as i64);
                                editing_mode = None;
                            }
                            EditingMode::Remove => {
                                interaction
                                    .create_response(
                                        &ctx.ctx.http,
                                        serenity::builder::CreateInteractionResponse::Acknowledge,
                                    )
                                    .await?;

                                let Some(index) =
                                    blacklisted_channels.iter().position(|blacklisted_channel| {
                                        *blacklisted_channel == channel.get() as i64
                                    })
                                else {
                                    ChannelBlacklistManage::save_blacklisted_channels(
                                        &blacklisted_channels,
                                        handler,
                                        ctx,
                                    )
                                    .await?;
                                    return Err(ConfigError {
                                        error: ResponseError::Execution(
                                            "Cannot remove channel",
                                            Some(
                                                "This channel is not added, you cannot remove it."
                                                    .to_string(),
                                            ),
                                        ),
                                        stages_to_skip: None,
                                    });
                                };

                                blacklisted_channels.remove(index);
                                editing_mode = None;
                            }
                        }
                    } else {
                        ChannelBlacklistManage::save_blacklisted_channels(
                            &blacklisted_channels,
                            handler,
                            ctx,
                        )
                        .await?;
                        return Err(ConfigError {
                            error: ResponseError::Execution(
                                "Invalid option",
                                Some("Please select a valid option.".to_string()),
                            ),
                            stages_to_skip: None,
                        });
                    }
                }
                "add" => {
                    if editing_mode.is_none() {
                        editing_mode = Some(EditingMode::Add);
                    }
                }
                "remove" => {
                    if editing_mode.is_none() {
                        editing_mode = Some(EditingMode::Remove);
                    }
                }
                "cancel" => {
                    if editing_mode.is_some() {
                        editing_mode = None;
                    }
                }
                "next" => {
                    if (current_page + 1) != max_pages {
                        current_page += 1;
                    }
                }
                "previous" => {
                    if current_page > 0 {
                        current_page = current_page.saturating_sub(1);
                    }
                }
                "done" => {
                    ChannelBlacklistManage::save_blacklisted_channels(
                        &blacklisted_channels,
                        handler,
                        ctx,
                    )
                    .await?;
                    return Ok(None);
                }
                "revert" => return Ok(Some(0)),
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    });
                }
            }

            ctx.reply(
                cmd,
                ChannelBlacklistManage::generate_display_message(
                    &blacklisted_channels,
                    current_page,
                    editing_mode.is_some(),
                ),
            )
            .await?;
        }
        Ok(None)
    }
}
