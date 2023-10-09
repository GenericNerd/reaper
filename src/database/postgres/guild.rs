use tracing::{debug, error};

use crate::models::{config::ModerationConfig, handler::Handler};

pub async fn get_moderation_config(handler: &Handler, guild_id: i64) -> Option<ModerationConfig> {
    debug!("Querying main database for guild {guild_id} moderation configuration");
    match sqlx::query_as!(
        ModerationConfig,
        "SELECT mute_role, default_strike_duration FROM moderation_configuration WHERE guild_id = $1",
        guild_id
    )
    .fetch_optional(&handler.main_database)
    .await {
        Ok(config) => config,
        Err(err) => {
            error!(
                "Attempted to query main database for guild {guild_id} moderation configuration, failed with error: {err}",
            );
            None
        }
    }
}
