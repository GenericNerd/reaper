use std::sync::Arc;

use serenity::{
    all::CommandInteraction,
    builder::{
        CreateCommand, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
        EditInteractionResponse,
    },
    gateway::ShardManager,
    prelude::TypeMapKey,
};

use crate::models::{
    command::{Command, CommandContext},
    handler::Handler,
    response::{ResponseError, ResponseResult},
};

pub struct InfoCommand;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

#[async_trait::async_trait]
impl Command for InfoCommand {
    fn name(&self) -> &'static str {
        "info"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("info").description("Get information about the bot")
    }

    async fn router(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> ResponseResult {
        let start = std::time::Instant::now();
        if let Err(err) = cmd
            .create_response(
                &ctx.ctx.http,
                CreateInteractionResponse::Defer(CreateInteractionResponseMessage::default()),
            )
            .await
        {
            return Err(ResponseError::Serenity(err));
        }
        let latency = start.elapsed();

        let guild_count = match sqlx::query!("SELECT COUNT(guild_id) FROM moderation_configuration")
            .fetch_one(&handler.main_database)
            .await
        {
            Ok(record) => record.count.unwrap(),
            Err(err) => {
                return Err(ResponseError::Execution(
                    "Failed to get guild count",
                    Some(err.to_string()),
                ))
            }
        };

        let action_count = match sqlx::query!("SELECT COUNT(id) FROM actions")
            .fetch_one(&handler.main_database)
            .await
        {
            Ok(record) => record.count.unwrap(),
            Err(err) => {
                return Err(ResponseError::Execution(
                    "Failed to get guild count",
                    Some(err.to_string()),
                ))
            }
        };

        let giveaway_count = match sqlx::query!("SELECT COUNT(id) FROM giveaways")
            .fetch_one(&handler.main_database)
            .await
        {
            Ok(record) => record.count.unwrap(),
            Err(err) => {
                return Err(ResponseError::Execution(
                    "Failed to get guild count",
                    Some(err.to_string()),
                ))
            }
        };

        if let Err(err) = cmd
            .edit_response(
                &ctx.ctx.http,
                EditInteractionResponse::new().embed(
                    CreateEmbed::new()
                        .title("Reaper Information")
                        .fields(vec![
                            (
                                "Network",
                                format!(
                                    "Shard ID {}\nLatency: {}ms",
                                    ctx.ctx.shard_id,
                                    latency.as_millis()
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
                                    pretty_duration::pretty_duration(
                                        &start.elapsed(),
                                        None
                                    )
                                ),
                                true,
                            ),
                        ])
                        .color(0xeb966d),
                ),
            )
            .await
        {
            return Err(ResponseError::Serenity(err));
        };

        Ok(())
    }
}
