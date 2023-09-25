use tracing::{debug, error};

use crate::models::{actions::ActionEscalation, config::ModerationConfig, handler::Handler};

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

pub async fn get_strike_escalations(handler: &Handler, guild_id: i64) -> Vec<ActionEscalation> {
    debug!("Querying main database for guild {guild_id} strike escalations");
    match sqlx::query_as_unchecked!(
        ActionEscalation,
        "SELECT guild_id, strike_count, action_type, action_duration FROM strike_escalations WHERE guild_id = $1",
        guild_id
    )
    .fetch_all(&handler.main_database)
    .await {
        Ok(escalations) => escalations,
        Err(err) => {
            error!(
                "Attempted to query main database for guild {guild_id} strike escalations, failed with error: {err}",
            );
            Vec::new()
        }
    }
}
