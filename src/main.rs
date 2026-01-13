#![deny(clippy::print_stdout, clippy::print_stderr)]

use tracing::error;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

use crate::models::bot::Bot;

mod events;
mod interactions;
mod models;

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

    let mut client = Bot::init(discord_token).await?;

    if let Err(err) = client.start_autosharded().await {
        error!(error = %err, "Auto-sharded client failed to start");
        return Err(err.into());
    }

    Ok(())
}
