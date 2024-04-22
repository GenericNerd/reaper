use std::sync::{atomic::AtomicBool, Arc};

use serenity::{all::ActionExecution, prelude::Context};
use tracing::error;

use crate::models::{command::CommandContext, handler::Handler};

impl Handler {
    pub async fn on_automod_trigger(&self, ctx: Context, execution: ActionExecution) {
        let Ok(rule) = ctx
            .http
            .get_automod_rule(execution.guild_id, execution.rule_id)
            .await
        else {
            error!(
                "Could not get automod rule {} from guild {}",
                execution.rule_id, execution.guild_id
            );
            return;
        };

        if !rule.name.to_ascii_lowercase().contains("strike") {
            return;
        }

        let Ok(guild) = execution.guild_id.to_partial_guild(&ctx.http).await else {
            error!("Could not get guild {} from cache", execution.guild_id);
            return;
        };

        let context = CommandContext {
            ctx,
            has_responsed: Arc::new(AtomicBool::new(false)),
            user_permissions: vec![],
            guild,
        };

        if let Err(err) = Box::pin(self.strike_user(
            &context,
            context.guild.id.get() as i64,
            execution.user_id.get() as i64,
            format!("Violated \"{}\" automod rule", rule.name),
            None,
            None,
        ))
        .await
        {
            error!("Could not strike user. Failed with error: {:?}", err);
        }
    }
}
