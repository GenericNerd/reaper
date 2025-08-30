use metrics::histogram;
use serenity::all::{Context, RoleId};
use tracing::{debug, error};

use crate::{
    events::EventRouter,
    models::{bot::Bot, member::Member},
};

impl EventRouter {
    #[tracing::instrument(skip(ctx, member), fields(guild_id = member.guild().as_u64(), user_id = member.user().as_u64()))]
    pub async fn member_join(&self, ctx: Context, member: Member) {
        let timing = histogram!("bot.timing.member_join");
        let start = std::time::Instant::now();

        let enabled = match sqlx::query!(
            "SELECT enabled FROM guild_role_recovery_config WHERE guild_id = $1",
            member.guild().as_i64()
        )
        .fetch_one(Bot::global().postgres())
        .await
        {
            Ok(row) => row.enabled,
            Err(err) => {
                error!(
                    guild_id = member.guild().as_u64(),
                    "Failed to fetch guild role recovery configuration: {err}"
                );
                return;
            }
        };

        if !enabled {
            return;
        }
        debug!("Role recovery enabled, fetching roles");

        let Ok(role_ids) = sqlx::query!(
            "SELECT role_id FROM role_recovery WHERE guild_id = $1 AND user_id = $2",
            member.guild().as_i64(),
            member.user().as_i64()
        )
        .fetch_all(Bot::global().postgres())
        .await
        else {
            error!(
                guild_id = member.guild().as_u64(),
                user_id = member.user().as_u64(),
                "Failed to fetch role recovery configuration"
            );
            return;
        };

        // TODO: Handle auto-roles here as well
        let roles = role_ids
            .iter()
            .map(|row| RoleId::new(row.role_id as u64))
            .collect::<Vec<RoleId>>();
        let mut roles_to_add = vec![];

        for role in roles {
            let role_failed = member
                .guild()
                .as_serenity_id()
                .role(&ctx.http, role)
                .await
                .is_err();
            if !role_failed {
                roles_to_add.push(role);
            }
        }

        if roles_to_add.is_empty() {
            return;
        }

        if let Err(err) = member
            .as_serenity()
            .add_roles(&ctx.http, &roles_to_add)
            .await
        {
            error!(
                guild_id = member.guild().as_u64(),
                user_id = member.user().as_u64(),
                "Failed to add roles to user: {err}"
            );
        }
        timing.record(start.elapsed());
    }
}
