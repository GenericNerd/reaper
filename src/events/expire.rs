use std::time::Duration;

use serenity::{
    all::{GuildId, UserId},
    prelude::Context,
};
use tracing::{debug, error};

use crate::models::{actions::ActionType, handler::Handler};

struct ExpiredAction {
    id: String,
    action_type: ActionType,
    user_id: i64,
    guild_id: i64,
}

pub async fn expire_actions(handler: Handler, ctx: Context) {
    loop {
        let start = std::time::Instant::now();
        let actions = match sqlx::query_as_unchecked!(
            ExpiredAction,
            "SELECT id, type as action_type, user_id, guild_id FROM actions WHERE expiry < now() AND active=true"
        )
        .fetch_all(&handler.main_database)
        .await
        {
            Ok(actions) => actions,
            Err(e) => {
                error!("Failed to fetch actions: {}", e);
                continue;
            }
        };

        for action in actions {
            debug!(
                "Expiring action with ID {} from guild {}",
                action.id, action.guild_id
            );
            match action.action_type {
                ActionType::Ban => {
                    if let Err(err) = ctx
                        .http
                        .remove_ban(
                            GuildId::new(action.guild_id as u64),
                            UserId::new(action.user_id as u64),
                            Some(&format!("Expiring ban {}", action.id)),
                        )
                        .await
                    {
                        error!("Failed to remove ban: {}", err);
                    }
                }
                ActionType::Mute => {
                    continue;
                }
                _ => {}
            }
            if let Err(err) = sqlx::query!("UPDATE actions SET active=false WHERE id=$1", action.id)
                .execute(&handler.main_database)
                .await
            {
                error!("Failed to expire action with ID {}: {}", action.id, err);
                continue;
            }
        }

        debug!(
            "Finished expiring actions in {}ms",
            start.elapsed().as_millis()
        );
        tokio::time::sleep(Duration::from_secs(45)).await;
    }
}
