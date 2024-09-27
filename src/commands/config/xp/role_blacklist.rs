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

pub struct RoleBlacklistEnter;
#[async_trait::async_trait]
impl ConfigStage for RoleBlacklistEnter {
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
                            .description("Would you like to configure role blacklists?\n\n> Blacklisting a role will prevent anyone with the role from earning XP.")
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

pub struct RoleBlacklistManage;

impl RoleBlacklistManage {
    const MAX_BLACKLISTED_ROLES_PER_PAGE: usize = 25;

    fn max_pages(blacklisted_roles_count: usize) -> usize {
        ((blacklisted_roles_count as f64
            / RoleBlacklistManage::MAX_BLACKLISTED_ROLES_PER_PAGE as f64)
            .ceil() as usize)
            .max(1)
    }

    fn generate_display_message(
        blacklisted_roles: &[i64],
        current_page: usize,
        editing: bool,
    ) -> Response {
        let start = current_page * RoleBlacklistManage::MAX_BLACKLISTED_ROLES_PER_PAGE;
        let mut end = start + RoleBlacklistManage::MAX_BLACKLISTED_ROLES_PER_PAGE;
        if end > blacklisted_roles.len() {
            end = blacklisted_roles.len();
        }
        let blacklisted_roles_to_render = blacklisted_roles[start..end].to_vec();

        let max_pages = RoleBlacklistManage::max_pages(blacklisted_roles.len());

        let mut components = vec![];
        if editing {
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
                    .disabled(blacklisted_roles.is_empty()),
                CreateButton::new("done")
                    .emoji('✅')
                    .style(ButtonStyle::Success),
            ]));
            components.push(CreateActionRow::Buttons(vec![CreateButton::new("revert")
                .style(ButtonStyle::Danger)
                .label("Revert")]));
        }

        let mut embed = CreateEmbed::new().title(format!(
            "Blacklisted Roles - Page {}/{}",
            current_page + 1,
            max_pages
        ));
        if !blacklisted_roles_to_render.is_empty() {
            embed = embed.description(format!(
                "The following roles are currently blacklisted:\n\n{}",
                blacklisted_roles_to_render
                    .iter()
                    .map(|role| format!("<@&{role}>"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
        embed = embed.color(EMBED_COLOR);

        Response::new().embed(embed).components(components)
    }

    async fn save_blacklisted_roles(
        blacklisted_roles: &Vec<i64>,
        handler: &Handler,
        ctx: &CommandContext,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM xp_role_blacklists WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .execute(&handler.main_database)
        .await?;
        for blacklisted_role in blacklisted_roles {
            sqlx::query!(
                "INSERT INTO xp_role_blacklists (guild_id, role) VALUES ($1, $2)",
                ctx.guild.id.get() as i64,
                blacklisted_role
            )
            .execute(&handler.main_database)
            .await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl ConfigStage for RoleBlacklistManage {
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

        let mut blacklisted_roles = sqlx::query!(
            "SELECT role FROM xp_role_blacklists WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_all(&handler.main_database)
        .await?
        .iter()
        .map(|role| role.role)
        .collect::<Vec<_>>();

        let mut current_page = 0;
        let mut editing_mode = None;

        let message = ctx
            .reply_get_message(
                cmd,
                RoleBlacklistManage::generate_display_message(
                    &blacklisted_roles,
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
            if interaction.data.custom_id.as_str() != "pick_role" {
                interaction
                    .create_response(
                        &ctx.ctx.http,
                        serenity::builder::CreateInteractionResponse::Acknowledge,
                    )
                    .await?;
            }

            let max_pages = RoleBlacklistManage::max_pages(blacklisted_roles.len());

            match interaction.data.custom_id.as_str() {
                "pick_role" => {
                    let Some(ref edit_mode) = editing_mode else {
                        RoleBlacklistManage::save_blacklisted_roles(
                            &blacklisted_roles,
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
                                if blacklisted_roles
                                    .iter()
                                    .any(|blacklisted_role| *blacklisted_role == role.get() as i64)
                                {
                                    RoleBlacklistManage::save_blacklisted_roles(
                                        &blacklisted_roles,
                                        handler,
                                        ctx,
                                    )
                                    .await?;
                                    return Err(ConfigError {
                                        error: ResponseError::Execution(
                                            "Cannot add role",
                                            Some(
                                                "This role is already added, you cannot add it again."
                                                    .to_string(),
                                            ),
                                        ),
                                        stages_to_skip: None,
                                    });
                                }

                                let multiplied_roles = sqlx::query!(
                                    "SELECT role FROM xp_role_multipliers WHERE guild_id = $1 AND role = $2",
                                    ctx.guild.id.get() as i64,
                                    role.get() as i64
                                )
                                .fetch_all(&handler.main_database)
                                .await?
                                .iter()
                                .map(|role| role.role)
                                .collect::<Vec<_>>();

                                if !multiplied_roles.is_empty() {
                                    RoleBlacklistManage::save_blacklisted_roles(
                                        &blacklisted_roles,
                                        handler,
                                        ctx,
                                    )
                                    .await?;
                                    return Err(ConfigError {
                                        error: ResponseError::Execution(
                                            "Cannot add role",
                                            Some(
                                                "This role is currently boosted, you cannot blacklist a boosted role."
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

                                blacklisted_roles.push(role.get() as i64);
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
                                    blacklisted_roles.iter().position(|blacklisted_role| {
                                        *blacklisted_role == role.get() as i64
                                    })
                                else {
                                    RoleBlacklistManage::save_blacklisted_roles(
                                        &blacklisted_roles,
                                        handler,
                                        ctx,
                                    )
                                    .await?;
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

                                blacklisted_roles.remove(index);
                                editing_mode = None;
                            }
                        }
                    } else {
                        RoleBlacklistManage::save_blacklisted_roles(
                            &blacklisted_roles,
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
                    RoleBlacklistManage::save_blacklisted_roles(&blacklisted_roles, handler, ctx)
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
                RoleBlacklistManage::generate_display_message(
                    &blacklisted_roles,
                    current_page,
                    editing_mode.is_some(),
                ),
            )
            .await?;
        }

        Ok(None)
    }
}
