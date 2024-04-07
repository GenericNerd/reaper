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

pub struct RemoveCommand;

#[async_trait::async_trait]
impl Command for RemoveCommand {
    fn name(&self) -> &'static str {
        "remove"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("remove")
            .dm_permission(false)
            .description("Remove an action")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "uuid",
                    "The UUID of the action",
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

        if !ctx.user_permissions.contains(&Permission::ModerationRemove) {
            return Err(ResponseError::Execution(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::ModerationReason.to_string())),
            ));
        }

        let options = Options {
            options: cmd.data.options(),
        };

        let Some(id) = options.get_string("uuid").into_owned() else {
            return Err(ResponseError::Execution(
                "UUID not provided!",
                Some("A UUID must be provided before continuing!".to_string()),
            ));
        };

        if let Err(err) = sqlx::query!("DELETE FROM actions WHERE id = $1", id)
            .execute(&handler.main_database)
            .await
        {
            error!("Could not remove action, failed with error: {:?}", err);
            return Err(ResponseError::Execution(
                "Could not remove action",
                Some("The action could not be removed. Please try again later.".to_string()),
            ));
        };

        let reply = ctx.reply(
            cmd,
            Response::new().embed(
                CreateEmbed::new()
                    .title("Action removed")
                    .description(format!("The action with ID `{id}` has been removed"))
                    .color(0x2e4045)
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
                        .embed(CreateEmbed::new().title("Action removed").description(format!("The action with ID `{id}` has been removed")).footer(CreateEmbedFooter::new(format!("Action removed | UUID: {id}"))).color(0x2e4045))
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
