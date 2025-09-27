use serenity::all::{
    ActionRowComponent, ButtonStyle, ComponentInteractionDataKind, CreateActionRow, CreateButton,
    CreateEmbed, CreateEmbedFooter, CreateInputText, CreateModal, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption, InputTextStyle, ReactionType,
};

use crate::{
    components::config::{
        ConfigEntry, ConfigStage, EMBED_COLOR, EditMode, advance_to, interaction_builder,
    },
    models::{
        bot::Bot,
        channel::Channel,
        context::Context,
        interactions::{
            Interaction, InteractionBuilder,
            config::{BoardsStage, ConfigInteraction},
        },
        response::{
            ExecutionError, InputError, InternalError, Response, ResponseError, ResponseResult,
        },
        user::User,
    },
};

const BOARDS_TITLE: &str = "Configuration - Boards";

fn boards_interaction_builder(
    user: User,
    stage: BoardsStage,
    single_category: bool,
) -> InteractionBuilder {
    interaction_builder(user, ConfigInteraction::Boards { stage }, single_category)
}

#[derive(Debug)]
pub struct BoardsEnter;
#[async_trait::async_trait]
impl ConfigStage for BoardsEnter {
    fn key(&self) -> (&'static str, &'static str) {
        ("boards", "enter")
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
            boards_interaction_builder(context.user, BoardsStage::SelectBoardChannel, data.1)
                .build(),
            interaction_builder(context.user, ConfigInteraction::Complete, data.1).build(),
        ];

