use humanize_duration::prelude::DurationExt;
use serenity::{
    all::CommandInteraction,
    builder::{CreateCommand, CreateEmbed},
};

use crate::{
    commands::Command,
    models::{
        bot::Bot,
        context::{Context, ContextReply},
        permissions::Permission,
        response::{Response, ResponseError, ResponseResult},
    },
};

pub struct InfoCommand;

#[async_trait::async_trait]
impl Command for InfoCommand {
    fn name(&self) -> &'static str {
        "info"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("info").description("Get information about the bot")
    }

    fn required_permission(&self) -> Option<Permission> {
        None
    }

    async fn router(&self, ctx: &Context<'_>, cmd: &CommandInteraction) -> ResponseResult {
        let Context::Populated(populated_ctx) = ctx else {
            return Err(ResponseError::Execution("Failed to obtain context".to_string(), Some("An internal part of Reaper failed to provide adequate context. Please report this to the support server.".to_string())));
        };

        let shard_runners = Bot::global().shard_manager().runners.lock().await;
        let Some(shard_latency) = shard_runners
            .get(&populated_ctx.ctx.shard_id)
            .map(|runner| runner.latency)
        else {
            return Err(ResponseError::Execution("Failed to obtain shard latency".to_string(), Some("An internal part of Reaper failed to provide adequate context. Please report this to the support server.".to_string())));
        };

        let guild_count = match sqlx::query!("SELECT COUNT(guild_id) FROM moderation_configuration")
            .fetch_one(Bot::global().postgres())
            .await
        {
            Ok(record) => record.count.unwrap(),
            Err(err) => {
                return Err(ResponseError::Sqlx(err));
            }
        };

        let action_count = match sqlx::query!("SELECT COUNT(id) FROM actions")
            .fetch_one(Bot::global().postgres())
            .await
        {
            Ok(record) => record.count.unwrap(),
            Err(err) => {
                return Err(ResponseError::Sqlx(err));
            }
        };

        let giveaway_count = match sqlx::query!("SELECT COUNT(id) FROM giveaways")
            .fetch_one(Bot::global().postgres())
            .await
        {
            Ok(record) => record.count.unwrap(),
            Err(err) => {
                return Err(ResponseError::Sqlx(err));
            }
        };

        ctx.reply(
            cmd,
            Response::new().embed(
                CreateEmbed::new()
                    .title("Reaper Information")
                    .fields(vec![
                        (
                            "Network",
                            format!(
                                "Shard ID {}\nLatency: {}",
                                populated_ctx.ctx.shard_id,
                                shard_latency.map_or("Pending".to_string(),|latency| latency.human(humanize_duration::Truncate::Millis).to_string())
                            ),
                            true,
                        ),
                        (
                            "Information",
                            format!(
                                "Serving {guild_count} guilds\nHandled {action_count} actions\nRunning {giveaway_count} giveaways"
                            ),
                            true,
                        ),
                        (
                            "Meta",
                            format!(
                                "Version: {}\nUptime: {}",
                                env!("CARGO_PKG_VERSION"),
                                Bot::global().start_time().elapsed().human(humanize_duration::Truncate::Millis)
                            ),
                            true,
                        ),
                    ])
                    .color(0xeb966d),
                ),
            )
            .await
            .map(|_| ())
    }
}
