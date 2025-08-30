use tracing::error;

use crate::{
    events::EventRouter,
    models::{bot::Bot, guild::Guild},
};
use metrics::{gauge, histogram};

impl EventRouter {
    #[tracing::instrument(skip(guild), fields(guild_id = guild.as_u64()))]
    pub async fn guild_leave(&self, guild: Guild) {
        let histogram = histogram!("bot.timing.guild_leave");
        let start = std::time::Instant::now();
        gauge!("bot.guild_count").decrement(1);
        if let Err(err) = sqlx::query!(
            "DELETE FROM moderation_configuration WHERE guild_id = $1",
            guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await
        {
            error!(
                guild_id = guild.as_u64(),
                "Failed to delete moderation configuration: {err}"
            );
        };

        if let Err(err) = sqlx::query!(
            "DELETE FROM logging_configuration WHERE guild_id = $1",
            guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await
        {
            error!(
                guild_id = guild.as_u64(),
                "Failed to delete logging configuration: {err}"
            );
        }

        if let Err(err) = sqlx::query!(
            "DELETE FROM guild_role_recovery_config WHERE guild_id = $1",
            guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await
        {
            error!(
                guild_id = guild.as_u64(),
                "Failed to delete guild role recovery configuration: {err}"
            )
        }
        histogram.record(start.elapsed());
    }
}
