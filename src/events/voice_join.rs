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
    pub async fn voice_join(&self, ctx: Context, state: VoiceState) {
        let guild_id = state.guild_id.unwrap();

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
                        CreateMessage::new().content(format!("<@{}>", state.user_id.get()))
                            .embed(CreateEmbed::new().title("joined").description(format!("<#{}>", state.channel_id.unwrap())).footer(CreateEmbedFooter::new(format!("User {} joined VC {}", state.user_id.get(), state.channel_id.unwrap()))).color(0x2dc770))
                            .allowed_mentions(CreateAllowedMentions::new().empty_roles().empty_users())
                ).await {
                    error!("Failed to send voice join log message: {}", err);
                }
            }
        }
    }
}
