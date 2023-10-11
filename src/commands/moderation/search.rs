use std::{collections::HashMap, time::Duration};

use inflections::Inflect;
use serenity::{
    all::{
        ButtonStyle, CommandInteraction, CommandOptionType, ComponentInteractionDataKind,
        ReactionType, User,
    },
    builder::{
        CreateActionRow, CreateButton, CreateCommand, CreateCommandOption, CreateEmbed,
        CreateEmbedFooter, CreateInteractionResponse,
    },
    futures::StreamExt,
};
use tracing::error;

use crate::{
    common::options::Options,
    models::{
        actions::Action,
        actions::{ActionType, DatabaseAction},
        command::{
            Command, CommandContext, CommandContextReply, InteractionContext,
            InteractionContextReply,
        },
        handler::Handler,
        permissions::Permission,
        response::{Response, ResponseError, ResponseResult},
    },
};

fn generate_search_response(
    user: &User,
    actions: &HashMap<u8, Action>,
    page: u8,
    expired: bool,
    time: &std::time::Instant,
) -> Response {
    if actions.is_empty() {
        return Response::new().embed(
            CreateEmbed::new()
                .title(format!("{}'s history", user.name))
                .description("No actions found"),
        );
    }
    let Some(action) = actions.get(&(page - 1)) else {
        return Response::new().embed(
            CreateEmbed::new()
                .title(format!("{}'s history", user.name))
                .description("Invalid page"),
        );
    };
    Response::new()
        .embed(
            CreateEmbed::new()
                .title(format!(
                    "{}'s {}history",
                    user.name,
                    if expired { "" } else { "active " }
                ))
                .description(format!("<@{}>", user.id.get()))
                .field(
                    "Type",
                    format!(
                        "{} {}",
                        action.action_type.to_string().to_title_case(),
                        if action.active { "" } else { "(Expired)" }
                    ),
                    true,
                )
                .field("Reason", action.reason.to_string(), true)
                .field("Moderator", format!("<@{}>", action.moderator_id), false)
                .field(
                    "Issued at",
                    format!("<t:{}:F>", action.created_at.unix_timestamp()),
                    true,
                )
                .field(
                    "Expires at",
                    match action.expiry {
                        Some(expiry) => format!("<t:{}:F>", expiry.unix_timestamp()),
                        None => "Never".to_string(),
                    },
                    true,
                )
                .color(if action.active {
                    match action.action_type {
                        ActionType::Strike => 0xeb966d,
                        ActionType::Mute => 0x2e4045,
                        ActionType::Kick => 0x000080,
                        ActionType::Ban => 0xf54029,
                    }
                } else {
                    match action.action_type {
                        ActionType::Strike => 0xbd7857,
                        ActionType::Mute => 0x182124,
                        ActionType::Kick => 0x000054,
                        ActionType::Ban => 0xba2f1e,
                    }
                })
                .footer(CreateEmbedFooter::new(format!(
                    "Page {}/{} | Total execution time: {:?}",
                    page,
                    actions.len(),
                    time.elapsed()
                ))),
        )
        .components(vec![CreateActionRow::Buttons(vec![
            CreateButton::new("previous")
                .style(ButtonStyle::Primary)
                .emoji(ReactionType::Unicode("◀".to_string()))
                .disabled(page - 1 == 0),
            CreateButton::new("next")
                .style(ButtonStyle::Primary)
                .emoji(ReactionType::Unicode("▶".to_string()))
                .disabled(page - 1 == u8::try_from(actions.len()).unwrap() - 1),
            CreateButton::new("close")
                .emoji(ReactionType::Unicode("✅".to_string()))
                .style(ButtonStyle::Success),
            CreateButton::new("uuid")
                .label("Get UUID")
                .style(ButtonStyle::Secondary),
        ])])
}

pub struct SearchCommand;

