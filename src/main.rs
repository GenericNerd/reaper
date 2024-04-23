#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::unreadable_literal)]

use serenity::{prelude::GatewayIntents, Client};
use sqlx::postgres::PgPoolOptions;
use std::{env, time::Instant};
use tracing::{error, info};

mod commands;
mod common;
mod database;
mod events;
mod models;

#[tokio::main]
async fn main() {
    println!("cargo:rerun-if-changed=migrations");

    let log_level = match env::var("DEBUG").unwrap_or(false.to_string()).as_str() {
        "true" => tracing::Level::DEBUG,
        _ => tracing::Level::INFO,
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    info!("Getting environment variables");
    let discord_token = env::var("DISCORD_TOKEN").unwrap();
    let main_db_username = env::var("DB_USER").unwrap_or("postgres".to_string());
    let main_db_password = env::var("DB_PASSWORD").unwrap();
    let main_db_host = env::var("DB_HOST").unwrap_or("localhost".to_string());
    let main_db_port = env::var("DB_PORT").unwrap_or("5432".to_string());
    let main_db_name = env::var("DB_NAME").unwrap_or("postgres".to_string());
    let redis_db_host = env::var("REDIS_HOST").unwrap_or("redis".to_string());
    let redis_db_port = env::var("REDIS_PORT").unwrap_or("6379".to_string());
    let redis_db_password = env::var("REDIS_PASSWORD").unwrap();
    let global_kill_guild =
        env::var("GLOBAL_KILL_GUILD").unwrap_or("1041788629250482208".to_string());
    let global_kill_role =
        env::var("GLOBAL_KILL_ROLE").unwrap_or("1232127614072918108".to_string());

    // Main database connection
    let connection_url = format!(
        "postgres://{main_db_username}:{main_db_password}@{main_db_host}:{main_db_port}/{main_db_name}"
    );
    info!("Establishing connection to main database");
    let main_database = PgPoolOptions::new().connect(&connection_url).await.unwrap();
    info!("Running outstanding migrations");
    sqlx::migrate!().run(&main_database).await.unwrap();
    info!("Connected to main database");

    // Revive all previously killed features
    info!("Reviving all previously killed features");
    sqlx::query!("UPDATE global_kills SET active = true")
        .execute(&main_database)
        .await
        .unwrap();

    // Redis database connection
    let redis_connection_url =
        format!("redis://:{redis_db_password}@{redis_db_host}:{redis_db_port}/");
    info!("Establishing connection to Redis database");
    let redis_database = redis::Client::open(redis_connection_url).unwrap();
    info!("Connected to Redis database");

    // Discord client connection
    let handler = models::handler::Handler {
        main_database,
        redis_database,
        start_time: Instant::now(),
        global_kill_guild: global_kill_guild.parse().unwrap(),
        global_kill_role: global_kill_role.parse().unwrap(),
    };
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&discord_token, intents)
        .event_handler(handler)
        .await
        .unwrap();

    if let Err(err) = client.start_autosharded().await {
        error!(
            "Attempted to start Reaper Discord client, but failed with error: {}",
            err
        );
    }
}
