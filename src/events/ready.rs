use serenity::all::{ActivityData, Command, Context, Ready};
use tracing::{Instrument, debug, error, info};

use crate::{events::EventRouter, models::bot::Bot};

impl EventRouter {
    #[tracing::instrument(skip(self, ctx, ready), fields(bot.id = %ready.user.id, guild_count = ready.guilds.len()))]
    pub async fn on_ready(&self, ctx: Context, ready: Ready) {
        let start = std::time::Instant::now();
        info!(bot.name = %ready.user.name, "Bot connected to gateway");

        ctx.set_activity(Some(ActivityData::playing("with users' emotions")));

        let mut success_count = 0;
        let mut fail_count = 0;

        for (name, command) in Bot::instance().commands() {
            // We use a sub-span for each command registration
            // This allows you to see exactly which command failed in a trace
            let span = tracing::info_span!("register_command", command.name = %name);
            match Command::create_global_command(&ctx.http, command.register())
                .instrument(span)
                .await
            {
                Ok(_) => {
                    success_count += 1;
                    debug!("Command registered successfully");
                }
                Err(e) => {
                    fail_count += 1;
                    error!(error = %e, "Failed to register command");
                }
            }
        }

        info!(
            ready_time_ms = start.elapsed().as_millis(),
            commands.total = success_count + fail_count,
            commands.success = success_count,
            commands.failed = fail_count,
            "Bot initialization complete"
        );
    }
}
