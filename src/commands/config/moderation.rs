use serenity::{
    all::{
        ActionRowComponent, ButtonStyle, CommandInteraction, ComponentInteractionDataKind,
        InputTextStyle,
    },
    builder::{
        CreateActionRow, CreateButton, CreateEmbed, CreateInputText, CreateInteractionResponse,
        CreateModal, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
    },
    futures::StreamExt,
};

use crate::{
    common::duration::Duration,
    models::{
        actions::{ActionEscalation, ActionType},
        command::{CommandContext, CommandContextReply},
        handler::Handler,
        response::{Response, ResponseError},
    },
};

use super::{ConfigError, ConfigStage, EMBED_COLOR};

const MODERATION_TITLE: &str = "Configuration - Moderation";

pub struct ModerationEscalations;

impl ModerationEscalations {
    fn generate_message(escalations: &Vec<ActionEscalation>) -> Response {
        let mut components = Vec::with_capacity(1);
        if escalations.len() < 15 {
            components.push(CreateActionRow::SelectMenu(CreateSelectMenu::new(
                "add_escalation",
                CreateSelectMenuKind::String {
                    options: vec![
                        CreateSelectMenuOption::new("Add a mute escalation", "mute"),
                        CreateSelectMenuOption::new("Add a kick escalation", "kick"),
                        CreateSelectMenuOption::new("Add a ban escalation", "ban"),
                    ],
                },
            )));
        }
        if !escalations.is_empty() {
            components.push(CreateActionRow::SelectMenu(CreateSelectMenu::new(
                "remove_escalation",
                CreateSelectMenuKind::String {
                    options: (0..escalations.len()).map(|index| {
                            CreateSelectMenuOption::new(
                                format!(
                                    "Remove {}{} escalation",
                                    index + 1,
                                    ordinal::Ordinal(index + 1).suffix()
                                ),
                                index.to_string(),
                            )
                        })
                        .collect(),
                },
            )));
        }
        components.push(CreateActionRow::Buttons(vec![
            CreateButton::new("done")
                .label("Done")
                .style(ButtonStyle::Success),
            CreateButton::new("revert")
                .label("Revert")
                .style(ButtonStyle::Danger),
        ]));
        Response::new()
            .embed(
                CreateEmbed::new()
                    .title(MODERATION_TITLE)
                    .description("You can now configure your strike escalations. These are actions that will happen when a user reaches a certain amount of strikes.")
                    .color(EMBED_COLOR)
                    .fields(escalations.iter().enumerate().map(|(index, escalation)| {
                        (format!("{}{} escalation", index + 1, ordinal::Ordinal(index + 1).suffix()), format!("At **{}** strikes, Reaper will **{}** the user {}.", escalation.strike_count, escalation.action_type.to_string(), match escalation.action_duration.as_ref() {
                            Some(duration) => format!("for **{duration}**"),
                            None => "**indefinitely**".to_string(),
                        }), false)
                    })),
            )
            .components(components)
    }

