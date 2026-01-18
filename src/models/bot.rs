use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::Instant,
};

use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use serenity::{
    Client,
    all::{GatewayIntents, ShardManager},
};

use crate::{
    events::EventRouter,
    interactions::commands::commands,
    models::interactions::traits::{CommandHandler, ComponentHandler, ModalHandler},
};

static BOT_INSTANCE: OnceLock<Bot> = OnceLock::new();

pub struct Bot {
    shard_manager: Arc<ShardManager>,
    postgres_connection: Pool<ConnectionManager<PgConnection>>,
    commands: HashMap<String, Box<dyn CommandHandler>>,
    components: HashMap<String, Box<dyn ComponentHandler>>,
    modals: HashMap<String, Box<dyn ModalHandler>>,
    start_time: Instant,
}

impl Bot {
    pub async fn init(
        token: String,
        postgres_connection: Pool<ConnectionManager<PgConnection>>,
    ) -> Result<Client, serenity::Error> {
        let event_handler = EventRouter;
        let intents = GatewayIntents::non_privileged()
            | GatewayIntents::GUILD_MEMBERS
            | GatewayIntents::MESSAGE_CONTENT;
        let client = Client::builder(token, intents)
            .event_handler(event_handler)
            .await?;

        BOT_INSTANCE.get_or_init(|| Bot {
            shard_manager: client.shard_manager.clone(),
            postgres_connection,
            commands: commands()
                .into_iter()
                .map(|command| (command.id().to_string(), command))
                .collect(),
            components: HashMap::new(),
            modals: HashMap::new(),
            start_time: Instant::now(),
        });

        Ok(client)
    }

    pub fn instance() -> &'static Bot {
        BOT_INSTANCE
            .get()
            .expect("Bot must be initialized before use")
    }

    pub fn commands(&self) -> &HashMap<String, Box<dyn CommandHandler>> {
        &self.commands
    }

    pub fn components(&self) -> &HashMap<String, Box<dyn ComponentHandler>> {
        &self.components
    }

    pub fn modals(&self) -> &HashMap<String, Box<dyn ModalHandler>> {
        &self.modals
    }

    pub fn shard_manager(&self) -> &ShardManager {
        &self.shard_manager
    }

    pub fn postgres(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        // TODO: Fix me
        self.postgres_connection.get().unwrap()
    }

    pub fn start_time(&self) -> Instant {
        self.start_time
    }
}
