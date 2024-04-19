use serenity::{
    all::{ChannelId, CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateMessage},
};
use tracing::error;

use crate::{
    common::{
        duration::Duration,
        logging::{get_log_channel, LogType},
        options::Options,
    },
    models::{
        command::{Command, CommandContext, CommandContextReply},
        config::LoggingConfig,
        handler::Handler,
        permissions::Permission,
        response::{Response, ResponseError, ResponseResult},
    },
};

pub struct DurationCommand;

#[async_trait::async_trait]
impl Command for DurationCommand {
    fn name(&self) -> &'static str {
        "duration"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("duration")
            .dm_permission(false)
            .description("Change the duration of a specified action")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "duration",
                    "The new duration for this action",
                )
                .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "uuid",
                    "The UUID of the action",
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

        if !ctx
            .user_permissions
            .contains(&Permission::ModerationDuration)
        {
            return Err(ResponseError::Execution(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::ModerationDuration)),
            ));
        }

        let options = Options {
            options: cmd.data.options(),
        };

        let Some(duration) = options.get_string("duration").into_owned() else {
            return Err(ResponseError::Execution(
                "Duration not provided!",
                Some("A duration must be provided before continuing!".to_string()),
            ));
        };

        let id = match options.get_string("uuid").into_owned() {
            Some(id) => id,
            None => match sqlx::query!(
                "SELECT id FROM actions WHERE moderator_id = $1 ORDER BY created_at DESC",
                cmd.user.id.get() as i64
            )
            .fetch_one(&handler.main_database)
            .await
            {
                Ok(record) => record.id,
                Err(err) => {
                    error!(
                        "Could not find most recent moderator action, failed with error: {:?}",
                        err
                    );
                    return Err(ResponseError::Execution("Could not find most recent action", Some("Your most recent action could not be found. This could be because you've yet to issue someone with an action. Please provide a UUID with your next command".to_string())));
                }
            },
        };

        let expiry = Duration::new(duration.as_str()).to_timestamp().unwrap();

        if let Err(err) = sqlx::query!(
            "UPDATE actions SET expiry = $1 WHERE id = $2",
            time::PrimitiveDateTime::new(expiry.date(), expiry.time()),
            id
        )
        .execute(&handler.main_database)
        .await
        {
            error!(
                "Could not update action duration, failed with error: {:?}",
                err
            );
            return Err(ResponseError::Execution(
                "Could not update action duration",
                Some(
                    "The action duration could not be updated. Please try again later.".to_string(),
                ),
            ));
        };

        let reply = ctx.reply(
            cmd,
            Response::new().embed(
                CreateEmbed::new()
                    .title("Duration updated")
                    .description(format!(
                        "The duration of action `{id}` has been updated to <t:{}:F>",
                        expiry.unix_timestamp()
                    ))
                    .color(0x0abfd6)
                    .footer(CreateEmbedFooter::new(format!(
                        "Total execution time: {:?}",
                        start.elapsed()
                    ))),
            ),
        );

        let log = if let Ok(config) = sqlx::query_as!(
            LoggingConfig,
            "SELECT log_actions, log_messages, log_voice, log_channel, log_action_channel, log_message_channel, log_voice_channel FROM logging_configuration WHERE guild_id = $1",
            cmd.guild_id.unwrap().get() as i64
        )
        .fetch_one(&handler.main_database)
        .await {
            get_log_channel(&config, &LogType::Action).map(|channel| ChannelId::new(channel as u64)
                .send_message(
                    &ctx.ctx,
                    CreateMessage::new()
                        .embed(CreateEmbed::new().title("Duration updated").description(format!("The duration of action `{id}` will now expire on <t:{}:F>", expiry.unix_timestamp())).footer(CreateEmbedFooter::new(format!("Duration updated | UUID: {id}"))).color(0x0abfd6))
                ))
        } else {
            None
        };

        match log {
            Some(log_future) => tokio::join!(log_future, reply).1,
            None => reply.await,
        }
    }
}
