use std::{collections::HashMap, time::Duration};

use serenity::{
    all::{GuildId, RoleId, UserId},
    prelude::Context,
};
use tracing::{debug, error};

use crate::models::{
    actions::{ActionType, DatabaseAction},
    handler::Handler,
};

struct MuteRole {
    mute_role: Option<i64>,
}

pub async fn expire_actions(handler: Handler, ctx: Context) {
    loop {
        let start = std::time::Instant::now();
        let actions = match sqlx::query_as!(
            DatabaseAction,
            "SELECT * FROM actions WHERE expiry < now() AND active=true"
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

        let mut guild_configurations: HashMap<i64, i64> = HashMap::new();
        for action in &actions {
            let guild_id = &action.guild_id;
            if !guild_configurations.contains_key(guild_id) {
                if let Ok(config) = sqlx::query_as!(
                    MuteRole,
                    "SELECT mute_role FROM moderation_configuration WHERE guild_id = $1",
                    guild_id
                )
                .fetch_one(&handler.main_database)
                .await
                {
                    if config.mute_role.is_some() {
                        guild_configurations.insert(*guild_id, config.mute_role.unwrap());
                    }
                };
            }
        }

        for action in actions {
            debug!(
                "Expiring action with ID {} from guild {}",
                action.id, action.guild_id
            );
            match ActionType::from(action.action_type) {
                ActionType::Mute => {
                    if let Some(mute_role) = guild_configurations.get(&action.guild_id) {
                        if let Err(err) = ctx
                            .http
                            .remove_member_role(
                                GuildId::new(action.guild_id as u64),
                                UserId::new(action.user_id as u64),
                                RoleId::new(*mute_role as u64),
                                Some(&format!("Expiring mute {}", action.id)),
                            )
                            .await
                        {
                            error!("Failed to remove mute: {}", err);
                        }
                    }
                    continue;
                }
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
