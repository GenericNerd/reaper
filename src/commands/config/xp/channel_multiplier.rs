use std::collections::BTreeMap;

use serenity::{
    all::{
        ActionRowComponent, ButtonStyle, CommandInteraction, ComponentInteractionDataKind,
        CreateActionRow, CreateButton, CreateEmbed, CreateInputText, CreateModal, CreateSelectMenu,
        CreateSelectMenuKind, InputTextStyle, ReactionType,
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

pub struct ChannelMultiplierEnter;
#[async_trait::async_trait]
impl ConfigStage for ChannelMultiplierEnter {
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
                            .description("Would you like to configure channel boosts?")
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

#[derive(serde::Deserialize, Clone)]
struct ChannelMultiplier {
    channel: i64,
    multiplier: f32,
}

pub struct ChannelMultiplierManage;

impl ChannelMultiplierManage {
    const MAX_MULTIPLIERS_PER_PAGE: usize = 25;

    fn max_pages(multiplier_count: usize) -> usize {
        ((multiplier_count as f64 / ChannelMultiplierManage::MAX_MULTIPLIERS_PER_PAGE as f64).ceil()
            as usize)
            .max(1)
    }

    fn generate_display_message(
        multipliers: &Vec<ChannelMultiplier>,
        current_page: usize,
        show_channels: bool,
    ) -> Response {
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
                        .map(|channel| format!("<#{channel}>"))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    true,
                )
            })
            .collect::<Vec<_>>();

        let start = current_page * ChannelMultiplierManage::MAX_MULTIPLIERS_PER_PAGE;
        let mut end = start + ChannelMultiplierManage::MAX_MULTIPLIERS_PER_PAGE;
        if end > fields.len() {
            end = fields.len();
        }
        let channels_to_render = fields[start..end].to_vec();

        let max_pages = ChannelMultiplierManage::max_pages(multipliers_by_mult.len());

        let mut components = vec![];
        if show_channels {
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
                    .disabled(multipliers.is_empty()),
                CreateButton::new("done")
                    .emoji('✅')
                    .style(ButtonStyle::Success),
            ]));
            components.push(CreateActionRow::Buttons(vec![CreateButton::new("revert")
                .style(ButtonStyle::Danger)
                .label("Revert")]));
        }

        Response::new()
            .embed(
                CreateEmbed::new()
                    .title(format!(
                        "Channel Boosts - Page {}/{}",
                        current_page + 1,
                        max_pages
                    ))
                    .fields(channels_to_render)
                    .color(EMBED_COLOR),
            )
            .components(components)
    }

    async fn save_multipliers(
        multipliers: &Vec<ChannelMultiplier>,
        handler: &Handler,
        ctx: &CommandContext,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM xp_channel_multipliers WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .execute(&handler.main_database)
        .await?;
        for multiplier in multipliers {
            sqlx::query!(
                "INSERT INTO xp_channel_multipliers (guild_id, channel, multiplier) VALUES ($1, $2, $3)",
                ctx.guild.id.get() as i64,
                multiplier.channel,
                multiplier.multiplier
            )
            .execute(&handler.main_database)
            .await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl ConfigStage for ChannelMultiplierManage {
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

        let mut multipliers = sqlx::query_as!(
            ChannelMultiplier,
            "SELECT channel, multiplier FROM xp_channel_multipliers WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_all(&handler.main_database)
        .await?;

        let mut current_page = 0;
        let mut editing_mode = None;

        let message = ctx
            .reply_get_message(
                cmd,
                ChannelMultiplierManage::generate_display_message(
                    &multipliers,
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

            let max_pages = ChannelMultiplierManage::max_pages(multipliers.len());

            match interaction.data.custom_id.as_str() {
                "pick_channel" => {
                    let Some(ref edit_mode) = editing_mode else {
                        ChannelMultiplierManage::save_multipliers(&multipliers, handler, ctx)
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
                                if sqlx::query!("SELECT channel FROM xp_channel_blacklists WHERE guild_id = $1 AND channel = $2", ctx.guild.id.get() as i64, channel.get() as i64)
                                    .fetch_optional(&handler.main_database)
                                    .await?
                                    .is_some() {
                                        ChannelMultiplierManage::save_multipliers(
                                            &multipliers,
                                            handler,
                                            ctx,
                                        )
                                        .await?;
                                        return Err(ConfigError {
                                            error: ResponseError::Execution(
                                                "Cannot add channel",
                                                Some(
                                                    "This channel is currently blacklisted, you cannot make it a boosted channel."
                                                        .to_string(),
                                                ),
                                            ),
                                            stages_to_skip: None,
                                        });
                                    }

                                if multipliers
                                    .iter()
                                    .any(|multiplier| multiplier.channel == channel.get() as i64)
                                {
                                    ChannelMultiplierManage::save_multipliers(
                                        &multipliers,
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

                                interaction
                                    .create_response(
                                        &ctx.ctx.http,
                                        serenity::builder::CreateInteractionResponse::Modal(
                                            CreateModal::new("add_multiplier_modal", "Add Boost")
                                                .components(vec![CreateActionRow::InputText(
                                                    CreateInputText::new(
                                                        InputTextStyle::Short,
                                                        "Boost (in %)",
                                                        "multiplier",
                                                    )
                                                    .placeholder("20")
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

                                    let multiplier = if let ActionRowComponent::InputText(text) =
                                        &interaction.data.components[0].components[0]
                                    {
                                        let Ok(multiplier) =
                                            text.value.as_ref().unwrap().parse::<i32>()
                                        else {
                                            ChannelMultiplierManage::save_multipliers(
                                                &multipliers,
                                                handler,
                                                ctx,
                                            )
                                            .await?;
                                            return Err(ResponseError::Execution(
                                                "Invalid multiplier",
                                                Some(
                                                    "Please enter a valid multiplier.".to_string(),
                                                ),
                                            )
                                            .into());
                                        };
                                        if multiplier < 1 {
                                            ChannelMultiplierManage::save_multipliers(
                                                &multipliers,
                                                handler,
                                                ctx,
                                            )
                                            .await?;
                                            return Err(ConfigError {
                                                error: ResponseError::Execution(
                                                    "Invalid multiplier",
                                                    Some(
                                                        "Please enter a valid multiplier."
                                                            .to_string(),
                                                    ),
                                                ),
                                                stages_to_skip: None,
                                            });
                                        }
                                        multiplier
                                    } else {
                                        ChannelMultiplierManage::save_multipliers(
                                            &multipliers,
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

                                    multipliers.push(ChannelMultiplier {
                                        channel: channel.get() as i64,
                                        multiplier: (multiplier as f32 / 100.0) + 1.0,
                                    });
                                    editing_mode = None;
                                }
                            }
                            EditingMode::Remove => {
                                interaction
                                    .create_response(
                                        &ctx.ctx.http,
                                        serenity::builder::CreateInteractionResponse::Acknowledge,
                                    )
                                    .await?;

                                let Some(index) = multipliers.iter().position(|multiplier| {
                                    multiplier.channel == channel.get() as i64
                                }) else {
                                    ChannelMultiplierManage::save_multipliers(
                                        &multipliers,
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

                                multipliers.remove(index);
                                editing_mode = None;
                            }
                        }
                    } else {
                        ChannelMultiplierManage::save_multipliers(&multipliers, handler, ctx)
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
                    ChannelMultiplierManage::save_multipliers(&multipliers, handler, ctx).await?;
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
                ChannelMultiplierManage::generate_display_message(
                    &multipliers,
                    current_page,
                    editing_mode.is_some(),
                ),
            )
            .await?;
        }
        Ok(None)
    }
}
