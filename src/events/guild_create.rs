use crate::{
    events::EventRouter,
    models::{bot::Bot, guild::Guild},
};
use metrics::{gauge, histogram};
use tracing::error;

impl EventRouter {
    #[tracing::instrument(skip(guild), fields(guild_id = guild.as_u64()))]
    pub async fn guild_create(&self, guild: Guild) {
        let timing = histogram!("bot.timing.guild_create");
        let start = std::time::Instant::now();
        gauge!("bot.guild_count").increment(1);
        if let Err(err) = sqlx::query!(
            "INSERT INTO moderation_configuration (guild_id) VALUES ($1)",
            guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await
        {
            error!(
                guild_id = guild.as_u64(),
                "Failed to insert moderation configuration: {}", err
            );
        }

        if let Err(err) = sqlx::query!(
            "INSERT INTO logging_configuration (guild_id) VALUES ($1)",
            guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await
        {
            error!(
                guild_id = guild.as_u64(),
                "Failed to insert logging configuration: {}", err
            );
        }

        if let Err(err) = sqlx::query!(
            "INSERT INTO guild_role_recovery_config (guild_id) VALUES ($1)",
            guild.as_i64()
        )
        .execute(Bot::global().postgres())
        .await
        {
            error!(
                guild_id = guild.as_u64(),
                "Failed to insert guild role recovery configuration: {}", err
            );
        }
        timing.record(start.elapsed());
    }
}
