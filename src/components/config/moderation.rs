// TODO: Add logging

use serenity::all::{
    ActionRowComponent, ButtonStyle, ComponentInteraction, ComponentInteractionDataKind,
    CreateActionRow, CreateButton, CreateEmbed, CreateInputText, CreateModal, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption, InputTextStyle, Permissions,
};

use crate::{
    components::config::{
        ConfigStage, EMBED_COLOR, advance_to, interaction_builder, logging::LoggingEnter,
    },
    models::{
        actions::{ActionEscalation, ActionType},
        bot::Bot,
        context::{Context, ContextComponentReplies, ContextReply},
        duration::Duration,
        interactions::{
            InteractionBuilder,
            config::{ConfigInteraction, ModerationStage},
        },
        response::{
            ExecutionError, InputError, InternalError, Response, ResponseError, ResponseResult,
        },
        role::Role,
        user::User,
    },
};

const MODERATION_TITLE: &str = "Configuration - Moderation";

pub fn moderation_interaction_builder(user: User, stage: ModerationStage) -> InteractionBuilder {
    interaction_builder(user, ConfigInteraction::Moderation { stage })
}

#[derive(Debug)]
pub struct MuteRole;
#[async_trait::async_trait]
impl ConfigStage for MuteRole {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "mute_role")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        None
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        component: &ComponentInteraction,
        _data: &ConfigInteraction,
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;

        if sqlx::query!(
            "SELECT guild_id FROM moderation_configuration WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .fetch_optional(Bot::global().postgres())
        .await?
        .is_none()
        {
            sqlx::query!(
                "INSERT INTO moderation_configuration (guild_id) VALUES ($1)",
                context.guild.as_i64()
            )
            .execute(Bot::global().postgres())
            .await?;
        }

        let mute_role = sqlx::query!(
            "SELECT mute_role FROM moderation_configuration WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .mute_role;

        let user = User::from(component.user.id);
        let interactions = [
            moderation_interaction_builder(user, ModerationStage::SelectedMuteRole).build(),
            moderation_interaction_builder(user, ModerationStage::DefaultStrikeDuration).build(),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.clone().to_vec())
            .await?;

        ctx.reply(component, Response::new().embed(
            CreateEmbed::new()
                .title(MODERATION_TITLE)
                // TODO: Improve and add descriptions
                .description(format!("You can add a role that Reaper will use to mute users.\nThe current mute role is: {}", match mute_role {
                    Some(role) => format!("<@&{role}>"),
                    None => "None".to_string(),
                }))
                .color(EMBED_COLOR),
        )
        .components(vec![
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    interactions[0].id,
                    CreateSelectMenuKind::Role { default_roles: None }
                )),
            CreateActionRow::Buttons(vec![
                CreateButton::new(interactions[1].id)
                    .label("Skip")
                    .style(ButtonStyle::Secondary),
            ])
        ])).await.map(|_| ())
    }
}

#[derive(Debug)]
pub struct SelectedMuteRole;
#[async_trait::async_trait]
impl ConfigStage for SelectedMuteRole {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "selected_mute_role")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("moderation", "mute_role"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        component: &ComponentInteraction,
        data: &ConfigInteraction,
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;

        let ComponentInteractionDataKind::RoleSelect { values } = &component.data.kind else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let role = values.first().ok_or_else(|| {
            ResponseError::Execution(ExecutionError::Input(InputError::NoRoleSelected))
        })?;
        let role = Role::from(*role);

        let role_position = match context.partial_guild.roles.get(&role.as_serenity()) {
            Some(role) => {
                if role.permissions.contains(Permissions::ADMINISTRATOR) {
                    u16::MAX - 1
                } else {
                    role.position
                }
            }
            None => {
                return Err(ResponseError::Execution(ExecutionError::Input(
                    InputError::InvalidRole {
                        message:
                            "Please select a role that does not have administrator permissions"
                                .to_string(),
                    },
                )));
            }
        };

        if role_position >= context.highest_role {
            return Err(ResponseError::Execution(ExecutionError::Input(
                InputError::InvalidRole {
                    message: "Please select a role that is lower than your highest role"
                        .to_string(),
                },
            )));
        }

        // TODO: Configure role for user?

        sqlx::query!(
            "UPDATE moderation_configuration SET mute_role = $1 WHERE guild_id = $2",
            role.as_i64(),
            context.guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await?;

        advance_to(DefaultStrikeDuration, ctx, component, data).await
    }
}