        let help_text = r"Boards automatically repost messages that receive enough reactions into a chosen channel.

> - You can create multiple boards, each using different channels.
> - You can also manage existing boards here.
> - Each board can have its own requirements (quota, emotes, etc.).";

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
                            .title(BOARDS_TITLE)
                            .description(format!(
                                "Would you like to configure boards?\n\n{help_text}"
                            ))
                            .color(EMBED_COLOR),
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
pub struct SelectBoardChannel;
#[async_trait::async_trait]
impl ConfigStage for SelectBoardChannel {
    fn key(&self) -> (&'static str, &'static str) {
        ("boards", "select_board_channel")
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
            boards_interaction_builder(
                context.user,
                BoardsStage::EditSettingsOrEmotes { channel_id: None },
                data.1,
            )
            .build(),
            // TODO: Change me
            interaction_builder(context.user, ConfigInteraction::Complete, data.1).build(),
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
                            .title(BOARDS_TITLE)
                            .description(
                                "Select a channel to create a board or edit it's settings.",
                            )
                            .color(EMBED_COLOR),
                    )
                    .components(vec![
                        CreateActionRow::SelectMenu(CreateSelectMenu::new(
                            interactions[0].id.to_string(),
                            CreateSelectMenuKind::Channel {
                                channel_types: None,
                                default_channels: None,
                            },
                        )),
                        CreateActionRow::Buttons(vec![
                            CreateButton::new(interactions[1].id.to_string())
                                .style(ButtonStyle::Success)
                                .label("Done"),
                        ]),
                    ]),
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct EditSettingsOrEmotes;
#[async_trait::async_trait]
impl ConfigStage for EditSettingsOrEmotes {
    fn key(&self) -> (&'static str, &'static str) {
        ("boards", "edit_settings_or_emotes")
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

        let ConfigInteraction::Boards { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let channel_id = if let BoardsStage::EditSettingsOrEmotes { channel_id } = stage {
            channel_id
        } else {
            &None
        };

        let channel = if let Some(channel_id) = channel_id {
            Channel::from(*channel_id)
        } else {
            let ComponentInteractionDataKind::ChannelSelect { values } =
                &entry.component()?.data.kind
            else {
                return Err(ResponseError::Execution(ExecutionError::Internal(
                    InternalError::InvalidInteractionType,
                )));
            };
            let channel = values.first().ok_or_else(|| {
                ResponseError::Execution(ExecutionError::Input(InputError::NoChannelSelected))
            })?;
            Channel::from(*channel)
        };

        sqlx::query!(
            "INSERT INTO boards (guild_id, channel_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            context.guild.as_i64(),
            channel.as_i64()
        )
        .execute(Bot::global().postgres())
        .await?;

        let board_settings = sqlx::query!(
            "SELECT emote_quota, ignore_self_reacts FROM boards WHERE guild_id = $1 AND channel_id = $2",
            context.guild.as_i64(),
            channel.as_i64()
        )
        .fetch_one(Bot::global().postgres())
        .await?;
        let (emote_quota, ignore_self_reacts) = (
            board_settings.emote_quota,
            board_settings.ignore_self_reacts,
        );
        let emotes = sqlx::query!(
            "SELECT emote FROM board_emotes WHERE guild_id = $1 AND channel_id = $2",
            context.guild.as_i64(),
            channel.as_i64()
        )
        .fetch_all(Bot::global().postgres())
        .await?
        .iter()
        .map(|e| e.emote.clone())
        .collect::<Vec<_>>();

        let help_text = format!(
            r"Would you like to edit the board settings or emotes?

Current settings:
> **Quota**: {emote_quota}
> **Ignoring self-reacts?**: {}
> **Emotes**:
{}",
            if ignore_self_reacts { '✅' } else { '❌' },
            if emotes.is_empty() {
                "> *None*".to_string()
            } else {
                emotes
                    .iter()
                    .map(|e| format!("> - {e}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        );

        let interactions = [
            boards_interaction_builder(
                context.user,
                BoardsStage::Quota {
                    channel_id: channel.as_i64(),
                },
                data.1,
            )
            .build(),
            boards_interaction_builder(
                context.user,
                BoardsStage::Emotes {
                    channel_id: channel.as_i64(),
                    emotes: Some(emotes),
                    page: 0,
                },
                data.1,
            )
            .build(),
            boards_interaction_builder(context.user, BoardsStage::SelectBoardChannel, data.1)
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
                            .title(BOARDS_TITLE)
                            .description(help_text)
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(interactions[0].id.to_string())
                            .style(ButtonStyle::Primary)
                            .label("Edit Settings"),
                        CreateButton::new(interactions[1].id.to_string())
                            .style(ButtonStyle::Primary)
                            .label("Edit Emotes"),
                        CreateButton::new(interactions[2].id.to_string())
                            .style(ButtonStyle::Secondary)
                            .label("Back"),
                    ])]),
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct Quota;
#[async_trait::async_trait]
impl ConfigStage for Quota {
    fn key(&self) -> (&'static str, &'static str) {
        ("boards", "quota")
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
        let ConfigInteraction::Boards { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let BoardsStage::Quota { channel_id } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let help_text = r"How many reactions should a message need to appear on this board?

> - Only reactions with the allowed emotes count.
> - Example: If set to 5, a message needs 5 reactions before it is added.";

        let interactions = [
            boards_interaction_builder(
                context.user,
                BoardsStage::ChangeQuota {
                    channel_id: *channel_id,
                },
                data.1,
            )
            .build(),
            boards_interaction_builder(
                context.user,
                BoardsStage::IgnoreSelfReacts {
                    channel_id: *channel_id,
                },
                data.1,
            )
            .build(),
        ];

        let quota = sqlx::query!(
            "SELECT emote_quota FROM boards WHERE guild_id = $1 AND channel_id = $2",
            context.guild.as_i64(),
            channel_id
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .emote_quota;

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
                            .title(BOARDS_TITLE)
                            .description(format!(
                                "{help_text}\n\nCurrently, the quota in this channel is: **{quota}**"
                            ))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(interactions[0].id.to_string())
                            .style(ButtonStyle::Primary)
                            .label("Change Quota"),
                        CreateButton::new(interactions[1].id.to_string())
                            .style(ButtonStyle::Secondary)
                            .label("Skip"),
                    ])]),
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct ChangeQuota;
#[async_trait::async_trait]
impl ConfigStage for ChangeQuota {
    fn key(&self) -> (&'static str, &'static str) {
        ("boards", "change_quota")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("boards", "quota"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Boards { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let BoardsStage::ChangeQuota { channel_id } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        entry
            .modal(
                ctx,
                CreateModal::new("emote_quota_modal", "Emote Quota").components(vec![
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, "Emote Quota", "emote_quota")
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
                if value.is_empty() {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidNumber,
                    )));
                }
                let Ok(emote_quota) = text.value.as_ref().unwrap().parse::<i32>().map_err(|_| {
                    ResponseError::Execution(ExecutionError::Input(InputError::InvalidNumber))
                }) else {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidNumber,
                    )));
                };

                if emote_quota <= 0 {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidNumber,
                    )));
                }

