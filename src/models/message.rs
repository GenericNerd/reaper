use tracing::{debug, error};

use super::response::ResponseError;

#[derive(Clone)]
pub struct Message {
    pub guild_id: i64,
    pub user_id: i64,
    pub channel_id: i64,
    pub message_id: i64,
    pub content: String,
    pub attachment: Option<String>,
}

impl Message {
    pub async fn new(
        redis: &redis::Client,
        guild_id: i64,
        user_id: i64,
        channel_id: i64,
        message_id: i64,
        content: String,
        attachment: Option<String>,
    ) -> Result<Self, ResponseError> {
        let start = std::time::Instant::now();

        let mut connection = match redis.get_multiplexed_async_connection().await {
            Ok(connection) => connection,
            Err(err) => {
                error!("Failed to get Redis connection: {:?}", err);
                return Err(ResponseError::Redis(err));
            }
        };

        debug!("Got Redis connection in {:?}", start.elapsed());

        let message = Self {
            guild_id,
            user_id,
            channel_id,
            message_id,
            content,
            attachment,
        };

        match redis::cmd("HSET")
            .arg(message.key())
            .arg("guild_id")
            .arg(message.guild_id)
            .arg("user_id")
            .arg(message.user_id)
            .arg("channel_id")
            .arg(message.channel_id)
            .arg("message_id")
            .arg(message.message_id)
            .arg("content")
            .arg(message.content.clone())
            .arg("attachment")
            .arg(message.attachment.clone().unwrap_or("null".to_string()))
            .query_async(&mut connection)
            .await
        {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to set message in Redis: {:?}", err);
                return Err(ResponseError::Redis(err));
            }
        }

        debug!("Set message in Redis in {:?}", start.elapsed());

        match redis::cmd("EXPIRE")
            .arg(message.key())
            .arg(86400)
            .query_async(&mut connection)
            .await
        {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to set message expiration in Redis: {:?}", err);
                return Err(ResponseError::Redis(err));
            }
        }

        debug!("Set message expiration in Redis in {:?}", start.elapsed());

        Ok(message)
    }

    pub fn key(&self) -> String {
        format!("{}:{}:{}", self.guild_id, self.channel_id, self.message_id)
    }

    pub async fn update(
        &self,
        redis: &redis::Client,
        content: Option<String>,
        attachment: Option<String>,
    ) -> Result<Message, ResponseError> {
        let content = match content {
            Some(content) => content,
            None => self.content.clone(),
        };
        let attachment = match attachment {
            Some(attachment) => Some(attachment),
            None => Some("null".to_string()),
        };
        Message::new(
            redis,
            self.guild_id,
            self.user_id,
            self.channel_id,
            self.message_id,
            content,
            attachment,
        )
        .await
    }
}

pub struct MessageQuery {
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_id: i64,
}

impl MessageQuery {
    pub async fn get_message(&self, redis: &redis::Client) -> Result<Message, ResponseError> {
        let start = std::time::Instant::now();

        let mut connection = match redis.get_multiplexed_async_connection().await {
            Ok(connection) => connection,
            Err(err) => {
                error!("Failed to get Redis connection: {:?}", err);
                return Err(ResponseError::Redis(err));
            }
        };

        debug!("Got Redis connection in {:?}", start.elapsed());

        let exists: u8 = match redis::cmd("HEXISTS")
            .arg(self.key())
            .arg("guild_id")
            .query_async(&mut connection)
            .await
        {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to check if message exists in Redis: {:?}", err);
                return Err(ResponseError::Redis(err));
            }
        };

        if exists == 0 {
            return Err(ResponseError::Execution(
                "Message not found in database",
                None,
            ));
        }

        let guild_id: i64 = match redis::cmd("HGET")
            .arg(self.key())
            .arg("guild_id")
            .query_async(&mut connection)
            .await
        {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to get message guild ID from Redis: {:?}", err);
                return Err(ResponseError::Redis(err));
            }
        };

        let user_id: i64 = match redis::cmd("HGET")
            .arg(self.key())
            .arg("user_id")
            .query_async(&mut connection)
            .await
        {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to get message user ID from Redis: {:?}", err);
                return Err(ResponseError::Redis(err));
            }
        };

        let channel_id: i64 = match redis::cmd("HGET")
            .arg(self.key())
            .arg("channel_id")
            .query_async(&mut connection)
            .await
        {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to get message channel ID from Redis: {:?}", err);
                return Err(ResponseError::Redis(err));
            }
        };

        let message_id: i64 = match redis::cmd("HGET")
            .arg(self.key())
            .arg("message_id")
            .query_async(&mut connection)
            .await
        {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to get message ID from Redis: {:?}", err);
                return Err(ResponseError::Redis(err));
            }
        };

        let content: String = match redis::cmd("HGET")
            .arg(self.key())
            .arg("content")
            .query_async(&mut connection)
            .await
        {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to get message content from Redis: {:?}", err);
                return Err(ResponseError::Redis(err));
            }
        };

        let attachment: Option<String> = match redis::cmd("HGET")
            .arg(self.key())
            .arg("attachment")
            .query_async(&mut connection)
            .await
        {
            Ok(res) => {
                // This must be allowed to be a manual filter due to type assumptions by the compiler
                #[allow(clippy::manual_filter)]
                if let Some(value) = res {
                    if value == "null" {
                        None
                    } else {
                        Some(value)
                    }
                } else {
                    None
                }
            }
            Err(err) => {
                error!("Failed to get message attachment from Redis: {:?}", err);
                return Err(ResponseError::Redis(err));
            }
        };

        Ok(Message {
            guild_id,
            user_id,
            channel_id,
            message_id,
            content,
            attachment,
        })
    }

    pub fn key(&self) -> String {
        format!("{}:{}:{}", self.guild_id, self.channel_id, self.message_id)
    }
}

impl From<Message> for MessageQuery {
    fn from(value: Message) -> Self {
        Self {
            guild_id: value.guild_id,
            channel_id: value.channel_id,
            message_id: value.message_id,
        }
    }
}