#[derive(Debug)]
pub struct DefaultStrikeDuration;
#[async_trait::async_trait]
impl ConfigStage for DefaultStrikeDuration {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "default_strike_duration")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        None
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        component: &ComponentInteraction,
        _data: &ConfigInteraction,
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;

        let default_strike_duration = sqlx::query!(
            "SELECT default_strike_duration FROM moderation_configuration WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .default_strike_duration;

        let help_text = r"> This refers to how long it takes for a strike to expire.
> Strikes™️ are Reaper's method of punishing users for breaking a rule, like the warns used in other bots!
> When a strike expires, they will not count towards strike escalations. Strike escalations allow you to automatically action against users for breaking a rule.
> You will be able to configure strike escalations later in the config.";

        let user = User::from(component.user.id);
        let interactions = [
            moderation_interaction_builder(user, ModerationStage::ChangeDefaultStrikeDuration)
                .build(),
            moderation_interaction_builder(
                user,
                ModerationStage::Escalations { escalations: None },
            )
            .build(),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.clone().to_vec())
            .await?;

        ctx.reply(
            component,
            Response::new()
                .embed(
                    CreateEmbed::new()
                        .title(MODERATION_TITLE)
                        .description(format!(
                                "What should be the default strike duration?\n\n{help_text}\n\nYour current setting is: **{}**",
                                default_strike_duration.as_deref().unwrap_or("30d")
                            ))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![
                        CreateActionRow::Buttons(vec![
                            CreateButton::new(interactions[0].id)
                                .label("Change")
                                .style(ButtonStyle::Success),
                            CreateButton::new(interactions[1].id)
                                .label("Skip")
                                .style(ButtonStyle::Secondary)
                        ]),
                    ]),
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct ChangeDefaultStrikeDuration;
#[async_trait::async_trait]
impl ConfigStage for ChangeDefaultStrikeDuration {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "change_default_strike_duration")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("moderation", "default_strike_duration"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        component: &ComponentInteraction,
        _data: &ConfigInteraction,
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;

        ctx.modal(
            component,
            CreateModal::new("default_strike_duration_modal", "Default Strike Duration")
                .components(vec![CreateActionRow::InputText(
                    CreateInputText::new(
                        InputTextStyle::Short,
                        "Duration",
                        "default_strike_duration",
                    )
                    .placeholder("30d")
                    .required(true),
                )]),
        )
        .await?;

        let user = User::from(component.user.id);
        let message = component.get_response(&context.ctx.http).await?;

        let modal_collector = message
            .await_modal_interaction(context.ctx)
            .author_id(user.as_serenity_id())
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
                let Some(duration) = Duration::new(value.as_str()).to_timestamp() else {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidDuration,
                    )));
                };
                if duration < time::OffsetDateTime::now_utc() {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidDuration,
                    )));
                }

                sqlx::query!(
                    "UPDATE moderation_configuration SET default_strike_duration = $1 WHERE guild_id = $2",
                    value,
                    context.guild.as_i64()
                ).execute(Bot::global().postgres()).await?;

                return advance_to(
                    Escalations,
                    ctx,
                    component,
                    &ConfigInteraction::Moderation {
                        stage: ModerationStage::Escalations { escalations: None },
                    },
                )
                .await;
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
pub struct Escalations;

