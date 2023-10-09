use tracing::error;

use crate::models::{actions::DatabaseAction, handler::Handler};

pub async fn get_active_strikes(
    handler: &Handler,
    guild_id: i64,
    user_id: i64,
) -> Vec<DatabaseAction> {
    match sqlx::query_as!(
        DatabaseAction,
        "SELECT * FROM actions WHERE guild_id = $1 AND user_id = $2 AND action_type = 'strike' AND active = true",
        guild_id,
        user_id
    ).fetch_all(&handler.main_database).await {
        Ok(strikes) => strikes,
        Err(err) => {
            error!(
                "Attempted to query main database for guild {guild_id} active strikes for user {user_id}, failed with error: {err}",
            );
            Vec::new()
        }
    }
}
