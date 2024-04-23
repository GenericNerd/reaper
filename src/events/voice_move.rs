use serenity::{
    all::{ChannelId, VoiceState},
    builder::{CreateAllowedMentions, CreateEmbed, CreateEmbedFooter, CreateMessage},
    client::Context,
};
use tracing::error;

use crate::{
    common::logging::{get_log_channel, LogType},
    models::{config::LoggingConfig, handler::Handler},
};

impl Handler {
    pub async fn voice_move(&self, ctx: Context, old: VoiceState, new: VoiceState) {
        let guild_id = new.guild_id.unwrap();

        if let Ok(config) = sqlx::query_as!(
            LoggingConfig,
            "SELECT log_actions, log_messages, log_voice, log_channel, log_action_channel, log_message_channel, log_voice_channel FROM logging_configuration WHERE guild_id = $1",
            guild_id.get() as i64
        )
        .fetch_one(&self.main_database)
        .await {
            if let Some(channel) = get_log_channel(self, &config, &LogType::Voice).await {
                if let Err(err) = ChannelId::new(channel as u64)
                    .send_message(
                        &ctx,
                        CreateMessage::new().content(format!("<@{}>", new.user_id.get()))
                            .embed(CreateEmbed::new().title("moved").description(format!("from <#{}> to <#{}>", old.channel_id.unwrap(), new.channel_id.unwrap())).footer(CreateEmbedFooter::new(format!("User {} moved from {} to {}", new.user_id.get(), old.channel_id.unwrap(), new.channel_id.unwrap()))).color(0x778889))
                            .allowed_mentions(CreateAllowedMentions::new().empty_roles().empty_users())
                ).await {
                    error!("Failed to send voice leave log message: {}", err);
                }
            }
        }
    }
}