impl Escalations {
    async fn create_response(
        &self,
        user: User,
        escalations: &[ActionEscalation],
    ) -> Result<Response, ResponseError> {
        let description_text = r"You can configure your strike escalations using the dropdowns below.

> What are escalations?
> 
> These are automatic actions (such as a mute, kick, or ban) that occur when a user reaches a certain number of strikes.
> If you make a mistake, press the Revert button to start over.";

        let mut components = vec![];
        let interactions = [
            moderation_interaction_builder(
                user,
                ModerationStage::AddEscalation {
                    escalations: escalations.to_vec(),
                },
            )
            .build(),
            moderation_interaction_builder(
                user,
                ModerationStage::RemoveEscalation {
                    escalations: escalations.to_vec(),
                },
            )
            .build(),
            moderation_interaction_builder(
                user,
                ModerationStage::SubmitEscalations {
                    escalations: escalations.to_vec(),
                },
            )
            .build(),
            moderation_interaction_builder(
                user,
                ModerationStage::Escalations { escalations: None },
            )
            .build(),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.clone().to_vec())
            .await?;

        if escalations.len() < 15 {
            components.push(CreateActionRow::SelectMenu(CreateSelectMenu::new(
                interactions[0].id.to_string(),
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
                interactions[1].id.to_string(),
                CreateSelectMenuKind::String {
                    options: (0..escalations.len())
                        .map(|index| {
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
            CreateButton::new(interactions[2].id)
                .label("Done")
                .style(ButtonStyle::Success),
            CreateButton::new(interactions[3].id)
                .label("Revert")
                .style(ButtonStyle::Danger),
        ]));

        Ok(Response::new()
            .embed(
                CreateEmbed::new()
                    .title(MODERATION_TITLE)
                    .description(description_text)
                    .color(EMBED_COLOR)
                    .fields(escalations.iter().enumerate().map(|(index, escalation)| {
                        (
                            format!(
                                "{}{} escalation",
                                index + 1,
                                ordinal::Ordinal(index + 1).suffix()
                            ),
                            format!(
                                "At **{}** strikes, Reaper will **{}** the user {}.",
                                escalation.strike_count,
                                escalation.action_type,
                                match escalation.action_duration.as_ref() {
                                    Some(duration) => format!("for **{duration}**"),
                                    None => "**indefinitely**".to_string(),
                                }
                            ),
                            false,
                        )
                    })),
            )
            .components(components))
    }
}

#[async_trait::async_trait]
impl ConfigStage for Escalations {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "escalations")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        None
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        component: &ComponentInteraction,
        data: &ConfigInteraction,
    ) -> ResponseResult {
        let ConfigInteraction::Moderation { stage } = data else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let ModerationStage::Escalations { escalations } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let context = ctx.get_populated_context()?;

        let escalations = match escalations {
            Some(escalations) => escalations,
            None => {
                &sqlx::query_as!(ActionEscalation, "SELECT strike_count, action_type, action_duration FROM strike_escalations WHERE guild_id = $1", context.guild.as_i64()).fetch_all(Bot::global().postgres()).await?
            }
        };
        let user = User::from(component.user.id);
        let response = self.create_response(user, escalations).await?;

        ctx.reply(component, response).await.map(|_| ())
    }
}

#[derive(Debug)]
pub struct AddEscalation;
#[async_trait::async_trait]
impl ConfigStage for AddEscalation {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "add_escalation")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("moderation", "escalations"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        component: &ComponentInteraction,
        data: &ConfigInteraction,
    ) -> ResponseResult {
        let ConfigInteraction::Moderation { stage } = data else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let ModerationStage::AddEscalation { escalations } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let context = ctx.get_populated_context()?;
        let user = User::from(component.user.id);

        let action_type = if let ComponentInteractionDataKind::StringSelect { values } =
            &component.data.kind
        {
            ActionType::from(&**values.first().ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoActionTypeSelected))
            })?)
        } else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
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
                CreateInputText::new(InputTextStyle::Short, "Duration", "escalation_duration")
                    .placeholder("30d")
                    .required(action_type == ActionType::Mute),
            ));
        }

        ctx.modal(
            component,
            CreateModal::new("escalation_modal", "Add Escalation").components(modal_components),
        )
        .await?;

        let message = component.get_response(&context.ctx.http).await?;

        let modal_collector = message
            .await_modal_interaction(context.ctx)
            .author_id(user.as_serenity_id())
            .timeout(std::time::Duration::new(300, 0));

        if let Some(modal_interaction) = modal_collector.await {
            modal_interaction
                .create_response(
                    &context.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;

            let strike_count = if let ActionRowComponent::InputText(text) =
                &modal_interaction.data.components[0].components[0]
            {
                let Ok(strike_count) = text.value.as_ref().unwrap().parse::<i32>().map_err(|_| {
                    ResponseError::Execution(ExecutionError::Input(InputError::InvalidStrikeCount))
                }) else {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidStrikeCount,
                    )));
                };

                if strike_count <= 0 {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidStrikeCount,
                    )));
                }

                strike_count
            } else {
                return Err(ResponseError::Execution(ExecutionError::Input(
                    InputError::InvalidStrikeCount,
                )));
            };

            let action_duration = if action_type == ActionType::Kick {
                None
            } else if let ActionRowComponent::InputText(text) =
                &modal_interaction.data.components[1].components[0]
            {
                let value = text.value.clone().unwrap();
                if value.is_empty() {
                    None
                } else {
                    let duration = Duration::new(value.as_str()).to_timestamp().unwrap();
                    if duration < time::OffsetDateTime::now_utc() {
                        return Err(ResponseError::Execution(ExecutionError::Input(
                            InputError::InvalidDuration,
                        )));
                    }
                    Some(value)
                }
            } else {
                return Err(ResponseError::Execution(ExecutionError::Input(
                    InputError::InvalidDuration,
                )));
            };

            let mut escalations = escalations.clone();

            escalations.push(ActionEscalation {
                strike_count,
                action_type,
                action_duration,
            });

            return advance_to(
                Escalations,
                ctx,
                component,
                &ConfigInteraction::Moderation {
                    stage: ModerationStage::Escalations {
                        escalations: Some(escalations),
                    },
                },
            )
            .await;
        }

        Err(ResponseError::Execution(ExecutionError::Input(
            InputError::Timeout {
                duration: "5 minutes".to_string(),
            },
        )))
    }
}