    async fn save_escalations(
        escalations: &Vec<ActionEscalation>,
        handler: &Handler,
        ctx: &CommandContext,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM strike_escalations WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .execute(&handler.main_database)
        .await?;
        for escalation in escalations {
            sqlx::query!(
                "INSERT INTO strike_escalations (guild_id, strike_count, action_type, action_duration) VALUES ($1, $2, $3, $4)",
                ctx.guild.id.get() as i64,
                i32::try_from(escalation.strike_count).unwrap(),
                escalation.action_type.to_string(),
                escalation.action_duration
            )
            .execute(&handler.main_database)
            .await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl ConfigStage for ModerationEscalations {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let original_escalations = sqlx::query_as!(
            ActionEscalation,
            "SELECT * FROM strike_escalations WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_all(&handler.main_database)
        .await?;
        let mut escalations = original_escalations.clone();

        let message = ctx
            .reply_get_message(cmd, ModerationEscalations::generate_message(&escalations))
            .await?;

        let mut collector = message
            .await_component_interactions(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60 * 5, 0))
            .stream();

        while let Some(interaction) = collector.next().await {
            if interaction.data.custom_id.as_str() != "add_escalation" {
                interaction
                    .create_response(
                        &ctx.ctx.http,
                        serenity::builder::CreateInteractionResponse::Acknowledge,
                    )
                    .await?;
            }
            match interaction.data.custom_id.as_str() {
                "add_escalation" => {
                    let action_type = if let ComponentInteractionDataKind::StringSelect { values } =
                        &interaction.data.kind
                    {
                        ActionType::from(values.first().unwrap().clone())
                    } else {
                        ModerationEscalations::save_escalations(&escalations, handler, ctx).await?;
                        return Err(ConfigError {
                            error: ResponseError::Execution(
                                "Invalid option",
                                Some("Please select a valid option.".to_string()),
                            ),
                            stages_to_skip: None,
                        });
                    };

                    let mut modal_components = vec![CreateActionRow::InputText(
                        CreateInputText::new(
                            InputTextStyle::Short,
                            "Strike Count",
                            "escalation_strike_count",
                        )
                        .placeholder("3")
                        .required(true),
                    )];
                    if action_type != ActionType::Kick {
                        modal_components.push(CreateActionRow::InputText(
                            CreateInputText::new(
                                InputTextStyle::Short,
                                "Duration",
                                "escalation_duration",
                            )
                            .placeholder("30d")
                            .required(action_type == ActionType::Mute),
                        ));
                    }
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            CreateInteractionResponse::Modal(
                                CreateModal::new("add_escalation_modal", "Add Escalation")
                                    .components(modal_components),
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

                        let strike_count = if let ActionRowComponent::InputText(text) =
                            &interaction.data.components[0].components[0]
                        {
                            let Ok(strike_count) = text.value.as_ref().unwrap().parse::<i64>()
                            else {
                                ModerationEscalations::save_escalations(&escalations, handler, ctx)
                                    .await?;
                                return Err(ResponseError::Execution(
                                    "Invalid strike count",
                                    Some("Please enter a valid strike count.".to_string()),
                                )
                                .into());
                            };
                            if strike_count > 0 {
                                if escalations
                                    .iter()
                                    .any(|escalation| escalation.strike_count == strike_count)
                                {
                                    ModerationEscalations::save_escalations(
                                        &escalations,
                                        handler,
                                        ctx,
                                    )
                                    .await?;
                                    return Err(ConfigError {
                                        error: ResponseError::Execution(
                                            "Invalid strike count",
                                            Some(
                                                "You cannot enter a duplicate strike count."
                                                    .to_string(),
                                            ),
                                        ),
                                        stages_to_skip: None,
                                    });
                                }
                                strike_count
                            } else {
                                ModerationEscalations::save_escalations(&escalations, handler, ctx)
                                    .await?;
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid strike count",
                                        Some(
                                            "You cannot enter a strike count below 0.".to_string(),
                                        ),
                                    ),
                                    stages_to_skip: None,
                                });
                            }
                        } else {
                            ModerationEscalations::save_escalations(&escalations, handler, ctx)
                                .await?;
                            return Err(ConfigError {
                                error: ResponseError::Execution(
                                    "Invalid option",
                                    Some("Please select a valid option.".to_string()),
                                ),
                                stages_to_skip: None,
                            });
                        };

                        let action_duration = if action_type == ActionType::Kick {
                            None
                        } else if let ActionRowComponent::InputText(text) =
                            &interaction.data.components[1].components[0]
                        {
                            let value = text.value.clone().unwrap();
                            if value == String::new() {
                                None
                            } else {
                                let duration =
                                    Duration::new(value.as_str()).to_timestamp().unwrap();
                                if duration < time::OffsetDateTime::now_utc() {
                                    ModerationEscalations::save_escalations(
                                        &escalations,
                                        handler,
                                        ctx,
                                    )
                                    .await?;
                                    return Err(ConfigError {
                                        error: ResponseError::Execution(
                                            "Invalid duration",
                                            Some("Please enter a valid duration.".to_string()),
                                        ),
                                        stages_to_skip: None,
                                    });
                                }
                                Some(value)
                            }
                        } else {
                            ModerationEscalations::save_escalations(&escalations, handler, ctx)
                                .await?;
                            return Err(ConfigError {
                                error: ResponseError::Execution(
                                    "Invalid option",
                                    Some("Please select a valid option.".to_string()),
                                ),
                                stages_to_skip: None,
                            });
                        };

                        escalations.push(ActionEscalation {
                            guild_id: ctx.guild.id.get() as i64,
                            strike_count,
                            action_type,
                            action_duration,
                        });
                    } else {
                        ModerationEscalations::save_escalations(&escalations, handler, ctx).await?;
                        return Err(ConfigError {
                            error: ResponseError::Execution(
                                "Time out",
                                Some(
                                    "We didn't get a response in time. Please try again."
                                        .to_string(),
                                ),
                            ),
                            stages_to_skip: Some(100),
                        });
                    }
                }
                "remove_escalation" => {
                    if let ComponentInteractionDataKind::StringSelect { values } =
                        &interaction.data.kind
                    {
                        let Ok(index) = values.first().unwrap().parse::<usize>() else {
                            ModerationEscalations::save_escalations(&escalations, handler, ctx)
                                .await?;
                            return Err(ResponseError::Execution(
                                "Invalid escalation",
                                Some("Please select a valid escalation.".to_string()),
                            )
                            .into());
                        };

                        escalations.remove(index);
                    } else {
                        ModerationEscalations::save_escalations(&escalations, handler, ctx).await?;
                        return Err(ConfigError {
                            error: ResponseError::Execution(
                                "Invalid option",
                                Some("Please select a valid option.".to_string()),
                            ),
                            stages_to_skip: None,
                        });
                    }
                }
                "done" => {
                    ModerationEscalations::save_escalations(&escalations, handler, ctx).await?;
                    return Ok(None);
                }
                "revert" => {
                    return Ok(Some(0));
                }
                _ => {
                    ModerationEscalations::save_escalations(&escalations, handler, ctx).await?;
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    });
                }
            }
            ctx.reply(cmd, ModerationEscalations::generate_message(&escalations))
                .await?;
        }

        Ok(None)
    }
}

