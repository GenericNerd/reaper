use serenity::{
    all::{ChannelId, CommandInteraction, CommandOptionType, GuildId, UserId},
    builder::{CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateMessage},
};
use tracing::debug;

use crate::{
    common::{
        duration::Duration,
        logging::{get_log_channel, LogType},
        options::Options,
    },
    database::postgres::{actions::get_active_strikes, guild::get_moderation_config},
    models::{
        actions::{Action, ActionType, DatabaseActionEscalation},
        command::{Command, CommandContext, CommandContextReply},
        config::LoggingConfig,
        handler::Handler,
        permissions::Permission,
        response::{Response, ResponseError, ResponseResult},
    },
};

pub struct StrikeAction {
    pub strike: Action,
    pub escalation: Option<Action>,
    pub dm_notified: bool,
}

impl Handler {
    pub async fn strike_user(
        &self,
        ctx: &CommandContext,
        guild_id: i64,
        user_id: i64,
        reason: String,
        moderator_id: Option<i64>,
        duration: Option<Duration>,
    ) -> Result<StrikeAction, ResponseError> {
        let start = std::time::Instant::now();

        let duration = if let Some(duration) = duration {
            if duration.permanent {
                None
            } else {
                Some(duration)
            }
        } else {
            let moderation_config = get_moderation_config(self, guild_id).await;
            let duration = match moderation_config {
                Some(config) => config
                    .default_strike_duration
                    .map(|duration| Duration::new(&duration)),
                None => None,
            };
            if duration.is_none() {
                return Err(ResponseError::Execution("No duration was provided", Some("A duration was not provided and this server has not configured a default strike duration".to_string())));
            }
            duration
        };

        let moderator_id = match moderator_id {
            Some(mod_id) => mod_id,
            None => ctx.ctx.cache.current_user().id.get() as i64,
        };

        debug!(
            "Gathered all required data to strike in {:?}",
            start.elapsed()
        );

        let action = Action::new(
            ActionType::Strike,
            user_id,
            moderator_id,
            guild_id,
            reason,
            duration,
        );

        let mut strike_action = StrikeAction {
            strike: action.clone(),
            escalation: None,
            dm_notified: false,
        };

        let guild_escalations = match sqlx::query_as!(
            DatabaseActionEscalation,
            "SELECT * FROM strike_escalations WHERE guild_id = $1",
            guild_id
        )
        .fetch_all(&self.main_database)
        .await
        {
            Ok(escalations) => escalations,
            Err(_) => vec![],
        };

        if !guild_escalations.is_empty() {
            let strike_count = get_active_strikes(self, guild_id, user_id).await.len();
            if let Some(escalation) = guild_escalations
                .iter()
                .find(|escalation| escalation.strike_count == (strike_count + 1) as i64)
            {
                match ActionType::from(escalation.action_type.as_str()) {
                    ActionType::Strike => {
                        return Err(ResponseError::Execution(
                            "Strike escalation action type is strike!",
                            Some("This should not happen, please contact a developer.".to_string()),
                        ))
                    }
                    ActionType::Kick => {
                        if let Ok(escalation) = self
                            .kick_user(
                                ctx,
                                guild_id,
                                user_id,
                                format!(
                                    "Strike escalation (reached {} strikes)",
                                    escalation.strike_count
                                ),
                                None,
                            )
                            .await
                        {
                            strike_action.escalation = Some(escalation.action);
                        };
                    }
                    ActionType::Mute => {
                        let Some(duration) = &escalation.action_duration else {
                            return Err(ResponseError::Execution(
                                "Strike escalation mute did not provide a duration",
                                Some(
                                    "This should not happen, please contact a developer."
                                        .to_string(),
                                ),
                            ));
                        };
                        if let Ok(escalation) = self
                            .mute_user(
                                ctx,
                                guild_id,
                                user_id,
                                format!(
                                    "Strike escalation (reached {} strikes)",
                                    escalation.strike_count
                                ),
                                None,
                                Duration::new(duration),
                            )
                            .await
                        {
                            strike_action.escalation = Some(escalation.action);
                        };
                    }
                    ActionType::Ban => {
                        if let Ok(escalation) = self
                            .ban_user(
                                ctx,
                                guild_id,
                                user_id,
                                format!(
                                    "Strike escalation (reached {} strikes)",
                                    escalation.strike_count
                                ),
                                None,
                                escalation
                                    .action_duration
                                    .as_ref()
                                    .map(|duration| Duration::new(duration)),
                            )
                            .await
                        {
                            strike_action.escalation = Some(escalation.action);
                        };
                    }
                }
            }
        }

        debug!(
            "Checked and completed strike escalation in {:?}",
            start.elapsed()
        );

        action.insert(self).await?;

        debug!(
            "Inserted strike action into database in {:?}",
            start.elapsed()
        );

        let fields = vec![
            ("Moderator", format!("<@{}>", action.moderator_id), true),
            ("Reason", action.reason.to_string(), true),
            (
                "Expires",
                format!("<t:{}:F>", action.expiry.unwrap().unix_timestamp()),
                true,
            ),
        ];

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
                                    .embed(CreateEmbed::new().title("Strike issued").description(format!("<@{}> has been issued a strike", action.user_id)).fields(fields.clone()).footer(CreateEmbedFooter::new(format!("User {} striked | UUID: {}", action.user_id, action.get_id()))).color(0xeb966d))
                            )
                    })
            },
            Err(_) => None
        };

        let dm_channel = UserId::new(user_id as u64).create_dm_channel(&ctx.ctx.http);

        if let Ok(channel) = match log_message {
            Some(log_future) => tokio::join!(log_future, dm_channel).1,
            None => dm_channel.await,
        } {
            strike_action.dm_notified = channel
                .send_message(
                    &ctx.ctx,
                    CreateMessage::new().embed(
                        CreateEmbed::new()
                            .title("Strike received")
                            .description(match GuildId::new(guild_id as u64).name(&ctx.ctx) {
                                Some(guild_name) => {
                                    format!("You've been issued a strike in {guild_name}")
                                }
                                None => "A server has issued you a strike".to_string(),
                            })
                            .fields(fields)
                            .footer(CreateEmbedFooter::new(format!(
                                "If you wish to appeal, please refer to the following action ID: {}",
                                action.get_id()
                            )))
                            .color(0xeb966d),
                    ),
                )
                .await
                .is_ok();
        };

        debug!("Completed strike action in {:?}", start.elapsed());

        Ok(strike_action)
    }
}

