use std::time::Instant;

#[derive(Clone)]
pub struct Handler {
    pub main_database: sqlx::PgPool,
    pub redis_database: redis::Client,
    pub start_time: Instant,
}
