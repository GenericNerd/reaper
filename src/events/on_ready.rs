use metrics::{gauge, histogram};
use serenity::all::{ActivityData, Command, Context, Ready};
use tracing::{debug, error, info};

use crate::{events::EventRouter, models::bot::Bot};

impl EventRouter {
    #[tracing::instrument(skip(ctx, ready), fields(user_name = ready.user.name))]
    pub async fn on_ready(&self, ctx: Context, ready: Ready) {
        let timing = histogram!("bot.timing.on_ready");
        let start = std::time::Instant::now();
        info!(
            "{} is connected with {} guilds",
            ready.user.name,
            ready.guilds.len()
        );
        gauge!("bot.guild_count").set(ready.guilds.len() as f64);

        ctx.set_activity(Some(ActivityData::playing("with users' emotions")));

        debug!("Registering tasks");
        tokio::spawn(async move {
            loop {
                Bot::global().interaction_state().cleanup().await;
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });

        debug!("Registering commands");
        let commands = Bot::global().commands();
        for (name, command) in commands {
            match Command::create_global_command(&ctx.http, command.register()).await {
                Ok(cmd) => info!("Registered command {}", cmd.name),
                Err(err) => error!("Failed to register command {}: {err}", name),
            }
        }

        timing.record(start.elapsed());
    }
}
