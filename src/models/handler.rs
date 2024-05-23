use std::time::Instant;

use serenity::all::{GuildId, RoleId};

#[derive(Clone)]
pub struct Handler {
    pub main_database: sqlx::PgPool,
    pub redis_database: redis::Client,
    pub start_time: Instant,
    pub global_kill_guild: GuildId,
    pub global_kill_role: RoleId,
}
