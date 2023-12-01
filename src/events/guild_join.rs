use serenity::all::Guild;

use crate::models::handler::Handler;

impl Handler {
    pub async fn on_guild_create(&self, guild: Guild) {
        sqlx::query!(
            "INSERT INTO moderation_configuration (guild_id) VALUES ($1)",
            guild.id.get() as i64
        )
        .execute(&self.main_database)
        .await
        .expect("Failed to insert moderation configuration for guild");

        sqlx::query!(
            "INSERT INTO logging_configuration (guild_id) VALUES ($1)",
            guild.id.get() as i64
        )
        .execute(&self.main_database)
        .await
        .expect("Failed to insert logging configuration for guild");

        sqlx::query!(
            "INSERT INTO guild_role_recovery_config (guild_id) VALUES ($1)",
            guild.id.get() as i64
        )
        .execute(&self.main_database)
        .await
        .expect("Failed to insert guild role recovery config for guild");
    }
}
