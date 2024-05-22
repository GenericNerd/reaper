use serenity::{
    all::{ChannelId, CommandInteraction, CommandOptionType, RoleId},
    builder::{CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateMessage},
};
use std::time::Instant;
use tracing::error;

use crate::{
    common::{
        logging::{get_log_channel, LogType},
        options::Options,
    },
    database::postgres::guild::get_moderation_config,
    models::{
        command::{Command, CommandContext, CommandContextReply},
        config::LoggingConfig,
        handler::Handler,
        permissions::Permission,
        response::{Response, ResponseError, ResponseResult},
    },
};

pub struct UnmuteCommand;

#[async_trait::async_trait]
impl Command for UnmuteCommand {
    fn name(&self) -> &'static str {
        "unmute"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("unmute")
            .dm_permission(false)
            .description("Unmute a user from the server")
            .add_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "The user to unmute")
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

        if !ctx.user_permissions.contains(&Permission::ModerationUnmute) {
            return Err(ResponseError::Execution(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::ModerationUnmute)),
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

        let mute_role = match get_moderation_config(handler, cmd.guild_id.unwrap().get() as i64)
            .await
        {
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

        if let Err(err) = ctx
            .ctx
            .http
            .remove_member_role(
                cmd.guild_id.unwrap(),
                user.id,
                mute_role,
                Some(&format!(
                    "Unmute by {} ({})",
                    cmd.user.name,
                    cmd.user.id.get()
                )),
            )
            .await
        {
            error!(
                "Could not unmute user {} in guild {}. Failed with error: {:?}",
                user.id.get(),
                cmd.guild_id.unwrap().get(),
                err
            );
            return Err(ResponseError::Execution(
                "Could not unmute user",
                Some("The user could not be unmuted. This could be because they currently do not have the mute role. Please double check before trying again".to_string()),
            ));
        }

        if let Err(err) = sqlx::query!("UPDATE actions SET active=false WHERE user_id = $1 AND action_type = 'mute' AND active = true", user.id.get() as i64).execute(&handler.main_database).await {
            error!("Could not expire active mutes for user {} in guild {}. Failed with error: {:?}", user.id.get(), cmd.guild_id.unwrap().get(), err);
        }

        let reply = ctx.reply(
            cmd,
            Response::new().embed(
                CreateEmbed::new()
                    .title("User unmuted!")
                    .description(format!("<@{}> has been unmuted", user.id.get()))
                    .color(0xd1bfba)
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
            if let Some(channel) = get_log_channel(&handler, &config, &LogType::Action).await {
                Some(ChannelId::new(channel as u64)
                    .send_message(
                        &ctx.ctx,
                        CreateMessage::new()
                            .embed(CreateEmbed::new().title("User unmuted").description(format!("<@{}> has been unmuted", user.id.get())).footer(CreateEmbedFooter::new(format!("User {} unmuted", user.id.get()))).color(0xd1bfba))
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
