use metrics::{counter, histogram};
use serenity::all::{ActionExecution, Context};
use tracing::error;

use crate::{events::EventRouter, models::guild::Guild};

impl EventRouter {
    #[tracing::instrument(skip(ctx, execution), fields(rule_id = execution.rule_id.get()))]
    pub async fn automod_action_execution(&self, ctx: Context, execution: ActionExecution) {
        let timing = histogram!("bot.timing.automod_action_execution");
        let start = std::time::Instant::now();
        let guild = Guild::from(execution.guild_id.get());
        let Ok(rule) = ctx
            .http
            .get_automod_rule(guild.as_serenity_id(), execution.rule_id)
            .await
        else {
            error!(
                guild_id = guild.as_u64(),
                rule_id = execution.rule_id.get(),
                "Failed to fetch automod rule",
            );
            return;
        };

        if !rule.name.to_ascii_lowercase().contains("strike") {
            return;
        }
        counter!("bot.automod.strike_count").increment(1);

        // TODO: Strike user
        timing.record(start.elapsed());
    }
}
