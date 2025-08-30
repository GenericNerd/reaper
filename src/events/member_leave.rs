use metrics::histogram;
use tracing::error;

use crate::{
    events::EventRouter,
    models::{bot::Bot, guild::Guild, user::User},
};

impl EventRouter {
    #[tracing::instrument(skip(guild, user), fields(guild_id = guild.as_u64(), user_id = user.as_u64()))]
    pub async fn member_leave(&self, guild: Guild, user: User) {
        let timing = histogram!("bot.timing.member_leave");
        let start = std::time::Instant::now();
        let xp_config = match sqlx::query!(
            "SELECT stack_rewards, reset_level_on_leave FROM xp_configuration WHERE guild_id = $1",
            guild.as_i64()
        )
        .fetch_optional(Bot::global().postgres())
        .await
        {
            Ok(row) => match row {
                Some(row) => row,
                None => return,
            },
            Err(err) => {
                error!("Failed to fetch XP configuration: {err}");
                return;
            }
        };

        if !xp_config.reset_level_on_leave {
            return;
        }

        timing.record(start.elapsed());
    }
}
