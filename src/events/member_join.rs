use serenity::{
    all::{Member, RoleId},
    prelude::Context,
};
use tracing::error;

use crate::models::handler::Handler;

impl Handler {
    pub async fn on_member_join(&self, ctx: Context, member: Member) {
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
                error!("Could not get current roles from database during update. Failed with error: {:?}", err);
                return;
            }
        };

        for role in roles {
            if let Err(err) = ctx
                .http
                .add_member_role(
                    member.guild_id,
                    member.user.id,
                    role,
                    Some("Role recovery on rejoin"),
                )
                .await
            {
                error!(
                    "Could not add role to user during rejoin. Failed with error: {:?}",
                    err
                );
            }
        }
    }
}
