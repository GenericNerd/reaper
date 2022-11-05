use std::env;
use serenity::{prelude::GatewayIntents, Client, framework::StandardFramework};
use tracing::error;

mod mongo;
mod commands;
mod events;

pub struct Handler {
    pub database: mongo::mongo::Database
}

#[tokio::main]
async fn main() {
    let token = match env::var("DISCORD_TOKEN") {
        Ok(token) => token,
        Err(err) => {
            error!("A token could not be retrieved from the environment. The error was: {}", err);
            return;
        }
    };
    let intents = GatewayIntents::non_privileged() | GatewayIntents::GUILD_MEMBERS | GatewayIntents::MESSAGE_CONTENT;

    let database = match mongo::mongo::connect().await {
        Ok(database) => database,
        Err(err) => {
            error!("A database connection could not be established. The error was: {}", err);
            return;
        }
    };

    let mut client = match Client::builder(&token, intents)
        .event_handler(Handler { database: database })
        .framework(StandardFramework::new()
            .configure(|c| c.with_whitespace(true).prefix(">>")))
        .await {
        Ok(client) => client,
        Err(err) => {
            error!("A client could not be created. The error was: {}", err);
            return;
        }
    };

    if let Err(err) = client.start().await {
        error!("The client could not be started. The error was: {}", err);
    }
}
