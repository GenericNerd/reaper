// TODO: Add logging

use serenity::all::{
    ActionRowComponent, ButtonStyle, ComponentInteractionDataKind, CreateActionRow, CreateButton,
    CreateEmbed, CreateInputText, CreateModal, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, InputTextStyle, PermissionOverwrite, PermissionOverwriteType,
    Permissions,
};

use crate::{
    components::config::{
        Complete, ConfigEntry, ConfigStage, EMBED_COLOR, advance_to, interaction_builder,
        logging::LoggingEnter,
    },
    models::{
        actions::{ActionEscalation, ActionType},
        bot::Bot,
        context::Context,
        duration::Duration,
        interactions::{
            Interaction, InteractionBuilder,
            config::{ConfigInteraction, LoggingStage, ModerationStage},
        },
        response::{
            ExecutionError, InputError, InternalError, Response, ResponseError, ResponseResult,
        },
        role::Role,
        user::User,
    },
};

const MODERATION_TITLE: &str = "Configuration - Moderation";

fn moderation_interaction_builder(
    user: User,
    stage: ModerationStage,
    single_category: bool,
) -> InteractionBuilder {
    interaction_builder(
        user,
        ConfigInteraction::Moderation { stage },
        single_category,
    )
}

#[derive(Debug)]
pub struct ModerationEnter;
#[async_trait::async_trait]
impl ConfigStage for ModerationEnter {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "enter")
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
            moderation_interaction_builder(context.user, ModerationStage::Footer, data.1).build(),
            interaction_builder(
                context.user,
                ConfigInteraction::Logging {
                    stage: LoggingStage::Enter,
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
                            .title("Moderation")
                            .description("Would you like to configure moderation?")
                            .color(0x5539CC),
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

#[derive(Debug)]
pub struct Footer;
#[async_trait::async_trait]
impl ConfigStage for Footer {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "footer")
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

        let interactions = [
            moderation_interaction_builder(context.user, ModerationStage::ChangeFooter, data.1)
                .build(),
            moderation_interaction_builder(context.user, ModerationStage::MuteRole, data.1).build(),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.clone().to_vec())
            .await?;

        let footer = sqlx::query!(
            "SELECT footer FROM moderation_configuration WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .footer;

        let help_text =
            "> Use the placeholder `{uuid}` to insert the action's universally unique identifier.";

        entry.reply(
            ctx,
            Response::new()
                .embed(
                    CreateEmbed::new()
                        .title(MODERATION_TITLE)
                        .description(format!("You can add a custom footer to DMs sent by Reaper when a user is punished.\n\n{help_text}\n\n{}", match footer {
                            Some(footer) => format!("Your current footer is:\n```{footer}```"),
                            None => "You currently don't have a footer set.".to_string()
                        }))
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
pub struct ChangeFooter;
#[async_trait::async_trait]
impl ConfigStage for ChangeFooter {
    fn key(&self) -> (&'static str, &'static str) {
        ("moderation", "change_footer")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("moderation", "footer"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;

        entry
            .modal(
                ctx,
                CreateModal::new("change_footer_modal", "Footer Text").components(vec![
                    CreateActionRow::InputText(
                        CreateInputText::new(
                            InputTextStyle::Short,
                            "Write {uuid} to replace with the UUID",
                            "footer",
                        )
                        .placeholder("If you want to appeal, reference {uuid}")
                        .required(true),
                    ),
                ]),
            )
            .await?;

        let message = entry.component()?.get_response(&context.ctx.http).await?;

        let modal_collector = message
            .await_modal_interaction(context.ctx)
            .author_id(context.user.as_serenity_id())
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

                sqlx::query!(
                    "UPDATE moderation_configuration SET footer = $1 WHERE guild_id = $2",
                    if value.is_empty() { None } else { Some(value) },
                    context.guild.as_i64(),
                )
                .execute(Bot::global().postgres())
                .await?;

                return advance_to(
                    MuteRole,
                    ctx,
                    entry,
                    (
                        &ConfigInteraction::Moderation {
                            stage: ModerationStage::MuteRole,
                        },
                        data.1,
                    ),
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
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;

        let mute_role = sqlx::query!(
            "SELECT mute_role FROM moderation_configuration WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .mute_role;

        let interactions = [
            moderation_interaction_builder(context.user, ModerationStage::SelectedMuteRole, data.1)
                .build(),
            moderation_interaction_builder(
                context.user,
                ModerationStage::DefaultStrikeDuration,
                data.1,
            )
            .build(),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.clone().to_vec())
            .await?;

        let help_text = r"> The mute role is a special role that Reaper will assign to users when they are affected by a mute.
> 
> - Members with this role will be prevented from sending messages or speaking in voice channels.
> - Reaper will automatically add and remove this role when muting and unmuting users.
> 
> If you change this role, Reaper will automatically modify permissions of all channels to deny this role from sending messages or speaking in voice channels.";

        entry.reply(ctx, Response::new().embed(
            CreateEmbed::new()
                .title(MODERATION_TITLE)
                .description(format!("You can now change what role Reaper will apply during mutes.\n\n{help_text}\n\nThe current mute role is: {}", match mute_role {
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
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;

        let current_role = sqlx::query!(
            "SELECT mute_role FROM moderation_configuration WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .mute_role
        .map(Role::from);

        let ComponentInteractionDataKind::RoleSelect { values } = &entry.component()?.data.kind
        else {
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

        if current_role.is_some() {
            let http = context.ctx.http.clone();
            let partial_guild = context.partial_guild.clone();
            let role_id = role.as_serenity();
            tokio::spawn(async move {
                if let Ok(channels) = partial_guild.channels(&http).await {
                    for (channel_id, _) in channels {
                        let _ = channel_id
                            .create_permission(
                                &http,
                                PermissionOverwrite {
                                    allow: Permissions::empty(),
                                    deny: Permissions::SEND_MESSAGES
                                        | Permissions::SEND_MESSAGES_IN_THREADS
                                        | Permissions::CREATE_PUBLIC_THREADS
                                        | Permissions::CREATE_PRIVATE_THREADS
                                        | Permissions::ADD_REACTIONS
                                        | Permissions::SPEAK
                                        | Permissions::STREAM
                                        | Permissions::USE_VAD,
                                    kind: PermissionOverwriteType::Role(role_id),
                                },
                            )
                            .await;
                    }
                }
            });
        }

        sqlx::query!(
            "UPDATE moderation_configuration SET mute_role = $1 WHERE guild_id = $2",
            role.as_i64(),
            context.guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await?;

        advance_to(DefaultStrikeDuration, ctx, entry, data).await
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
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;

        let default_strike_duration = sqlx::query!(
            "SELECT default_strike_duration FROM moderation_configuration WHERE guild_id = $1",
            context.guild.as_i64()
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .default_strike_duration;

        let help_text = r"When striking a user, how long should it take for a strike to expire?

> Strikes™️ are Reaper's method of punishing users for breaking rules, similar to warns in other bots.
> 
> - Strike escalations allow you to automatically take action against repeat offenders (e.g. mute, kick, ban).
> - When a strike expires, it no longer counts towards strike escalations.
> - You will be able to configure strike escalations later in the config.";

        let interactions = [
            moderation_interaction_builder(
                context.user,
                ModerationStage::ChangeDefaultStrikeDuration,
                data.1,
            )
            .build(),
            moderation_interaction_builder(
                context.user,
                ModerationStage::Escalations { escalations: None },
                data.1,
            )
            .build(),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.clone().to_vec())
            .await?;

        entry.reply(
            ctx,
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
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;

        entry
            .modal(
                ctx,
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

        let message = entry.component()?.get_response(&context.ctx.http).await?;

        let modal_collector = message
            .await_modal_interaction(context.ctx)
            .author_id(context.user.as_serenity_id())
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
                    entry,
                    (
                        &ConfigInteraction::Moderation {
                            stage: ModerationStage::Escalations { escalations: None },
                        },
                        data.1,
                    ),
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
    fn interactions(
        user: User,
        escalations: &[ActionEscalation],
        single_category: bool,
    ) -> Vec<Interaction> {
        [
            moderation_interaction_builder(
                user,
                ModerationStage::AddEscalation {
                    escalations: escalations.to_vec(),
                },
                single_category,
            )
            .build(),
            moderation_interaction_builder(
                user,
                ModerationStage::RemoveEscalation {
                    escalations: escalations.to_vec(),
                },
                single_category,
            )
            .build(),
            moderation_interaction_builder(
                user,
                ModerationStage::SubmitEscalations {
                    escalations: escalations.to_vec(),
                },
                single_category,
            )
            .build(),
            moderation_interaction_builder(
                user,
                ModerationStage::Escalations { escalations: None },
                single_category,
            )
            .build(),
        ]
        .to_vec()
    }

    async fn create_response(
        &self,
        user: User,
        escalations: &[ActionEscalation],
        single_category: bool,
    ) -> Result<Response, ResponseError> {
        let help_text = r"You can configure your strike escalations using the dropdowns below.

> Strike escalations are automatic actions that occur when a user reaches a certain number of strikes.
> 
> - Examples of actions include a mute, kick, or ban.
> - Escalations help automate moderation for repeat offenders.
> - If you make a mistake while configuring, press the Revert button to start over.";

        let mut components = vec![];
        let interactions = Escalations::interactions(user, escalations, single_category);

        Bot::global()
            .interaction_state()
            .register(interactions.clone())
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
                    .description(help_text)
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
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let ConfigInteraction::Moderation { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let escalations = match stage {
            ModerationStage::Escalations { escalations } => escalations,
            ModerationStage::AddEscalation { escalations }
            | ModerationStage::RemoveEscalation { escalations } => &Some(escalations.clone()),
            _ => &None,
        };
        let context = ctx.get_populated_context()?;

        let escalations = match escalations {
            Some(escalations) => escalations,
            None => {
                &sqlx::query_as!(ActionEscalation, "SELECT strike_count, action_type, action_duration FROM strike_escalations WHERE guild_id = $1", context.guild.as_i64()).fetch_all(Bot::global().postgres()).await?
            }
        };

        let response = self
            .create_response(context.user, escalations, data.1)
            .await?;

        entry.reply(ctx, response).await.map(|_| ())
    }
}

#[derive(Debug)]
pub struct AddEscalation;

impl AddEscalation {
    fn modal_components(action_type: ActionType) -> Vec<CreateActionRow> {
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

        modal_components
    }
}

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
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        macro_rules! invalid {
            ("strike") => {
                Err(ResponseError::Execution(ExecutionError::Input(
                    InputError::InvalidStrikeCount,
                )))
            };
            ("duration") => {
                Err(ResponseError::Execution(ExecutionError::Input(
                    InputError::InvalidDuration,
                )))
            };
        }

        let ConfigInteraction::Moderation { stage } = data.0 else {
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

        let action_type = if let ComponentInteractionDataKind::StringSelect { values } =
            &entry.component()?.data.kind
        {
            ActionType::from(&**values.first().ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoActionTypeSelected))
            })?)
        } else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let modal_components = AddEscalation::modal_components(action_type);

        entry
            .modal(
                ctx,
                CreateModal::new("escalation_modal", "Add Escalation").components(modal_components),
            )
            .await?;

        let message = entry.component()?.get_response(&context.ctx.http).await?;

        let modal_collector = message
            .await_modal_interaction(context.ctx)
            .author_id(context.user.as_serenity_id())
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
                    return invalid!("strike");
                };

                if strike_count <= 0 {
                    return invalid!("strike");
                }

                strike_count
            } else {
                return invalid!("strike");
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
                        return invalid!("duration");
                    }
                    Some(value)
                }
            } else {
                return invalid!("duration");
            };

            let mut escalations = escalations.clone();

            escalations.push(ActionEscalation {
                strike_count,
                action_type,
                action_duration,
            });

            return advance_to(Escalations, ctx, entry, data).await;
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
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let ConfigInteraction::Moderation { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let ModerationStage::RemoveEscalation { escalations } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let ComponentInteractionDataKind::StringSelect { values } = &entry.component()?.data.kind
        else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let index = values
            .first()
            .ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoEscalationSelected))
            })?
            .parse::<usize>()
            .map_err(|_| {
                ResponseError::Execution(ExecutionError::Input(InputError::InvalidEscalation))
            })?;

        let mut escalations = escalations.clone();
        escalations.remove(index);

        advance_to(Escalations, ctx, entry, data).await
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
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let ConfigInteraction::Moderation { stage } = data.0 else {
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

        if data.1 {
            advance_to(Complete, ctx, entry, data).await
        } else {
            advance_to(LoggingEnter, ctx, entry, data).await
        }
    }
}
