use humantime::format_duration;
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
        response::{ExecutionError, InternalError, Response, ResponseError, ResponseResult},
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
        let context = ctx.get_populated_context()?;

        let shard_runners = Bot::global().shard_manager().runners.lock().await;
        let Some(shard_latency) = shard_runners
            .get(&context.ctx.shard_id)
            .map(|runner| runner.latency)
        else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::FailedToObtainShardLatency,
            )));
        };

        let guild_count = match sqlx::query!("SELECT COUNT(guild_id) FROM moderation_configuration")
            .fetch_one(Bot::global().postgres())
            .await
        {
            Ok(record) => record.count.unwrap(),
            Err(err) => {
                return Err(ResponseError::Sqlx(Box::new(err)));
            }
        };

        let action_count = match sqlx::query!("SELECT COUNT(id) FROM actions")
            .fetch_one(Bot::global().postgres())
            .await
        {
            Ok(record) => record.count.unwrap(),
            Err(err) => {
                return Err(ResponseError::Sqlx(Box::new(err)));
            }
        };

        let giveaway_count = match sqlx::query!("SELECT COUNT(id) FROM giveaways")
            .fetch_one(Bot::global().postgres())
            .await
        {
            Ok(record) => record.count.unwrap(),
            Err(err) => {
                return Err(ResponseError::Sqlx(Box::new(err)));
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
                                context.ctx.shard_id,
                                shard_latency.map_or("Pending".to_string(),|latency| format_duration(latency).to_string())
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
                                format_duration(Bot::global().start_time().elapsed())
                            ),
                            true,
                        ),
                    ])
                    .color(0xeb_966d),
                ),
            )
            .await
            .map(|_| ())
    }
}