pub struct StrikeCommand;

#[async_trait::async_trait]
impl Command for StrikeCommand {
    fn name(&self) -> &'static str {
        "strike"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("strike")
            .dm_permission(false)
            .description("Strike a user in the server, effectively warning them")
            .add_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "The user to strike")
                    .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "reason",
                    "The reason for the strike",
                )
                .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "duration",
                    "The duration of the strike",
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
        let start = std::time::Instant::now();

        if !ctx.user_permissions.contains(&Permission::ModerationStrike) {
            return Err(ResponseError::Execution(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::ModerationStrike)),
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
                "You cannot strike yourself!",
                Some("You cannot strike yourself, that would be silly.".to_string()),
            ));
        }
        let Some(reason) = options.get_string("reason").into_owned() else {
            return Err(ResponseError::Execution(
                "No reason provided!",
                Some("Please provide a reason for the strike.".to_string()),
            ));
        };
        let duration = options
            .get_string("duration")
            .into_owned()
            .as_deref()
            .map(Duration::new);

        let action = handler
            .strike_user(
                ctx,
                ctx.guild.id.get() as i64,
                user.id.get() as i64,
                reason.clone(),
                Some(cmd.user.id.get() as i64),
                duration,
            )
            .await?;

        ctx.reply(
            cmd,
            Response::new().embed(
                CreateEmbed::new()
                    .title("Strike issued")
                    .description(if action.dm_notified {
                        format!("<@{}> was issued a strike", user.id.get())
                    } else {
                        format!(
                            "<@{}> was issued a strike\n*<@{}> could not be notified*",
                            user.id.get(),
                            user.id.get()
                        )
                    })
                    .field("Reason", reason, true)
                    .field("Moderator", format!("<@{}>", cmd.user.id.get()), true)
                    .field(
                        "Expires",
                        format!("<t:{}:F>", action.strike.expiry.unwrap().unix_timestamp()),
                        true,
                    )
                    .footer(CreateEmbedFooter::new(format!(
                        "UUID: {} | Total execution time: {:?}",
                        action.strike.get_id(),
                        start.elapsed()
                    )))
                    .color(0xeb966d),
            ),
        )
        .await
    }
}
