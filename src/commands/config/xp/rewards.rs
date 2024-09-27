use std::collections::BTreeMap;

use serenity::{
    all::{
        ActionRowComponent, ButtonStyle, CommandInteraction, ComponentInteractionDataKind,
        CreateActionRow, CreateButton, CreateEmbed, CreateInputText, CreateInteractionResponse,
        CreateModal, CreateSelectMenu, CreateSelectMenuKind, InputTextStyle, ReactionType,
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

pub struct RewardsEnter;
#[async_trait::async_trait]
impl ConfigStage for RewardsEnter {
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
                            .description("Would you like to configure level rewards?")
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
struct Reward {
    role: i64,
    level: i64,
}

pub struct ManageRewards;

impl ManageRewards {
    const MAX_REWARDS_PER_PAGE: usize = 25;

    fn max_pages(reward_count: usize) -> usize {
        ((reward_count as f64 / ManageRewards::MAX_REWARDS_PER_PAGE as f64).ceil() as usize).max(1)
    }

    fn generate_display_message(
        rewards: &Vec<Reward>,
        current_page: usize,
        show_roles: bool,
    ) -> Response {
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

        let start = current_page * ManageRewards::MAX_REWARDS_PER_PAGE;
        let mut end = start + ManageRewards::MAX_REWARDS_PER_PAGE;
        if end > fields.len() {
            end = fields.len();
        }
        let fields_to_render = fields[start..end].to_vec();

        let max_pages = ManageRewards::max_pages(rewards_by_level.len());

        let mut components = vec![];
        if show_roles {
            components.push(CreateActionRow::SelectMenu(CreateSelectMenu::new(
                "pick_role",
                CreateSelectMenuKind::Role {
                    default_roles: None,
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
                    .disabled(rewards.is_empty()),
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
                        "Level Rewards - Page {}/{}",
                        current_page + 1,
                        max_pages
                    ))
                    .fields(fields_to_render)
                    .color(EMBED_COLOR),
            )
            .components(components)
    }

    async fn save_rewards(
        rewards: &Vec<Reward>,
        handler: &Handler,
        ctx: &CommandContext,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM xp_rewards WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .execute(&handler.main_database)
        .await?;
        for reward in rewards {
            sqlx::query!(
                "INSERT INTO xp_rewards (guild_id, role, level) VALUES ($1, $2, $3)",
                ctx.guild.id.get() as i64,
                reward.role,
                reward.level
            )
            .execute(&handler.main_database)
            .await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl ConfigStage for ManageRewards {
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

        let mut rewards = sqlx::query_as!(
            Reward,
            "SELECT role, level FROM xp_rewards WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_all(&handler.main_database)
        .await?;

        let mut current_page = 0;
        let mut editing_mode = None;
        let message = ctx
            .reply_get_message(
                cmd,
                ManageRewards::generate_display_message(&rewards, current_page, false),
            )
            .await?;

        let mut collector = message
            .await_component_interactions(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60 * 10, 0))
            .stream();

        while let Some(interaction) = collector.next().await {
            if interaction.data.custom_id.as_str() != "pick_role" {
                interaction
                    .create_response(
                        &ctx.ctx.http,
                        serenity::builder::CreateInteractionResponse::Acknowledge,
                    )
                    .await?;
            }

            let max_pages = ManageRewards::max_pages(rewards.len());

            match interaction.data.custom_id.as_str() {
                "pick_role" => {
                    let Some(ref edit_mode) = editing_mode else {
                        ManageRewards::save_rewards(&rewards, handler, ctx).await?;
                        return Err(ConfigError {
                            error: ResponseError::Execution(
                                "Invalid option",
                                Some("Please select a valid option.".to_string()),
                            ),
                            stages_to_skip: None,
                        });
                    };

                    if let ComponentInteractionDataKind::RoleSelect { values } =
                        &interaction.data.kind
                    {
                        let role = values.first().ok_or_else(|| {
                            ResponseError::Execution(
                                "No role selected",
                                Some("Please select a role.".to_string()),
                            )
                        })?;

                        match edit_mode {
                            EditingMode::Add => {
                                if rewards
                                    .iter()
                                    .any(|reward| reward.role == role.get() as i64)
                                {
                                    ManageRewards::save_rewards(&rewards, handler, ctx).await?;
                                    return Err(ConfigError {
                                        error: ResponseError::Execution(
                                            "Cannot add role",
                                            Some("This role is already added, you cannot add it again.".to_string()),
                                        ),
                                        stages_to_skip: None,
                                    });
                                }

                                interaction
                                    .create_response(
                                        &ctx.ctx.http,
                                        CreateInteractionResponse::Modal(
                                            CreateModal::new("add_reward_modal", "Add Reward")
                                                .components(vec![CreateActionRow::InputText(
                                                    CreateInputText::new(
                                                        InputTextStyle::Short,
                                                        "Level",
                                                        "reward_level",
                                                    )
                                                    .placeholder("30")
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

                                    let level = if let ActionRowComponent::InputText(text) =
                                        &interaction.data.components[0].components[0]
                                    {
                                        let Ok(level) = text.value.as_ref().unwrap().parse::<i64>()
                                        else {
                                            ManageRewards::save_rewards(&rewards, handler, ctx)
                                                .await?;
                                            return Err(ResponseError::Execution(
                                                "Invalid level",
                                                Some("Please enter a valid level.".to_string()),
                                            )
                                            .into());
                                        };
                                        if level < 1 {
                                            ManageRewards::save_rewards(&rewards, handler, ctx)
                                                .await?;
                                            return Err(ConfigError {
                                                error: ResponseError::Execution(
                                                    "Invalid level",
                                                    Some("Please enter a valid level.".to_string()),
                                                ),
                                                stages_to_skip: None,
                                            });
                                        }
                                        level
                                    } else {
                                        ManageRewards::save_rewards(&rewards, handler, ctx).await?;
                                        return Err(ConfigError {
                                            error: ResponseError::Execution(
                                                "Invalid option",
                                                Some("Please select a valid option.".to_string()),
                                            ),
                                            stages_to_skip: None,
                                        });
                                    };

                                    rewards.push(Reward {
                                        role: role.get() as i64,
                                        level,
                                    });
                                    editing_mode = None;
                                }
                            }
                            EditingMode::Remove => {
                                interaction
                                    .create_response(
                                        &ctx.ctx.http,
                                        CreateInteractionResponse::Acknowledge,
                                    )
                                    .await?;

                                let Some(index) = rewards
                                    .iter()
                                    .position(|reward| reward.role == role.get() as i64)
                                else {
                                    ManageRewards::save_rewards(&rewards, handler, ctx).await?;
                                    return Err(ConfigError {
                                        error: ResponseError::Execution(
                                            "Cannot remove role",
                                            Some(
                                                "This role is not added, you cannot remove it."
                                                    .to_string(),
                                            ),
                                        ),
                                        stages_to_skip: None,
                                    });
                                };

                                rewards.remove(index);
                                editing_mode = None;
                            }
                        }
                    } else {
                        ManageRewards::save_rewards(&rewards, handler, ctx).await?;
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
                    ManageRewards::save_rewards(&rewards, handler, ctx).await?;
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
                ManageRewards::generate_display_message(
                    &rewards,
                    current_page,
                    editing_mode.is_some(),
                ),
            )
            .await?;
        }

        Ok(None)
    }
}
