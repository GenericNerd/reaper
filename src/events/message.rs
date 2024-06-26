use serenity::all::Message as DiscordMessage;
use tracing::error;

use crate::models::{handler::Handler, message::Message};

impl Handler {
    pub async fn on_message(&self, message: DiscordMessage) {
        let attachment_url = message
            .attachments
            .first()
            .map(|attachment| attachment.url.to_string());

        if let Err(err) = Message::new(
            &self.redis_database,
            message.guild_id.unwrap().get() as i64,
            message.author.id.get() as i64,
            message.channel_id.get() as i64,
            message.id.get() as i64,
            message.content,
            attachment_url,
        )
        .await
        {
            error!("Failed to create message: {:?}", err);
        };
    }
}
