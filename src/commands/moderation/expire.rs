use serenity::{
    all::{ChannelId, CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateMessage},
};
use std::time::Instant;
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

pub struct ExpireCommand;

#[async_trait::async_trait]
impl Command for ExpireCommand {
    fn name(&self) -> &'static str {
        "expire"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("expire")
            .dm_permission(false)
            .description("Expire an action")
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
        let start = Instant::now();

        if !ctx.user_permissions.contains(&Permission::ModerationExpire) {
            return Err(ResponseError::Execution(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::ModerationExpire)),
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

        if let Err(err) = sqlx::query!("UPDATE actions SET active = false WHERE id = $1", id)
            .execute(&handler.main_database)
            .await
        {
            error!("Could not expire action, failed with error: {:?}", err);
            return Err(ResponseError::Execution(
                "Could not expire action",
                Some("The action could not be expired. Please try again later.".to_string()),
            ));
        };

        let reply = ctx.reply(
            cmd,
            Response::new().embed(
                CreateEmbed::new()
                    .title("Action expired")
                    .description(format!(
                        "The action with ID `{id}` has been manually expired"
                    ))
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
                        .embed(CreateEmbed::new().title("Action expired").description(format!("The action with ID `{id}` has been manually expired")).footer(CreateEmbedFooter::new(format!("Action expired | UUID: {id}"))).color(0x2e4045))
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
