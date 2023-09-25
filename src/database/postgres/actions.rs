use tracing::error;

use crate::models::{actions::Action, handler::Handler};

pub async fn get_active_strikes(handler: &Handler, guild_id: i64, user_id: i64) -> Vec<Action> {
    match sqlx::query_as_unchecked!(
        Action,
        "SELECT id, type as action_type, user_id, moderator_id, guild_id, reason, active, expiry FROM actions WHERE guild_id = $1 AND user_id = $2 AND type = 'strike' AND active = true",
        guild_id,
        user_id
    )
    .fetch_all(&handler.main_database)
    .await {
        Ok(strikes) => strikes,
        Err(err) => {
            error!(
                "Attempted to query main database for guild {guild_id} active strikes for user {user_id}, failed with error: {err}",
            );
            Vec::new()
        }
    }
}