#[derive(Debug)]
pub struct RemoveEscalation;
#[async_trait::async_trait]
impl ConfigStage for RemoveEscalation {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "remove_escalation")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("moderation", "escalations"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        component: &ComponentInteraction,
        data: &ConfigInteraction,
    ) -> ResponseResult {
        let ConfigInteraction::Moderation { stage } = data else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let ModerationStage::RemoveEscalation { escalations } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let ComponentInteractionDataKind::StringSelect { values } = &component.data.kind else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let index = values
            .first()
            .ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoRoleSelected))
            })?
            .parse::<usize>()
            .map_err(|_| {
                ResponseError::Execution(ExecutionError::Input(InputError::InvalidRole {
                    message: "We couldn't quite see that role. Please try again!".to_string(),
                }))
            })?;

        let mut escalations = escalations.clone();
        escalations.remove(index);

        advance_to(
            Escalations,
            ctx,
            component,
            &ConfigInteraction::Moderation {
                stage: ModerationStage::Escalations {
                    escalations: Some(escalations),
                },
            },
        )
        .await
    }
}

#[derive(Debug)]
pub struct SubmitEscalations;
#[async_trait::async_trait]
impl ConfigStage for SubmitEscalations {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "submit_escalations")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("moderation", "escalations"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        component: &ComponentInteraction,
        data: &ConfigInteraction,
    ) -> ResponseResult {
        let ConfigInteraction::Moderation { stage } = data else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let ModerationStage::SubmitEscalations { escalations } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let context = ctx.get_populated_context()?;

        sqlx::query!(
            "DELETE FROM strike_escalations WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await?;

        for escalation in escalations {
            sqlx::query!(
                "INSERT INTO strike_escalations (guild_id, strike_count, action_type, action_duration) VALUES ($1, $2, $3, $4)",
                context.guild.as_i64(),
                escalation.strike_count,
                escalation.action_type.to_string(),
                escalation.action_duration
            ).execute(Bot::global().postgres()).await?;
        }

        advance_to(LoggingEnter, ctx, component, data).await
    }
}