                sqlx::query!(
                    "UPDATE boards SET emote_quota = $1 WHERE guild_id = $2 AND channel_id = $3",
                    emote_quota,
                    context.guild.as_i64(),
                    channel_id
                )
                .execute(Bot::global().postgres())
                .await?;

                return advance_to(IgnoreSelfReacts, ctx, entry, data).await;
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
pub struct IgnoreSelfReacts;
#[async_trait::async_trait]
impl ConfigStage for IgnoreSelfReacts {
    fn key(&self) -> (&'static str, &'static str) {
        ("boards", "ignore_self_reacts")
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
        let ConfigInteraction::Boards { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let BoardsStage::IgnoreSelfReacts { channel_id } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let help_text = r"Should a user's own reactions count towards the quota for board posts?

> - If enabled, when someone reacts to their **own message**, that reaction will be ignored.
> - This prevents users from pushing their own messages onto the board without community support.
> - If disabled, self-reactions will count the same as reactions from others.";

        let interactions = [
            boards_interaction_builder(
                context.user,
                BoardsStage::ChangeIgnoreSelfReacts {
                    channel_id: *channel_id,
                    is_ignoring: true,
                },
                data.1,
            )
            .build(),
            boards_interaction_builder(
                context.user,
                BoardsStage::ChangeIgnoreSelfReacts {
                    channel_id: *channel_id,
                    is_ignoring: false,
                },
                data.1,
            )
            .build(),
            boards_interaction_builder(
                context.user,
                BoardsStage::EditSettingsOrEmotes {
                    channel_id: Some(*channel_id),
                },
                data.1,
            )
            .build(),
        ];

        let ignoring = sqlx::query!(
            "SELECT ignore_self_reacts FROM boards WHERE guild_id = $1 AND channel_id = $2",
            context.guild.as_i64(),
            channel_id
        )
        .fetch_one(Bot::global().postgres())
        .await?
        .ignore_self_reacts;

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
                            .title(BOARDS_TITLE)
                            .description(format!(
                                "{help_text}\n\nCurrently, self-reacts are: **{}**",
                                if ignoring { "Ignored" } else { "Not Ignored" }
                            ))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(interactions[0].id.to_string())
                            .style(ButtonStyle::Success)
                            .label("Ignore"),
                        CreateButton::new(interactions[1].id.to_string())
                            .style(ButtonStyle::Danger)
                            .label("Don't ignore"),
                        CreateButton::new(interactions[2].id.to_string())
                            .style(ButtonStyle::Secondary)
                            .label("Skip"),
                    ])]),
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct ChangeIgnoreSelfReacts;
#[async_trait::async_trait]
impl ConfigStage for ChangeIgnoreSelfReacts {
    fn key(&self) -> (&'static str, &'static str) {
        ("boards", "change_ignore_self_reacts")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        Some(("boards", "ignore_self_reacts"))
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        entry: &ConfigEntry,
        data: (&ConfigInteraction, bool),
    ) -> ResponseResult {
        let context = ctx.get_populated_context()?;
        let ConfigInteraction::Boards { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let BoardsStage::ChangeIgnoreSelfReacts {
            channel_id,
            is_ignoring,
        } = stage
        else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        sqlx::query!(
            "UPDATE boards SET ignore_self_reacts = $1 WHERE guild_id = $2 AND channel_id = $3",
            *is_ignoring,
            context.guild.as_i64(),
            *channel_id
        )
        .execute(Bot::global().postgres())
        .await?;

        return advance_to(
            EditSettingsOrEmotes,
            ctx,
            entry,
            (
                &ConfigInteraction::Boards {
                    stage: BoardsStage::EditSettingsOrEmotes {
                        channel_id: Some(*channel_id),
                    },
                },
                data.1,
            ),
        )
        .await;
    }
}

#[derive(Debug)]
pub struct Emotes;

impl Emotes {
    const MAX_EMOTES_PER_PAGE: usize = 25;
    fn max_pages(emote_count: usize) -> usize {
        (emote_count / Self::MAX_EMOTES_PER_PAGE).max(1)
    }

