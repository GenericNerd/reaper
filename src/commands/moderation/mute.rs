use std::sync::atomic::{AtomicBool, Ordering};

use serenity::{
    all::{ChannelId, CommandInteraction, CommandOptionType, GuildId, RoleId, UserId},
    builder::{CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateMessage},
};
use tracing::debug;

use crate::{
    common::{
        duration::Duration,
        logging::{get_log_channel, LogType},
        options::Options,
    },
    database::postgres::guild::get_moderation_config,
    models::{
        actions::{Action, ActionDatabaseInsert, ActionType},
        command::{Command, CommandContext, CommandContextReply},
        config::LoggingConfig,
        handler::Handler,
        permissions::Permission,
        response::{Response, ResponseError, ResponseResult},
    },
};

impl Handler {
    pub async fn mute_user(
        &self,
        ctx: &CommandContext,
        guild_id: i64,
        user_id: i64,
        reason: String,
        moderator_id: Option<i64>,
        duration: Duration,
    ) -> Result<ActionDatabaseInsert, ResponseError> {
        let start = std::time::Instant::now();

        let mute_role = match get_moderation_config(self, guild_id).await {
            Some(config) => match config.mute_role {
                Some(id) => RoleId::new(id as u64),
                None => {
                    return Err(ResponseError::Execution(
                        "Could not find mute role!",
                        Some(
                            "Please contact your server administrator to configure a mute role"
                                .to_string(),
                        ),
                    ));
                }
            },
            None => {
                return Err(ResponseError::Execution(
                    "Could not find a moderation configuration!",
                    Some(
                        "Please contact your server administrator to configure the server moderation"
                            .to_string(),
                    ),
                ));
            }
        };

        let moderator_id = match moderator_id {
            Some(mod_id) => mod_id,
            None => ctx.ctx.cache.current_user().id.get() as i64,
        };

        debug!(
            "Gathered all required data to mute in {:?}",
            start.elapsed()
        );

        let action = Action::new(
            ActionType::Mute,
            user_id,
            moderator_id,
            guild_id,
            reason,
            Some(duration),
        );

        action.insert(self).await?;

        let fields = vec![
            ("Moderator", format!("<@{}>", action.moderator_id), true),
            ("Reason", action.reason.to_string(), true),
            (
                "Expires",
                format!("<t:{}:F>", action.expiry.unwrap().unix_timestamp()),
                true,
            ),
        ];

        let action_insert = ActionDatabaseInsert {
            action: action.clone(),
            dm_notified: AtomicBool::new(false),
        };

        let log_message = match sqlx::query_as!(
            LoggingConfig,
            "SELECT log_actions, log_messages, log_voice, log_channel, log_action_channel, log_message_channel, log_voice_channel FROM logging_configuration WHERE guild_id = $1",
            guild_id
        )
        .fetch_one(&self.main_database)
        .await {
            Ok(config) => {
                get_log_channel(&config, &LogType::Action)
                    .map(|channel| {
                        ChannelId::new(channel as u64)
                            .send_message(
                                &ctx.ctx,
                                CreateMessage::new()
                                    .embed(CreateEmbed::new().title("Mute issued").description(format!("<@{}> has been muted", action.user_id)).fields(fields.clone()).footer(CreateEmbedFooter::new(format!("User {} muted | UUID: {}", action.user_id, action.get_id()))).color(0x2e4045))
                            )
                    })
            },
            Err(_) => None
        };

        let dm_channel = UserId::new(user_id as u64).create_dm_channel(&ctx.ctx.http);

        let mute_role_future = ctx.ctx.http.add_member_role(
            GuildId::new(guild_id as u64),
            UserId::new(user_id as u64),
            mute_role,
            Some(&action.reason),
        );

        if let Ok(channel) = match log_message {
            Some(log_future) => tokio::join!(log_future, mute_role_future, dm_channel).2,
            None => tokio::join!(mute_role_future, dm_channel).1,
        } {
            if channel
                .send_message(
                    &ctx.ctx,
                    CreateMessage::new().embed(
                        CreateEmbed::new()
                            .title("Muted!")
                            .description(match GuildId::new(guild_id as u64).name(&ctx.ctx) {
                                Some(guild_name) => {
                                    format!("You've been muted in {guild_name}")
                                }
                                None => "A server has muted you".to_string(),
                            })
                            .fields(fields)
                            .footer(CreateEmbedFooter::new(format!(
                                "If you wish to appeal, please refer to the following action ID: {}",
                                action.get_id()
                            )))
                            .color(0x2e4045),
                    ),
                )
                .await
                .is_ok() {
                    action_insert.dm_notified.store(true, Ordering::Relaxed);
                }
        };

        debug!("Completed mute action in {:?}", start.elapsed());

        Ok(action_insert)
    }
}

pub struct MuteCommand;

#[async_trait::async_trait]
impl Command for MuteCommand {
    fn name(&self) -> &'static str {
        "mute"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("mute")
            .dm_permission(false)
            .description("Mute a user from the server")
            .add_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "The user to mute")
                    .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "reason",
                    "The reason for the mute",
                )
                .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "duration",
                    "The duration of the mute",
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

        if !ctx.user_permissions.contains(&Permission::ModerationMute) {
            return Err(ResponseError::Execution(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::ModerationMute.to_string())),
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
                "You cannot mute yourself!",
                Some("You cannot mute yourself, that would be silly.".to_string()),
            ));
        }
        let Some(reason) = options.get_string("reason").into_owned() else {
            return Err(ResponseError::Execution(
                "No reason provided!",
                Some("Please provide a reason for the mute.".to_string()),
            ));
        };
        let Some(duration) = options
            .get_string("duration")
            .into_owned()
            .as_deref()
            .map(Duration::new)
        else {
            return Err(ResponseError::Execution(
                "Invalid or no duration provided",
                Some(
                    "The duration value was either not provided, or was not a valid duration."
                        .to_string(),
                ),
            ));
        };

        let action = handler
            .mute_user(
                ctx,
                ctx.guild.id.get() as i64,
                user.id.get() as i64,
                reason,
                Some(cmd.user.id.get() as i64),
                duration,
            )
            .await?;

        ctx.reply(
            cmd,
            Response::new().embed(
                CreateEmbed::new()
                    .title("User muted")
                    .description(if action.dm_notified.load(Ordering::Relaxed) {
                        format!("<@{}> has been muted", user.id.get())
                    } else {
                        format!(
                            "<@{}> has been muted\n*<@{}> could not be notified*",
                            user.id.get(),
                            user.id.get()
                        )
                    })
                    .field("Reason", action.action.reason.to_string(), true)
                    .field("Moderator", format!("<@{}>", cmd.user.id.get()), true)
                    .field(
                        "Expires",
                        match action.action.expiry {
                            Some(duration) => format!("<t:{}:F>", duration.unix_timestamp()),
                            None => "Never".to_string(),
                        },
                        true,
                    )
                    .footer(CreateEmbedFooter::new(format!(
                        "UUID: {} | Total execution time: {:?}",
                        action.action.get_id(),
                        start.elapsed()
                    )))
                    .color(0x2e4045),
            ),
        )
        .await
    }
}
