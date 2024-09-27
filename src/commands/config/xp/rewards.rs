use std::collections::HashMap;

use serenity::all::{
    ButtonStyle, CommandInteraction, CreateActionRow, CreateButton, CreateEmbed, CreateSelectMenu,
    CreateSelectMenuKind, ReactionType,
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
                        CreateButton::new("no")
                            .label("No")
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
                "no" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

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

#[derive(serde::Deserialize, Clone)]
struct Reward {
    role: i64,
    level: i64,
}

pub struct ManageRewards;

impl ManageRewards {
    fn generate_display_message(
        rewards: &Vec<Reward>,
        current_page: usize,
        show_roles: bool,
    ) -> Response {
        const MAX_REWARDS_PER_PAGE: usize = 25;

        let mut rewards_by_level: HashMap<i64, Vec<i64>> = HashMap::new();
        for reward in rewards {
            if let std::collections::hash_map::Entry::Vacant(e) =
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

        // 25 rewards per page, so we need to calculate which fields are on current_page
        let start = current_page * MAX_REWARDS_PER_PAGE;
        let mut end = start + MAX_REWARDS_PER_PAGE;
        if end > fields.len() {
            end = fields.len();
        }
        let fields_to_render = fields[start..end].to_vec();

        let max_pages =
            (rewards_by_level.len() as f64 / MAX_REWARDS_PER_PAGE as f64).ceil() as usize;

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
                CreateButton::new("close")
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
                    .fields(fields_to_render),
            )
            .components(components)
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
        let rewards = sqlx::query_as!(
            Reward,
            "SELECT role, level FROM xp_rewards WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_all(&handler.main_database)
        .await?;

        let _message = ctx
            .reply_get_message(
                cmd,
                ManageRewards::generate_display_message(&rewards, 0, true),
            )
            .await?;

        tokio::time::sleep(std::time::Duration::new(30, 0)).await;

        let _message = ctx
            .reply_get_message(
                cmd,
                ManageRewards::generate_display_message(&rewards, 0, false),
            )
            .await;

        tokio::time::sleep(std::time::Duration::new(30, 0)).await;

        Ok(None)
    }
}
