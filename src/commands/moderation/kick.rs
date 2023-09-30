use objectid::ObjectId;
use serenity::{
    all::{ChannelId, CommandInteraction, CommandOptionType, GuildId, UserId},
    builder::{CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateMessage},
};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, error};

use crate::{
    common::{
        logging::{get_log_channel, LogType},
        options::Options,
    },
    models::{
        actions::{Action, ActionInsert, ActionType},
        command::{Command, CommandContext, CommandContextReply},
        config::LoggingConfig,
        handler::Handler,
        permissions::Permission,
        response::{Response, ResponseError, ResponseResult},
    },
};

impl Handler {
    pub async fn kick_user(
        &self,
        ctx: &CommandContext,
        guild_id: i64,
        user_id: i64,
        reason: String,
        moderator_id: Option<i64>,
    ) -> Result<ActionInsert, ResponseError> {
        let start = std::time::Instant::now();

        let moderator_id = if moderator_id.is_none() {
            ctx.ctx.cache.current_user().id.0.get() as i64
        } else {
            moderator_id.unwrap()
        };

        debug!(
            "Gathered all required data to kick in {:?}",
            start.elapsed()
        );

        let action = Action {
            id: ObjectId::new().unwrap().to_string(),
            action_type: ActionType::Kick,
            user_id,
            moderator_id,
            guild_id,
            reason,
            active: false,
            expiry: None,
        };

        let fields = vec![
            ("Moderator", format!("<@{}>", action.moderator_id), true),
            ("Reason", action.reason.to_string(), true),
        ];

        let action_insert = ActionInsert {
            action: action.clone(),
            dm_notified: AtomicBool::new(false),
        };

        if let Ok(dm_channel) = UserId::new(user_id as u64)
            .create_dm_channel(&ctx.ctx.http)
            .await
        {
            if dm_channel
                .send_message(
                    &ctx.ctx,
                    CreateMessage::new().embed(
                        CreateEmbed::new()
                            .title("Kicked!")
                            .description(match GuildId::new(guild_id as u64).name(&ctx.ctx) {
                                Some(guild_name) => {
                                    format!("You've been kicked from {guild_name}")
                                }
                                None => "A server has kicked you".to_string(),
                            })
                            .fields(fields.clone())
                            .color(0x000080),
                    ),
                )
                .await
                .is_ok()
            {
                action_insert.dm_notified.store(true, Ordering::Relaxed);
            }
        };

        debug!("Attempted to send a DM in {:?}", start.elapsed());

        if let Err(err) = ctx
            .ctx
            .http
            .kick_member(
                GuildId::new(guild_id as u64),
                UserId::new(user_id as u64),
                Some(action.reason.as_str()),
            )
            .await
        {
            error!("Failed to kick user: {}", err);
            return Err(ResponseError::SerenityError(err));
        }

        if let Err(err) = sqlx::query_unchecked!(
            "INSERT INTO actions (id, type, user_id, moderator_id, guild_id, reason, active, expiry) VALUES ($1, 'kick', $2, $3, $4, $5, $6, $7)",
            action.id,
            action.user_id,
            action.moderator_id,
            action.guild_id,
            action.reason,
            action.active,
            action.expiry
        ).execute(&self.main_database).await {
            error!("Failed to insert kick action into database: {}", err);
            return Err(ResponseError::ExecutionError(
                "Failed to insert kick action into database!",
                Some("Please contact the bot owner for assistance.".to_string()),
            ));
        }

        debug!(
            "Inserted kick action into database in {:?}",
            start.elapsed()
        );

        if let Ok(config) = sqlx::query_as!(
            LoggingConfig,
            "SELECT log_actions, log_messages, log_voice, log_channel, log_action_channel, log_message_channel, log_voice_channel FROM logging_configuration WHERE guild_id = $1",
            guild_id
        )
        .fetch_one(&self.main_database)
        .await {
            if let Some(channel) = get_log_channel(&config, &LogType::Action) {
                if let Err(err) = ChannelId::new(channel as u64)
                    .send_message(
                        &ctx.ctx,
                        CreateMessage::new()
                            .embed(CreateEmbed::new().title("User kicked").description(format!("<@{}> has been kicked", action.user_id)).fields(fields).footer(CreateEmbedFooter::new(format!("User {} kicked | UUID: {}", action.user_id, action.id))).color(0x000080))
                ).await {
                    error!("Failed to send kick log message: {}", err);
                }
            }
        }

        Ok(action_insert)
    }
}

pub struct KickCommand;

#[async_trait::async_trait]
impl Command for KickCommand {
    fn name(&self) -> &'static str {
        "kick"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("kick")
            .dm_permission(false)
            .description("Kick a user from the server")
            .add_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "The user to kick")
                    .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "reason",
                    "The reason for the kick",
                )
                .required(true),
            )
    }

    async fn router(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> ResponseResult {
        let start = std::time::Instant::now();

        if !ctx.user_permissions.contains(&Permission::ModerationKick) {
            return Err(ResponseError::ExecutionError(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::ModerationKick.to_string())),
            ));
        }

        let options = Options {
            options: cmd.data.options(),
        };

        let Some(user) = options.get_user("user").into_owned() else {
            return Err(ResponseError::ExecutionError("No member found!", Some("The user option either was not provided, or this command was not ran in a guild. Both of these should not occur, if they do, please contact a developer.".to_string())));
        };
        let Some(reason) = options.get_string("reason").into_owned() else {
            return Err(ResponseError::ExecutionError(
                "No reason provided!",
                Some("Please provide a reason for the kick.".to_string()),
            ));
        };

        let action = handler
            .kick_user(
                ctx,
                ctx.guild.id.0.get() as i64,
                user.id.0.get() as i64,
                reason,
                Some(cmd.user.id.0.get() as i64),
            )
            .await?;

        ctx.reply(
            cmd,
            Response::new().embed(
                CreateEmbed::new()
                    .title("Kick issued")
                    .description(if action.dm_notified.load(Ordering::Relaxed) {
                        format!("<@{}> was kicked", user.id.0.get())
                    } else {
                        format!(
                            "<@{}> was kicked\n*<@{}> could not be notified*",
                            user.id.0.get(),
                            user.id.0.get()
                        )
                    })
                    .field("Reason", action.action.reason, true)
                    .field("Moderator", format!("<@{}>", cmd.user.id.0.get()), true)
                    .footer(CreateEmbedFooter::new(format!(
                        "UUID: {} | Total execution time: {:?}",
                        action.action.id,
                        start.elapsed()
                    )))
                    .color(0x000080),
            ),
        )
        .await
    }
}