#[async_trait::async_trait]
impl Command for SearchCommand {
    fn name(&self) -> &'static str {
        "search"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("search")
            .dm_permission(false)
            .description("Searches for moderation history")
            .add_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "The user to search")
                    .required(false),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Boolean,
                    "expired",
                    "Whether to include expired actions",
                )
                .required(false),
            )
    }

    async fn router(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> ResponseResult {
        let mut start = std::time::Instant::now();

        let options = Options {
            options: cmd.data.options(),
        };

        let user = match options.get_user("user").into_owned() {
            Some(user) => {
                if user == cmd.user {
                    cmd.user.clone()
                } else {
                    user
                }
            }
            None => cmd.user.clone(),
        };

        let expired = options.get_boolean("expired").unwrap_or(false);

        let permission_required = if user == cmd.user {
            if expired {
                Permission::ModerationSearchSelfExpired
            } else {
                Permission::ModerationSearchSelf
            }
        } else if expired {
            Permission::ModerationSearchOthersExpired
        } else {
            Permission::ModerationSearchOthers
        };

        if !ctx.user_permissions.contains(&permission_required) {
            return Err(ResponseError::ExecutionError(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", permission_required.to_string())),
            ));
        }

        let actions = match if expired {
            sqlx::query_as!(DatabaseAction, "SELECT * FROM actions WHERE user_id = $1 AND guild_id = $2 ORDER BY created_at DESC", user.id.get() as i64, cmd.guild_id.unwrap().get() as i64).fetch_all(&handler.main_database).await
        } else {
            sqlx::query_as!(DatabaseAction, "SELECT * FROM actions WHERE user_id = $1 AND guild_id = $2 AND active=true ORDER BY created_at DESC", user.id.get() as i64, cmd.guild_id.unwrap().get() as i64).fetch_all(&handler.main_database).await
        } {
            Ok(db_actions) => {
                let mut actions = HashMap::new();
                for (index, db_action) in db_actions.iter().enumerate() {
                    actions.insert(
                        u8::try_from(index).unwrap(),
                        Action::from(db_action.clone()),
                    );
                }
                actions
            }
            Err(_) => {
                return Err(ResponseError::ExecutionError(
                    "Failed to fetch actions",
                    Some("Please contact the developer for assistance".to_string()),
                ))
            }
        };

        let mut page = 1;
        let message = ctx
            .reply_get_message(
                cmd,
                generate_search_response(&user, &actions, page, expired, &start),
            )
            .await?;

        let mut interaction_stream = message
            .await_component_interactions(&ctx.ctx)
            .timeout(Duration::new(60 * 60 * 24, 0))
            .stream();
        while let Some(interaction) = interaction_stream.next().await {
            let interaction_context =
                InteractionContext::new(handler, ctx.ctx.clone(), &interaction).await;

            start = std::time::Instant::now();
            match interaction_context.interaction.data.kind {
                ComponentInteractionDataKind::Button => {}
                _ => continue,
            }
            if interaction_context.interaction.user.id != cmd.user.id {
                let permission_required = if user == interaction_context.interaction.user {
                    if expired {
                        Permission::ModerationSearchSelfExpired
                    } else {
                        Permission::ModerationSearchSelf
                    }
                } else if expired {
                    Permission::ModerationSearchOthersExpired
                } else {
                    Permission::ModerationSearchOthers
                };

                if !interaction_context
                    .user_permissions
                    .contains(&permission_required)
                {
                    if let Err(err) = interaction_context.error_message(
                        ResponseError::ExecutionError(
                            "You do not have permission to do this!",
                            Some(format!(
                                "You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.",
                                permission_required.to_string()
                            ))
                        )).await {
                        error!("Failed to reply to command interaction with error: {:?}", err);
                    }
                    continue;
                }
            }
            let mut update_required = false;
            match interaction_context.interaction.data.custom_id.as_str() {
                "previous" => {
                    if page > 1 {
                        page -= 1;
                        update_required = true;
                    }
                }
                "next" => {
                    if page < u8::try_from(actions.len()).unwrap() {
                        page += 1;
                        update_required = true;
                    }
                }
                "uuid" => {
                    let action = actions.get(&(page - 1)).unwrap();
                    if let Err(err) = interaction_context
                        .reply(Response::new().content(action.get_id()).ephemeral(true))
                        .await
                    {
                        error!("Failed to send action UUID with error: {:?}", err);
                    }
                }
                "close" => {
                    if let Err(err) = cmd.delete_response(&ctx.ctx.http).await {
                        error!(
                            "Failed to delete command interaction response with error: {:?}",
                            err
                        );
                    }
                }
                _ => {}
            }
            if update_required {
                ctx.reply(
                    cmd,
                    generate_search_response(&user, &actions, page, expired, &start),
                )
                .await?;
                if let Err(err) = interaction_context
                    .interaction
                    .create_response(&ctx.ctx.http, CreateInteractionResponse::Acknowledge)
                    .await
                {
                    error!(
                        "Failed to acknowledge interaction response with error: {:?}",
                        err
                    );
                }
            }
        }
        Ok(())
    }
}
