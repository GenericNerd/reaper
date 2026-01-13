#![deny(clippy::print_stdout, clippy::print_stderr)]

use diesel::{
    Connection, PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use tracing::error;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

use crate::models::{
    bot::Bot,
    permission::Permission,
    serenity::{guild::Guild, user::User},
};

mod events;
mod interactions;
mod models;
mod schema;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let axiom_layer = tracing_axiom::default("reaper")?;
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(axiom_layer)
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let discord_token = std::env::var("DISCORD_TOKEN")?;
    let database_url = std::env::var("DATABASE_URL")?;

    let mut postgres_connection = PgConnection::establish(&database_url)?;
    postgres_connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| e as Box<dyn std::error::Error>)?;
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = Pool::builder().test_on_check_out(true).build(manager)?;

    let mut client = Bot::init(discord_token, pool).await?;

    if let Err(err) = client.start_autosharded().await {
        error!(error = %err, "Auto-sharded client failed to start");
        return Err(err.into());
    }

    Ok(())
}
