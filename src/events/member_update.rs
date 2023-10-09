use serenity::all::{GuildMemberUpdateEvent, Member, RoleId};
use tracing::error;

use crate::models::handler::Handler;

impl Handler {
    pub async fn on_member_update(&self, member: Member, event: GuildMemberUpdateEvent) {
        let roles = match sqlx::query!(
            "SELECT role_id FROM role_recovery WHERE guild_id = $1 AND user_id = $2",
            member.guild_id.get() as i64,
            member.user.id.get() as i64
        )
        .fetch_all(&self.main_database)
        .await
        {
            Ok(roles) => {
                let mut role_ids = vec![];
                for role in roles {
                    role_ids.push(RoleId::new(role.role_id as u64));
                }
                role_ids
            }
            Err(err) => {
                error!(
                    "Could not get current roles from database during update. Failed with error: {:?}",
                    err
                );
                return;
            }
        };

        if roles != event.roles {
            let guild_id = member.guild_id.get() as i64;
            let user_id = member.user.id.get() as i64;

            let roles_to_add = event
                .roles
                .clone()
                .into_iter()
                .filter(|role| !roles.contains(role))
                .collect::<Vec<RoleId>>();
            let roles_to_remove = roles
                .into_iter()
                .filter(|role| !event.roles.contains(role))
                .collect::<Vec<RoleId>>();
            for role in roles_to_add {
                let role_id = role.get() as i64;

                if let Err(err) = sqlx::query!(
                    "INSERT INTO role_recovery (guild_id, user_id, role_id) VALUES ($1, $2, $3)",
                    guild_id,
                    user_id,
                    role_id
                )
                .execute(&self.main_database)
                .await
                {
                    error!(
                        "Could not insert role into role recovery. Failed with error: {:?}",
                        err
                    );
                };
            }
            for role in roles_to_remove {
                let role_id = role.get() as i64;

                if let Err(err) = sqlx::query!(
                    "DELETE FROM role_recovery WHERE guild_id = $1 AND user_id = $2 AND role_id = $3",
                    guild_id,
                    user_id,
                    role_id
                ).execute(&self.main_database).await {
                    error!(
                        "Could not delete role from role recovery. Failed with error: {:?}",
                        err
                    );
                }
            }
        }
    }
}