pub struct ModerationDefaultStrikeDuration;
#[async_trait::async_trait]
impl ConfigStage for ModerationDefaultStrikeDuration {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let default_strike_duration = sqlx::query!(
            "SELECT default_strike_duration FROM moderation_configuration WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?
        .default_strike_duration;

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(MODERATION_TITLE)
                            .description(format!(
                                "You can configure the default strike duration, it is currently set to **{}**",
                                default_strike_duration.unwrap_or("30d".to_string())
                            ))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![
                        CreateActionRow::Buttons(vec![
                            CreateButton::new("change")
                                .label("Change")
                                .style(ButtonStyle::Success),
                            CreateButton::new("skip")
                                .label("Skip")
                                .style(ButtonStyle::Secondary)
                        ]),
                    ]),
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
                    return Ok(None);
                }
                "change" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            CreateInteractionResponse::Modal(
                                CreateModal::new(
                                    "default_strike_duration_modal",
                                    "Default Strike Duration",
                                )
                                .components(vec![
                                    CreateActionRow::InputText(
                                        CreateInputText::new(
                                            InputTextStyle::Short,
                                            "Duration",
                                            "default_strike_duration",
                                        )
                                        .placeholder("30d")
                                        .required(true),
                                    ),
                                ]),
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
                            let value = text.value.clone().unwrap();
                            if value == String::new() {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid duration",
                                        Some("Please enter a valid duration.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            }
                            let duration = Duration::new(value.as_str()).to_timestamp().unwrap();
                            if duration < time::OffsetDateTime::now_utc() {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid duration",
                                        Some("Please enter a valid duration.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            }
                            sqlx::query!(
                                "UPDATE moderation_configuration SET default_strike_duration = $1 WHERE guild_id = $2",
                                value,
                                ctx.guild.id.get() as i64
                            ).execute(&handler.main_database).await?;
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
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Time out",
                            Some("We didn't get a response in time. Please try again.".to_string()),
                        ),
                        stages_to_skip: Some(100),
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

pub struct ModerationMuteRole;
#[async_trait::async_trait]
impl ConfigStage for ModerationMuteRole {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let mute_role = sqlx::query!(
            "SELECT mute_role FROM moderation_configuration WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?
        .mute_role;

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(MODERATION_TITLE)
                            .description(format!("You can add a role that Reaper will use to mute users.\nThe current mute role is: {}", match mute_role {
                                Some(role) => format!("<@&{role}>"),
                                None => "None".to_string(),
                            }))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![
                        CreateActionRow::SelectMenu(
                            CreateSelectMenu::new(
                                "mute_role",
                                CreateSelectMenuKind::Role { default_roles: None }
                            )),
                        CreateActionRow::Buttons(vec![
                            CreateButton::new("skip")
                                .label("Skip")
                                .style(ButtonStyle::Secondary),
                        ])
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
                "skip" => return Ok(None),
                "mute_role" => {
                    if let ComponentInteractionDataKind::RoleSelect { values } =
                        interaction.data.kind
                    {
                        let role = values
                            .get(0)
                            .ok_or_else(|| {
                                ResponseError::Execution(
                                    "No role selected",
                                    Some("Please select a role.".to_string()),
                                )
                            })?
                            .get() as i64;

                        sqlx::query!(
                            "UPDATE moderation_configuration SET mute_role = $1 WHERE guild_id = $2",
                            role,
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

pub struct ModerationEnter;
#[async_trait::async_trait]
impl ConfigStage for ModerationEnter {
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
                            .title(MODERATION_TITLE)
                            .description("Would you like to configure moderation?")
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
