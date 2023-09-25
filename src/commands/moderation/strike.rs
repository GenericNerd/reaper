use objectid::ObjectId;
use serenity::{
    all::{ChannelId, CommandInteraction, CommandOptionType, UserId},
    builder::{CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateMessage},
};
use tracing::{debug, error};

use crate::{
    common::{duration::Duration, options::Options},
    database::postgres::{
        actions::get_active_strikes,
        guild::{get_moderation_config, get_strike_escalations},
    },
    models::{
        actions::{Action, ActionType},
        command::{Command, CommandContext, CommandContextReply},
        handler::Handler,
        permissions::Permission,
        response::{Response, ResponseError, ResponseResult},
    },
};

pub struct StrikeCommand;

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
                let offset = duration.to_timestamp().unwrap();
                Some(time::PrimitiveDateTime::new(offset.date(), offset.time()))
            }
        } else {
            let moderation_config = get_moderation_config(self, guild_id).await;
            let duration = match moderation_config {
                Some(config) => match config.default_strike_duration {
                    Some(duration) => {
                        let offset = Duration::new(&duration).to_timestamp().unwrap();
                        Some(time::PrimitiveDateTime::new(offset.date(), offset.time()))
                    }
                    None => None,
                },
                None => None,
            };
            if duration.is_none() {
                return Err(ResponseError::ExecutionError("No duration was provided", Some("A duration was not provided and this server has not configured a default strike duration".to_string())));
            }
            duration
        };

        let moderator_id = if moderator_id.is_none() {
            ctx.ctx.cache.current_user().id.0.get() as i64
        } else {
            moderator_id.unwrap()
        };

        debug!(
            "Gathered all required data to strike in {:?}",
            start.elapsed()
        );

        let guild_escalations = get_strike_escalations(self, guild_id).await;

        if guild_escalations.is_empty() {
            let strike_count = get_active_strikes(self, guild_id, user_id).await.len();
            if let Some(escalation) = guild_escalations
                .iter()
                .find(|escalation| escalation.strike_count == strike_count as i64)
            {
                match escalation.action_type {
                    ActionType::Strike => {
                        return Err(ResponseError::ExecutionError(
                            "Strike escalation action type is strike!",
                            Some("This should not happen, please contact a developer.".to_string()),
                        ))
                    }
                    ActionType::Mute | ActionType::Kick | ActionType::Ban => {}
                }
            }
        }

        debug!(
            "Checked and completed strike escalation in {:?}",
            start.elapsed()
        );

        let action = Action {
            id: ObjectId::new().unwrap().to_string(),
            action_type: ActionType::Strike,
            user_id,
            moderator_id,
            guild_id,
            reason,
            active: true,
            expiry: duration,
        };

        let mut strike_action = StrikeAction {
            strike: action.clone(),
            escalation: None,
            dm_notified: false,
        };

        if let Err(err) = sqlx::query_unchecked!(
            "INSERT INTO actions (id, type, user_id, moderator_id, guild_id, reason, active, expiry) VALUES ($1, 'strike', $2, $3, $4, $5, $6, $7)",
            action.id,
            action.user_id,
            action.moderator_id,
            action.guild_id,
            action.reason,
            action.active,
            action.expiry
        ).execute(&self.main_database).await {
            error!("Failed to insert strike action into database: {}", err);
            return Err(ResponseError::ExecutionError(
                "Failed to insert strike action into database!",
                Some("Please contact the bot owner for assistance.".to_string()),
            ));
        }

        debug!(
            "Inserted strike action into database in {:?}",
            start.elapsed()
        );

        let log = ChannelId::new(823255446847881226).send_message(
            &ctx.ctx,
            CreateMessage::new().content("This is a test of logging"),
        );
        let dm_channel = UserId::new(user_id as u64).create_dm_channel(&ctx.ctx.http);

        if let Ok(channel) = tokio::join!(log, dm_channel).1 {
            if channel
                .send_message(
                    &ctx.ctx,
                    CreateMessage::new().content("You got striked lol"),
                )
                .await
                .is_ok()
            {
                strike_action.dm_notified = true;
            }
        };

        Ok(strike_action)
    }
}

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
            return Err(ResponseError::ExecutionError(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::PermissionsView.to_string())),
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
                ctx.guild.id.0.get() as i64,
                user.id.0.get() as i64,
                reason.clone(),
                Some(cmd.user.id.0.get() as i64),
                duration.clone(),
            )
            .await?;

        ctx.reply(
            cmd,
            Response::new().embed(
                CreateEmbed::new()
                    .title("Strike issued")
                    .description(if action.dm_notified {
                        format!("<@{}> was issued a strike", user.id.0.get())
                    } else {
                        format!(
                            "<@{}> was issued a strike\n*<@{}> could not be notified*",
                            user.id.0.get(),
                            user.id.0.get()
                        )
                    })
                    .field("Reason", reason, true)
                    .field("Moderator", format!("<@{}>", cmd.user.id.0.get()), true)
                    .field(
                        "Expires on",
                        match duration {
                            Some(duration) => match duration.to_timestamp() {
                                Some(timestamp) => {
                                    let now = time::OffsetDateTime::now_utc();
                                    format!(
                                        "<t:{}:F>",
                                        (timestamp.unix_timestamp() + now.unix_timestamp())
                                            - time::OffsetDateTime::now_utc().unix_timestamp()
                                    )
                                }
                                None => "Never".to_string(),
                            },
                            None => "Never".to_string(),
                        },
                        true,
                    )
                    .footer(CreateEmbedFooter::new(format!(
                        "UUID: {} | Total execution time: {:?}",
                        action.strike.id,
                        start.elapsed()
                    ))),
            ),
        )
        .await?;

        Ok(())
    }
}
