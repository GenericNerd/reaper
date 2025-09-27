use redis::Client as RedisClient;
use serenity::{
    Client,
    all::{GatewayIntents, ShardManager},
};
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::OnceCell;

use crate::{
    commands::{Command, get_commands_available},
    components::{Component, get_components_available},
    events::EventRouter,
    models::{guild::Guild, interactions::InteractionState},
};

static BOT_INSTANCE: OnceCell<Arc<Bot>> = OnceCell::const_new();

pub struct Bot {
    commands: HashMap<&'static str, Arc<dyn Command>>,
    components: HashMap<&'static str, Arc<dyn Component>>,
    shard_manager: Arc<ShardManager>,
    postgres: Arc<Pool<Postgres>>,
    redis: RedisClient,
    start_time: std::time::Instant,
    global_kill_guild: Option<Guild>,
    interaction_state: InteractionState,
}

impl Bot {
    pub async fn init(
        token: String,
        postgres: Arc<Pool<Postgres>>,
        redis: RedisClient,
        global_kill_guild: Option<Guild>,
    ) -> Result<Client, serenity::Error> {
        let event_handler = EventRouter;
        let intents = GatewayIntents::non_privileged()
            | GatewayIntents::GUILD_MEMBERS
            | GatewayIntents::MESSAGE_CONTENT;
        let client = Client::builder(token, intents)
            .event_handler(event_handler)
            .await?;

        BOT_INSTANCE
            .get_or_init(|| async {
                Arc::new(Self {
                    commands: get_commands_available(),
                    components: get_components_available(),
                    shard_manager: client.shard_manager.clone(),
                    interaction_state: InteractionState::new(postgres.clone()),
                    postgres,
                    redis,
                    start_time: std::time::Instant::now(),
                    global_kill_guild,
                })
            })
            .await;

        Ok(client)
    }

    /// Get the global bot instance
    pub fn global() -> &'static Arc<Bot> {
        BOT_INSTANCE
            .get()
            .expect("Bot must be initialized before use")
    }

    pub fn commands(&self) -> &HashMap<&'static str, Arc<dyn Command>> {
        &self.commands
    }

    pub fn components(&self) -> &HashMap<&'static str, Arc<dyn Component>> {
        &self.components
    }

    pub fn shard_manager(&self) -> &Arc<ShardManager> {
        &self.shard_manager
    }

    pub fn postgres(&self) -> &Pool<Postgres> {
        self.postgres.as_ref()
    }

    pub fn redis(&self) -> &RedisClient {
        &self.redis
    }

    pub fn start_time(&self) -> &std::time::Instant {
        &self.start_time
    }

    pub fn global_kill_guild(&self) -> Option<&Guild> {
        self.global_kill_guild.as_ref()
    }

    pub fn interaction_state(&self) -> &InteractionState {
        &self.interaction_state
    }
}
