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

pub struct UnbanCommand;

#[async_trait::async_trait]
impl Command for UnbanCommand {
    fn name(&self) -> &'static str {
        "unban"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("unban")
            .dm_permission(false)
            .description("Unban a user from the server")
            .add_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "The user to unban")
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

        if !ctx.user_permissions.contains(&Permission::ModerationUnban) {
            return Err(ResponseError::Execution(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::ModerationUnban)),
            ));
        }

        let options = Options {
            options: cmd.data.options(),
        };

        let Some(user) = options.get_user("user").into_owned() else {
            return Err(ResponseError::Execution(
                "User not provided!",
                Some("A user must be provided before continuing!".to_string()),
            ));
        };

        if let Err(err) = ctx
            .ctx
            .http
            .remove_ban(
                cmd.guild_id.unwrap(),
                user.id,
                Some(&format!(
                    "Unban by {} ({})",
                    cmd.user.name,
                    cmd.user.id.get()
                )),
            )
            .await
        {
            error!(
                "Could not unban user {} in guild {}. Failed with error: {:?}",
                user.id.get(),
                cmd.guild_id.unwrap().get(),
                err
            );
            return Err(ResponseError::Execution(
                "Could not unban user",
                Some("The user could not be unbanned. This could be because no ban exists. Please double check before trying again".to_string()),
            ));
        };

        if let Err(err) = sqlx::query!("UPDATE actions SET active=false WHERE user_id = $1 AND action_type = 'ban' AND active = true", user.id.get() as i64).execute(&handler.main_database).await {
            error!("Could not expire active bans for user {} in guild {}. Failed with error: {:?}", user.id.get(), cmd.guild_id.unwrap().get(), err);
        }

        let reply = ctx.reply(
            cmd,
            Response::new().embed(
                CreateEmbed::new()
                    .title("User unbanned!")
                    .description(format!("<@{}> has been unbanned", user.id.get()))
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
            if let Some(channel) = get_log_channel(&config, &LogType::Action) {
                Some(ChannelId::new(channel as u64)
                    .send_message(
                        &ctx.ctx,
                        CreateMessage::new()
                            .embed(CreateEmbed::new().title("User unbanned").description(format!("<@{}> has been unbanned", user.id.get())).footer(CreateEmbedFooter::new(format!("User {} unbanned", user.id.get()))).color(0x0abfd6))
                ))
            } else {
                None
            }
        } else {
            None
        };

        match log {
            Some(log_future) => tokio::join!(log_future, reply).1,
            None => reply.await,
        }
    }
}
