#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
use tracing::error;

use crate::models::handler::Handler;

impl Handler {
    pub async fn on_member_leave(&self, guild_id: i64, user_id: i64) {
        let guild_configuration = match sqlx::query!(
            "SELECT stack_rewards, reset_level_on_leave FROM xp_configuration WHERE guild_id = $1",
            guild_id
        )
        .fetch_optional(&self.main_database)
        .await
        {
            Ok(guild_configuration) => match guild_configuration {
                Some(guild_configuration) => guild_configuration,
                None => {
                    return;
                }
            },
            Err(err) => {
                error!(
                    "Failed to fetch XP configuration. Failed with error: {:?}",
                    err
                );
                return;
            }
        };

        if !guild_configuration.reset_level_on_leave {
            return;
        }

        let user_xp = match sqlx::query!(
            "SELECT xp FROM user_xp WHERE guild_id = $1 AND user_id = $2",
            guild_id,
            user_id
        )
        .fetch_optional(&self.main_database)
        .await
        {
            Ok(user_xp) => match user_xp {
                Some(user_xp) => user_xp.xp,
                None => 0,
            },
            Err(err) => {
                error!(
                    "Failed to fetch XP from database. Failed with error: {:?}",
                    err
                );
                return;
            }
        };

        if let Err(err) = sqlx::query!(
            "DELETE FROM user_xp WHERE guild_id = $1 AND user_id = $2",
            guild_id,
            user_id
        )
        .execute(&self.main_database)
        .await
        {
            error!(
                "Failed to delete XP from user. Failed with error: {:?}",
                err
            );
        }

        let user_level =
            ((-25.0 + f64::sqrt((625 + (200 * user_xp)) as f64)) / 100.0).floor() as i64;
        let roles_to_remove = if guild_configuration.stack_rewards {
            match sqlx::query!(
                "SELECT role FROM xp_rewards WHERE guild_id = $1 AND level <= $2",
                guild_id,
                user_level
            )
            .fetch_all(&self.main_database)
            .await
            {
                Ok(roles) => roles.iter().map(|role| role.role).collect::<Vec<_>>(),
                Err(err) => {
                    error!("Failed to fetch XP rewards. Failed with error: {:?}", err);
                    return;
                }
            }
        } else {
            match sqlx::query!(
                "WITH closest_level AS (SELECT MAX(level) AS max_level FROM xp_rewards WHERE guild_id = $1 AND level <= $2) SELECT role FROM xp_rewards WHERE level = (SELECT max_level FROM closest_level) AND guild_id = $1",
                guild_id,
                user_level
            )
            .fetch_all(&self.main_database)
            .await {
                Ok(roles) => roles.iter().map(|role| role.role).collect::<Vec<_>>(),
                Err(err) => {
                    error!("Failed to fetch XP rewards. Failed with error: {:?}", err);
                    return;
                }
            }
        };

        for role in roles_to_remove {
            if let Err(err) = sqlx::query!(
                "DELETE FROM role_recovery WHERE guild_id = $1 AND user_id = $2 AND role_id = $3",
                guild_id,
                user_id,
                role
            )
            .execute(&self.main_database)
            .await
            {
                error!(
                    "Failed to remove role recovery. Failed with error: {:?}",
                    err
                );
            }
        }
    }
}
