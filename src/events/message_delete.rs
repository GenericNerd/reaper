use serenity::{
    all::{ChannelId, GuildId, UserId},
    builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage},
    prelude::Context,
};
use tracing::error;

use crate::{
    common::logging::{get_log_channel, LogType},
    models::{config::LoggingConfig, handler::Handler, message::MessageQuery},
};

impl Handler {
    pub async fn on_message_delete(
        &self,
        ctx: Context,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
    ) {
        let query = MessageQuery {
            guild: guild_id,
            channel: channel_id,
            message: message_id,
        };

        let message = match query.get_message(&self.redis_database).await {
            Ok(message) => message,
            Err(err) => {
                error!("Failed to get message: {:?}", err);
                return;
            }
        };

        let mut deleted_by = message.user_id;
        match ctx
            .http
            .get_audit_logs(GuildId::new(guild_id as u64), None, None, None, Some(1))
            .await
        {
            Ok(audit_log) => {
                if let Some(entry) = audit_log.entries.first() {
                    if entry.action.num() == 72 {
                        if let Some(target) = entry.target_id {
                            if target.get() == message.user_id as u64 {
                                deleted_by = entry.user_id.get() as i64;
                            }
                        }
                    }
                }
            }
            Err(err) => {
                error!("Failed to get audit log: {:?}", err);
            }
        };

        let author = match ctx.http.get_user(UserId::new(message.user_id as u64)).await {
            Ok(author) => author,
            Err(err) => {
                error!("Failed to get author: {:?}", err);
                return;
            }
        };

        let mut embed = CreateEmbed::new()
            .title("Message Deleted")
            .description(message.content)
            .color(0xee2e46)
            .author(
                CreateEmbedAuthor::new(&author.name)
                    .icon_url(author.avatar_url().unwrap_or_default()),
            );
        if message.user_id == deleted_by {
            embed = embed.footer(CreateEmbedFooter::new(format!(
                "Message by {} deleted from {}",
                author.id.get(),
                message.channel_id
            )));
        } else {
            embed = embed.footer(CreateEmbedFooter::new(format!(
                "Message by {} deleted by {} from {}",
                author.id.get(),
                deleted_by,
                message.channel_id
            )));
        }
        if let Some(attachment) = message.attachment {
            embed = embed.image(attachment);
        }

        if let Ok(config) = sqlx::query_as!(
            LoggingConfig,
            "SELECT log_actions, log_messages, log_voice, log_channel, log_action_channel, log_message_channel, log_voice_channel FROM logging_configuration WHERE guild_id = $1",
            guild_id
        )
        .fetch_one(&self.main_database)
        .await {
            if let Some(channel) = get_log_channel(&config, &LogType::Message) {
                if let Err(err) = ChannelId::new(channel as u64)
                    .send_message(
                        &ctx,
                        CreateMessage::new()
                            .embed(embed)
                ).await {
                    error!("Failed to send ban log message: {}", err);
                }
            }
        }
    }
}
