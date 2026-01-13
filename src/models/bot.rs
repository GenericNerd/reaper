use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use serenity::{
    Client,
    all::{GatewayIntents, ShardManager},
};

use crate::{
    events::EventRouter,
    models::interactions::traits::{Command, Component, Modal},
};

static BOT_INSTANCE: OnceLock<Bot> = OnceLock::new();

pub struct Bot {
    shard_manager: Arc<ShardManager>,
    commands: HashMap<String, Box<dyn Command>>,
    components: HashMap<String, Box<dyn Component>>,
    modals: HashMap<String, Box<dyn Modal>>,
}

impl Bot {
    pub async fn init(token: String) -> Result<Client, serenity::Error> {
        let event_handler = EventRouter;
        let intents = GatewayIntents::non_privileged()
            | GatewayIntents::GUILD_MEMBERS
            | GatewayIntents::MESSAGE_CONTENT;
        let client = Client::builder(token, intents)
            .event_handler(event_handler)
            .await?;

        BOT_INSTANCE.get_or_init(|| Bot {
            shard_manager: client.shard_manager.clone(),
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

    pub fn components(&self) -> &HashMap<String, Box<dyn Component>> {
        &self.components
    }

    pub fn modals(&self) -> &HashMap<String, Box<dyn Modal>> {
        &self.modals
    }
}
