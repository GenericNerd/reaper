use std::time::{SystemTime, UNIX_EPOCH};

use serenity::{
    all::{
        ChannelId, CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption,
        CreateEmbed, CreateEmbedFooter, CreateMessage, GetMessages,
    },
    model::error::Error as ModelError,
    Error,
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
        message::MessageQuery,
        permissions::Permission,
        response::{Response, ResponseError, ResponseResult},
    },
};

pub struct PurgeCommand;

#[async_trait::async_trait]
impl Command for PurgeCommand {
    fn name(&self) -> &'static str {
        "purge"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("purge")
            .description("Purge multiple messages from a channel")
            .dm_permission(false)
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "count",
                    "The amount of messages to delete",
                )
                .required(false),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::User,
                    "author",
                    "Specify a user to delete messages from",
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
        if !ctx.user_permissions.contains(&Permission::ModerationPurge) {
            return Err(ResponseError::Execution(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::ModerationPurge)),
            ));
        }

        let mut deleted_messages = 0;

        let options = Options {
            options: cmd.data.options(),
        };

        let message_count_to_delete = match options.get_integer("count") {
            Some(count) => count.min(1000),
            None => 100,
        };

        let author = options.get_user("author").into_owned();

        let mut messages_to_delete = vec![];

        for _ in 0..((message_count_to_delete / 100) + 1).min(10) {
            let mut capped_messages = false;

            let messages = match cmd
                .channel_id
                .messages(
                    &ctx.ctx.http,
                    GetMessages::new()
                        .limit(
                            u8::try_from(message_count_to_delete - deleted_messages)
                                .unwrap()
                                .min(100),
                        )
                        .after(
                            ((SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                                - (14 * 24 * 60 * 60))
                                * 1000
                                - 1420070400000)
                                << 22,
                        ),
                )
                .await
            {
                Ok(messages) => messages,
                Err(err) => {
                    error!("Error getting messages: {:?}", err);
                    return Err(ResponseError::Serenity(err));
                }
            };

            if messages.len()
                != usize::try_from((message_count_to_delete - deleted_messages).min(100)).unwrap()
            {
                capped_messages = true;
            }

            for message in messages {
                if let Some(author) = &author {
                    if message.author.id == author.id && !message.author.bot {
                        messages_to_delete.push(message);
                    }
                } else if !message.author.bot {
                    messages_to_delete.push(message);
                }
            }

            if messages_to_delete.len() == 1 {
                MessageQuery {
                    guild: cmd.guild_id.unwrap().get() as i64,
                    channel: cmd.channel_id.get() as i64,
                    message: messages_to_delete[0].id.get() as i64,
                }
                .delete(&handler.redis_database)
                .await?;

                match cmd
                    .channel_id
                    .delete_message(&ctx.ctx.http, messages_to_delete[0].id)
                    .await
                {
                    Ok(()) => {
                        deleted_messages += 1;
                    }
                    Err(err) => {
                        error!("Error deleting message: {:?}", err);
                        return Err(ResponseError::Serenity(err));
                    }
                };
            } else {
                for message in &messages_to_delete {
                    MessageQuery {
                        guild: cmd.guild_id.unwrap().get() as i64,
                        channel: cmd.channel_id.get() as i64,
                        message: message.id.get() as i64,
                    }
                    .delete(&handler.redis_database)
                    .await?;
                }

                match cmd
                    .channel_id
                    .delete_messages(&ctx.ctx.http, messages_to_delete.iter().map(|m| m.id))
                    .await
                {
                    Ok(()) => {
                        deleted_messages += messages_to_delete.len() as i64;
                    }
                    Err(err) => {
                        error!("Error deleting messages: {:?}", err);
                        if let Error::Model(ModelError::BulkDeleteAmount) = err {
                        } else {
                            return Err(ResponseError::Serenity(err));
                        }
                    }
                };
            }

            if capped_messages {
                break;
            }

            messages_to_delete = vec![];
        }

        if let Ok(config) = sqlx::query_as!(
            LoggingConfig,
            "SELECT log_actions, log_messages, log_voice, log_channel, log_action_channel, log_message_channel, log_voice_channel FROM logging_configuration WHERE guild_id = $1",
            cmd.guild_id.unwrap().get() as i64
        )
        .fetch_one(&handler.main_database)
        .await {
            let footer = if author.is_some() {
                format!("Moderator {} purged {} messages from {}", cmd.user.id.get(), deleted_messages, author.clone().unwrap().id.get())
            } else {
                format!("Moderator {} purged {} messages", cmd.user.id.get(), deleted_messages)
            };

            let description = if author.is_some() {
                format!("<@{}> has purged `{}` messages from <#{}>", cmd.user.id.get(), deleted_messages, cmd.channel_id.get())
            } else {
                format!("<@{}> has purged `{}` messages", cmd.user.id.get(), deleted_messages)
            };

            if let Some(channel) = get_log_channel(handler, &config, &LogType::Action).await {
                if let Err(err) = ChannelId::new(channel as u64)
                    .send_message(
                        &ctx.ctx,
                        CreateMessage::new()
                            .embed(CreateEmbed::new().title("Messages purged").description(description).footer(CreateEmbedFooter::new(footer)).color(0xc92525))
                ).await {
                    error!("Failed to send messages purged message: {}", err);
                }
            }
        }

        let description = if author.is_some() {
            format!(
                "`{deleted_messages}` messages posted by <@{}> have been purged",
                author.unwrap().id.get()
            )
        } else {
            format!("`{deleted_messages}` messages have been purged")
        };

        if ctx
            .reply(
                cmd,
                Response::new().embed(
                    CreateEmbed::new()
                        .title("Messages purged")
                        .description(&description)
                        .color(0xc92525),
                ),
            )
            .await
            .is_err()
        {
            match cmd
                .channel_id
                .send_message(
                    &ctx.ctx.http,
                    CreateMessage::new().embed(
                        CreateEmbed::new()
                            .title("Messages purged")
                            .description(description)
                            .color(0xc92525),
                    ),
                )
                .await
            {
                Ok(_) => Ok(()),
                Err(err) => Err(ResponseError::Serenity(err)),
            }
        } else {
            Ok(())
        }
    }
}