    fn generate_interactions(
        user: User,
        single_category: bool,
        channel_id: i64,
        emotes: &[String],
        current_page: usize,
        max_pages: usize,
        edit_mode: Option<EditMode>,
    ) -> (Vec<Interaction>, Vec<CreateActionRow>) {
        let mut components = vec![];
        let mut interactions = vec![];

        if let Some(edit_mode) = edit_mode {
            interactions.append(&mut vec![
                boards_interaction_builder(
                    user,
                    BoardsStage::Emotes {
                        channel_id,
                        page: current_page,
                        emotes: Some(emotes.to_owned()),
                    },
                    single_category,
                )
                .build(),
                boards_interaction_builder(
                    user,
                    BoardsStage::Emotes {
                        channel_id,
                        page: current_page,
                        emotes: None,
                    },
                    single_category,
                )
                .build(),
            ]);

            if edit_mode == EditMode::Remove {
                interactions.push(
                    boards_interaction_builder(
                        user,
                        BoardsStage::RemoveEmote {
                            channel_id,
                            page: current_page,
                            emotes: emotes.to_owned(),
                        },
                        single_category,
                    )
                    .build(),
                );
            }

            if edit_mode == EditMode::Remove {
                components.push(CreateActionRow::SelectMenu(CreateSelectMenu::new(
                    interactions[2].id.to_string(),
                    CreateSelectMenuKind::String {
                        options: (0..emotes.len())
                            .map(|index| {
                                CreateSelectMenuOption::new(
                                    format!(
                                        "Remove {}{} emote",
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

            components.append(&mut vec![CreateActionRow::Buttons(vec![
                CreateButton::new(interactions[0].id.to_string())
                    .style(ButtonStyle::Secondary)
                    .label("Cancel edit"),
                CreateButton::new(interactions[1].id.to_string())
                    .style(ButtonStyle::Danger)
                    .label("Revert"),
            ])]);
        } else {
            interactions.append(&mut vec![
                boards_interaction_builder(
                    user,
                    BoardsStage::Emotes {
                        channel_id,
                        page: current_page.saturating_sub(1),
                        emotes: Some(emotes.to_owned()),
                    },
                    single_category,
                )
                .build(),
                boards_interaction_builder(
                    user,
                    BoardsStage::Emotes {
                        channel_id,
                        page: current_page + 1,
                        emotes: Some(emotes.to_owned()),
                    },
                    single_category,
                )
                .build(),
                boards_interaction_builder(
                    user,
                    BoardsStage::AddEmote {
                        channel_id,
                        page: current_page,
                        emotes: emotes.to_owned(),
                    },
                    single_category,
                )
                .build(),
                boards_interaction_builder(
                    user,
                    BoardsStage::RemoveEmote {
                        channel_id,
                        page: current_page,
                        emotes: emotes.to_owned(),
                    },
                    single_category,
                )
                .build(),
                boards_interaction_builder(
                    user,
                    BoardsStage::SaveEmotes {
                        channel_id,
                        emotes: emotes.to_owned(),
                    },
                    single_category,
                )
                .build(),
                boards_interaction_builder(
                    user,
                    BoardsStage::Emotes {
                        channel_id,
                        page: current_page,
                        emotes: None,
                    },
                    single_category,
                )
                .build(),
            ]);
            components.append(&mut vec![
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[0].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("◀".to_string()))
                        .disabled(current_page == 0),
                    CreateButton::new(interactions[1].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("▶".to_string()))
                        .disabled((current_page + 1) == max_pages),
                    CreateButton::new(interactions[2].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("➕".to_string())),
                    CreateButton::new(interactions[3].id.to_string())
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("➖".to_string()))
                        .disabled(emotes.is_empty()),
                    CreateButton::new(interactions[4].id.to_string())
                        .emoji('✅')
                        .style(ButtonStyle::Success),
                ]),
                CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[5].id.to_string())
                        .style(ButtonStyle::Danger)
                        .label("Revert"),
                ]),
            ]);
        }

        (interactions, components)
    }

    async fn generate_message(
        user: User,
        single_category: bool,
        channel_id: i64,
        emotes: &[String],
        current_page: usize,
        edit_mode: Option<EditMode>,
    ) -> Result<Response, ResponseError> {
        let mut emotes = emotes.to_vec();
        emotes.sort();

        let fields = emotes
            .iter()
            .enumerate()
            .map(|(index, emote)| (format!("#{}", index + 1), format!("**{emote}**"), true))
            .collect::<Vec<_>>();

        let start = current_page * Emotes::MAX_EMOTES_PER_PAGE;
        let mut end = start + Emotes::MAX_EMOTES_PER_PAGE;
        if end > fields.len() {
            end = fields.len();
        }
        let fields = fields[start..end].to_vec();
        let max_pages = Emotes::max_pages(emotes.len());
        let (interactions, components) = Self::generate_interactions(
            user,
            single_category,
            channel_id,
            &emotes,
            current_page,
            max_pages,
            edit_mode.clone(),
        );

        Bot::global()
            .interaction_state()
            .register(interactions.clone())
            .await?;

        let mut embed = CreateEmbed::new()
            .title(format!(
                "Board Emotes - Page {}/{}",
                current_page + 1,
                max_pages
            ))
            .description(format!("Editing emotes for <#{channel_id}>"))
            .fields(fields)
            .color(EMBED_COLOR);

        if let Some(edit_mode) = edit_mode
            && edit_mode == EditMode::Add
        {
            embed = embed.footer(CreateEmbedFooter::new(
                "Add an emote by reacting to this message",
            ));
        }

        Ok(Response::new().embed(embed).components(components))
    }
}

#[async_trait::async_trait]
impl ConfigStage for Emotes {
    fn key(&self) -> (&'static str, &'static str) {
        ("boards", "emotes")
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

        let ConfigInteraction::Boards { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let (channel_id, emotes, page) = match stage {
            BoardsStage::Emotes {
                channel_id,
                emotes,
                page,
            } => (channel_id, emotes, page),
            BoardsStage::AddEmote {
                channel_id,
                emotes,
                page,
            }
            | BoardsStage::RemoveEmote {
                channel_id,
                emotes,
                page,
            } => (channel_id, &Some(emotes.clone()), page),
            _ => {
                return Err(ResponseError::Execution(ExecutionError::Internal(
                    InternalError::InvalidInteractionType,
                )));
            }
        };

        let emotes = match emotes {
            Some(emotes) => emotes.clone(),
            None => sqlx::query!(
                "SELECT emote FROM board_emotes WHERE guild_id = $1 AND channel_id = $2",
                context.guild.as_i64(),
                channel_id
            )
            .fetch_all(Bot::global().postgres())
            .await?
            .iter()
            .map(|s| s.emote.clone())
            .collect::<Vec<_>>(),
        };

        entry
            .reply(
                ctx,
                Emotes::generate_message(context.user, data.1, *channel_id, &emotes, *page, None)
                    .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct AddEmote;
#[async_trait::async_trait]
impl ConfigStage for AddEmote {
    fn key(&self) -> (&'static str, &'static str) {
        ("boards", "add_emote")
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
        let ConfigInteraction::Boards { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let BoardsStage::AddEmote {
            channel_id,
            page,
            emotes,
        } = stage
        else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let message = entry
            .reply(
                ctx,
                Emotes::generate_message(
                    context.user,
                    data.1,
                    *channel_id,
                    emotes,
                    *page,
                    Some(EditMode::Add),
                )
                .await?,
            )
            .await?;

        let reaction_collector = message
            .await_reaction(context.ctx)
            .author_id(context.user.as_serenity_id())
            .timeout(std::time::Duration::new(300, 0));

        if let Some(reaction) = reaction_collector.await {
            let emote = match &reaction.emoji {
                ReactionType::Custom {
                    animated: _,
                    id,
                    name,
                } => {
                    let Some(name) = name else {
                        return Err(ResponseError::Execution(ExecutionError::Input(
                            InputError::InvalidEmote,
                        )));
                    };

                    let _ = context
                        .ctx
                        .http
                        .get_emoji(context.guild.as_serenity_id(), *id)
                        .await
                        .map_err(|_| {
                            ResponseError::Execution(ExecutionError::Input(
                                InputError::EmoteNotInServer,
                            ))
                        })?;

                    format!("<:{}:{}>", name, id.get())
                }
                ReactionType::Unicode(character) => character.to_string(),
                _ => {
                    return Err(ResponseError::Execution(ExecutionError::Input(
                        InputError::InvalidEmote,
                    )));
                }
            };

            reaction.delete(&context.ctx).await?;

            let mut emotes = emotes.clone();
            emotes.push(emote);

            return advance_to(
                Emotes,
                ctx,
                entry,
                (
                    &ConfigInteraction::Boards {
                        stage: BoardsStage::AddEmote {
                            channel_id: *channel_id,
                            page: *page,
                            emotes,
                        },
                    },
                    data.1,
                ),
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
pub struct RemoveEmote;
#[async_trait::async_trait]
impl ConfigStage for RemoveEmote {
    fn key(&self) -> (&'static str, &'static str) {
        ("boards", "remove_emote")
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
        let ConfigInteraction::Boards { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let BoardsStage::RemoveEmote {
            channel_id,
            page,
            emotes,
        } = stage
        else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        if let ComponentInteractionDataKind::StringSelect { values } = &entry.component()?.data.kind
        {
            let index = values
                .first()
                .ok_or_else(|| {
                    ResponseError::Execution(ExecutionError::Input(InputError::NoEmoteSelected))
                })?
                .parse::<usize>()
                .map_err(|_| {
                    ResponseError::Execution(ExecutionError::Input(InputError::InvalidEmote))
                })?;

            let mut emotes = emotes.clone();
            emotes.remove(index);

            return advance_to(
                Emotes,
                ctx,
                entry,
                (
                    &ConfigInteraction::Boards {
                        stage: BoardsStage::RemoveEmote {
                            channel_id: *channel_id,
                            page: *page,
                            emotes,
                        },
                    },
                    data.1,
                ),
            )
            .await;
        }

        entry
            .reply(
                ctx,
                Emotes::generate_message(
                    context.user,
                    data.1,
                    *channel_id,
                    emotes,
                    *page,
                    Some(EditMode::Remove),
                )
                .await?,
            )
            .await
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct SaveEmotes;
#[async_trait::async_trait]
impl ConfigStage for SaveEmotes {
    fn key(&self) -> (&'static str, &'static str) {
        ("boards", "save_emotes")
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
        let ConfigInteraction::Boards { stage } = data.0 else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let BoardsStage::SaveEmotes { channel_id, emotes } = stage else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };

        let mut tx = Bot::global().postgres().begin().await?;
        sqlx::query!(
            "DELETE FROM board_emotes WHERE guild_id = $1 AND channel_id = $2",
            context.guild.as_i64(),
            channel_id
        )
        .execute(&mut *tx)
        .await?;

        for emote in emotes {
            sqlx::query!(
                "INSERT INTO board_emotes (guild_id, channel_id, emote) VALUES ($1, $2, $3)",
                context.guild.as_i64(),
                channel_id,
                emote
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        advance_to(
            EditSettingsOrEmotes,
            ctx,
            entry,
            (
                &ConfigInteraction::Boards {
                    stage: BoardsStage::EditSettingsOrEmotes {
                        channel_id: Some(*channel_id),
                    },
                },
                data.1,
            ),
        )
        .await
    }
}
