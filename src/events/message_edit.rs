use serenity::{
    all::{ChannelId, MessageUpdateEvent, UserId},
    builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage},
    prelude::Context,
};
use tracing::error;

use crate::{
    common::logging::{get_log_channel, LogType},
    models::{config::LoggingConfig, handler::Handler, message::MessageQuery},
};

impl Handler {
    pub async fn on_message_edit(&self, ctx: Context, event: MessageUpdateEvent) {
        let guild_id = event.guild_id.unwrap().get() as i64;
        let channel_id = event.channel_id.get() as i64;
        let message_id = event.id.get() as i64;

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

        let author = match ctx.http.get_user(UserId::new(message.user_id as u64)).await {
            Ok(author) => author,
            Err(err) => {
                error!("Failed to get author: {:?}", err);
                return;
            }
        };

        if event.content.is_none() && message.attachment.is_none() && event.attachments.is_none() {
            return;
        }

        let content = match event.content {
            Some(content) => {
                if content == message.content {
                    None
                } else {
                    Some(content)
                }
            }
            None => None,
        };

        let current_attachment = match event.attachments {
            Some(attachments) => attachments
                .first()
                .map(|attachment| attachment.url.to_string()),
            None => None,
        };
        let mut change_attachment = current_attachment.clone();
        if let Some(current_attachment) = &current_attachment {
            if let Some(message_attachment) = &message.attachment {
                if current_attachment == message_attachment {
                    change_attachment = None;
                }
            }
        }
        if let Some(message_attachment) = &message.attachment {
            if current_attachment.is_none() {
                change_attachment = Some(message_attachment.to_string());
            }
        }

        let mut embed = CreateEmbed::new()
            .title("Message Edited")
            .color(0xf5e0a9)
            .author(
                CreateEmbedAuthor::new(&author.name)
                    .icon_url(author.avatar_url().unwrap_or_default()),
            )
            .footer(CreateEmbedFooter::new(format!(
                "User {} edited a message in {}",
                author.id.get(),
                message.channel_id
            )));
        let mut fields = vec![];
        if let Some(content) = &content {
            fields.push(("Old content", message.clone().content, false));
            fields.push(("New content", content.clone(), false));
        }
        if let Some(attachment) = &change_attachment {
            fields.push(("Attachment changed", String::new(), false));
            embed = embed.image(attachment);
        }
        embed = embed.fields(fields);

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

        if let Err(err) = message
            .update(&self.redis_database, content, current_attachment)
            .await
        {
            error!("Failed to update message: {:?}", err);
        };
    }
}
