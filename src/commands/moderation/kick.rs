use serenity::{
    all::{ChannelId, CommandInteraction, CommandOptionType, GuildId, UserId},
    builder::{CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateMessage},
};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
use tracing::{debug, error};

use crate::{
    common::{
        logging::{get_log_channel, LogType},
        options::Options,
    },
    models::{
        actions::{Action, ActionDatabaseInsert, ActionType},
        command::{Command, CommandContext, CommandContextReply},
        config::LoggingConfig,
        handler::Handler,
        highest_role::get_highest_role,
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
    ) -> Result<ActionDatabaseInsert, ResponseError> {
        let start = Instant::now();

        let moderator_id = match moderator_id {
            Some(mod_id) => mod_id,
            None => ctx.ctx.cache.current_user().id.get() as i64,
        };

        debug!(
            "Gathered all required data to kick in {:?}",
            start.elapsed()
        );

        let action = Action::new(
            ActionType::Kick,
            user_id,
            moderator_id,
            guild_id,
            reason,
            None,
        );

        let fields = vec![
            ("Moderator", format!("<@{}>", action.moderator_id), true),
            ("Reason", action.reason.to_string(), true),
        ];

        let action_insert = ActionDatabaseInsert {
            action: action.clone(),
            dm_notified: AtomicBool::new(false),
        };

        if sqlx::query!("SELECT active FROM global_kills WHERE feature = 'commands.dm'")
            .fetch_one(&self.main_database)
            .await
            .unwrap()
            .active
            && ctx
                .ctx
                .http
                .get_member(GuildId::new(guild_id as u64), UserId::new(user_id as u64))
                .await
                .is_ok()
        {
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
        }

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
            return Err(ResponseError::Serenity(err));
        }

        action.insert(self).await?;

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
            if let Some(channel) = get_log_channel(self, &config, &LogType::Action).await {
                if let Err(err) = ChannelId::new(channel as u64)
                    .send_message(
                        &ctx.ctx,
                        CreateMessage::new()
                            .embed(CreateEmbed::new().title("User kicked").description(format!("<@{}> has been kicked", action.user_id)).fields(fields).footer(CreateEmbedFooter::new(format!("User {} kicked | UUID: {}", action.user_id, action.get_id()))).color(0x000080))
                ).await {
                    error!("Failed to send kick log message: {}", err);
                }
            }
        }

        debug!("Completed kick action in {:?}", start.elapsed());

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
        let start = Instant::now();

        if !ctx.user_permissions.contains(&Permission::ModerationKick) {
            return Err(ResponseError::Execution(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::ModerationKick)),
            ));
        }

        let options = Options {
            options: cmd.data.options(),
        };

        let Some(user) = options.get_user("user").into_owned() else {
            return Err(ResponseError::Execution("No member found!", Some("The user option either was not provided, or this command was not ran in a guild. Both of these should not occur, if they do, please contact a developer.".to_string())));
        };
        if user == cmd.user {
            return Err(ResponseError::Execution(
                "You cannot kick yourself!",
                Some("You cannot kick yourself, that would be silly.".to_string()),
            ));
        }
        let Some(reason) = options.get_string("reason").into_owned() else {
            return Err(ResponseError::Execution(
                "No reason provided!",
                Some("Please provide a reason for the kick.".to_string()),
            ));
        };

        let target_user_highest_role = get_highest_role(ctx, &user).await;
        if ctx.highest_role <= target_user_highest_role {
            return Err(ResponseError::Execution(
                "You cannot kick this user!",
                Some(
                    "You cannot kick a user with a role equal to or higher than yours.".to_string(),
                ),
            ));
        }

        let bot = ctx.ctx.cache.current_user().to_owned();
        let bot_highest_role = get_highest_role(ctx, &bot).await;
        if bot_highest_role <= target_user_highest_role {
            return Err(ResponseError::Execution(
                "Reaper cannot kick this user!",
                Some(
                    "Reaper cannot kick a user with a role equal to or higher than itself."
                        .to_string(),
                ),
            ));
        }

        let action = handler
            .kick_user(
                ctx,
                ctx.guild.id.get() as i64,
                user.id.get() as i64,
                reason,
                Some(cmd.user.id.get() as i64),
            )
            .await?;

        ctx.reply(
            cmd,
            Response::new().embed(
                CreateEmbed::new()
                    .title("Kick issued")
                    .description(if action.dm_notified.load(Ordering::Relaxed) {
                        format!("<@{}> was kicked", user.id.get())
                    } else {
                        format!(
                            "<@{}> was kicked\n*<@{}> could not be notified*",
                            user.id.get(),
                            user.id.get()
                        )
                    })
                    .field("Reason", action.action.reason.to_string(), true)
                    .field("Moderator", format!("<@{}>", cmd.user.id.get()), true)
                    .footer(CreateEmbedFooter::new(format!(
                        "UUID: {} | Total execution time: {:?}",
                        action.action.get_id(),
                        start.elapsed()
                    )))
                    .color(0x000080),
            ),
        )
        .await
    }
}
