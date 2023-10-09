use serenity::all::UnavailableGuild;

use crate::models::handler::Handler;

impl Handler {
    pub async fn on_guild_leave(&self, guild: UnavailableGuild) {
        sqlx::query!(
            "DELETE FROM moderation_configuration WHERE guild_id = $1",
            guild.id.0.get() as i64
        )
        .execute(&self.main_database)
        .await
        .expect("Failed to delete moderation configuration for guild");

        sqlx::query!(
            "DELETE FROM logging_configuration WHERE guild_id = $1",
            guild.id.0.get() as i64
        )
        .execute(&self.main_database)
        .await
        .expect("Failed to delete logging configuration for guild");
    }
}
