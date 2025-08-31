#![deny(clippy::all)]
#![warn(clippy::pedantic)]
use std::{net::SocketAddr, sync::Arc};

use metrics_exporter_prometheus::PrometheusBuilder;
use redis::Client;
use sqlx::postgres::PgPoolOptions;
use tokio::{io::AsyncWriteExt, net::TcpListener};
use tracing::{debug, error, info};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod commands;
mod components;
mod events;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=migrations");
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive("info".parse()?))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!("Fetching environment variables");
    let prometheus_port = std::env::var("PROMETHEUS_PORT").unwrap_or("9000".to_string());
    let db_username = std::env::var("DB_USER").unwrap_or("postgres".to_string());
    let db_password = std::env::var("DB_PASSWORD")?;
    let db_name = std::env::var("DB_NAME").unwrap_or("reaper".to_string());
    let db_host = std::env::var("DB_HOST").unwrap_or("database".to_string());
    let redis_db_password = std::env::var("REDIS_PASSWORD")?;
    let redis_db_host = std::env::var("REDIS_HOST").unwrap_or("redis".to_string());
    let discord_token = std::env::var("DISCORD_TOKEN")?;

    info!("Starting Prometheus service");
    let prometheus_builder = PrometheusBuilder::new();
    let metric_recorder = prometheus_builder.build_recorder();
    let metric_handle = metric_recorder.handle();
    metrics::set_global_recorder(metric_recorder)
        .map_err(|_| "Failed to set global Prometheus recorder")?;

    let prometheus_addr = format!("0.0.0.0:{prometheus_port}").parse::<SocketAddr>()?;
    info!("Prometheus server listening on {}", prometheus_addr);
    let prometheus_listener = TcpListener::bind(prometheus_addr).await?;

    // Prometheus server
    tokio::spawn(async move {
        let span = tracing::span!(tracing::Level::INFO, "prometheus_server");
        let _enter = span.enter();
        loop {
            match prometheus_listener.accept().await {
                Ok((mut stream, peer_addr)) => {
                    debug!("Received connection from {} for metrics", peer_addr);
                    let metric_families = metric_handle.render();
                    let response = format!(
                        "HTTP/1.1 200 OK\nContent-Type: text/plain; version={}; charset=utf-8\nContent-Length: {}\n\n{}",
                        env!("CARGO_PKG_VERSION"),
                        metric_families.len(),
                        metric_families
                    );
                    if let Err(e) = stream.write_all(response.as_bytes()).await {
                        error!("Failed to write response to {}: {}", peer_addr, e);
                    }
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    });

    info!("Initialising Postgres database connection");
    let connection_url = format!("postgres://{db_username}:{db_password}@{db_host}/{db_name}");
    let connection_pool = PgPoolOptions::new().connect(&connection_url).await?;
    info!("Running outstanding migrations");
    sqlx::migrate!().run(&connection_pool).await?;

    info!("Initialising Redis database connection");
    let connection_url = format!("redis://:{redis_db_password}@{redis_db_host}/");
    let redis_database = Client::open(connection_url)?;
    info!("Redis database connection established");

    let mut client = models::bot::Bot::init(
        discord_token,
        Arc::new(connection_pool),
        redis_database,
        None,
    )
    .await?;

    info!("Starting client");
    if let Err(e) = client.start_autosharded().await {
        error!("Attempted to start client, but failed with error: {}", e);
    }
    Ok(())
}
