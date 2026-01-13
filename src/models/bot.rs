use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
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
    models::interactions::traits::{Command, RawComponent, RawModal},
};

static BOT_INSTANCE: OnceLock<Bot> = OnceLock::new();

pub struct Bot {
    shard_manager: Arc<ShardManager>,
    postgres_connection: Pool<ConnectionManager<PgConnection>>,
    commands: HashMap<String, Box<dyn Command>>,
    components: HashMap<String, Box<dyn RawComponent>>,
    modals: HashMap<String, Box<dyn RawModal>>,
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
            commands: HashMap::new(),
            components: HashMap::new(),
            modals: HashMap::new(),
        });

        Ok(client)
    }

    pub fn instance() -> &'static Bot {
        BOT_INSTANCE
            .get()
            .expect("Bot must be initialized before use")
    }

    pub fn commands(&self) -> &HashMap<String, Box<dyn Command>> {
        &self.commands
    }

    pub fn components(&self) -> &HashMap<String, Box<dyn RawComponent>> {
        &self.components
    }

    pub fn modals(&self) -> &HashMap<String, Box<dyn RawModal>> {
        &self.modals
    }

    pub fn shard_manager(&self) -> &ShardManager {
        &self.shard_manager
    }

    pub fn postgres(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.postgres_connection.get().unwrap()
    }
}
