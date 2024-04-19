use serenity::{
    all::{ChannelId, CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateMessage},
};
use tracing::error;

use crate::{
    common::{
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

pub struct ReasonCommand;

#[async_trait::async_trait]
impl Command for ReasonCommand {
    fn name(&self) -> &'static str {
        "reason"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("reason")
            .dm_permission(false)
            .description("Change the reason of a specified action")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "uuid",
                    "The UUID of the action",
                )
                .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "reason",
                    "The new reason for this action",
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

        if !ctx.user_permissions.contains(&Permission::ModerationReason) {
            return Err(ResponseError::Execution(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::ModerationReason)),
            ));
        }

        let options = Options {
            options: cmd.data.options(),
        };

        let Some(reason) = options.get_string("reason").into_owned() else {
            return Err(ResponseError::Execution(
                "Reason not provided!",
                Some("A reason must be provided before continuing!".to_string()),
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

        if let Err(err) = sqlx::query!("UPDATE actions SET reason = $1 WHERE id = $2", reason, id)
            .execute(&handler.main_database)
            .await
        {
            error!(
                "Could not update action reason, failed with error: {:?}",
                err
            );
            return Err(ResponseError::Execution(
                "Could not update action reason",
                Some("The action reason could not be updated. Please try again later.".to_string()),
            ));
        };

        let reply = ctx.reply(
            cmd,
            Response::new().embed(
                CreateEmbed::new()
                    .title("Reason updated")
                    .description(format!(
                        "The reason of action `{id}` has been updated to {reason}"
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
                        .embed(CreateEmbed::new().title("Reason updated").description(format!("The reason of action `{id}` has been updated to {reason}")).footer(CreateEmbedFooter::new(format!("Reason updated | UUID: {id}"))).color(0x0abfd6))
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
