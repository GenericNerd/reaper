use std::sync::Arc;

use serenity::{
    all::CommandInteraction,
    builder::{CreateCommand, CreateEmbed, CreateMessage, EditInteractionResponse},
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
        let message = match cmd
            .channel_id
            .send_message(&ctx.ctx, CreateMessage::new().content("..."))
            .await
        {
            Ok(message) => message,
            Err(err) => {
                return Err(ResponseError::Serenity(err));
            }
        };
        let end = std::time::Instant::now();
        if let Err(err) = message.delete(&ctx.ctx).await {
            return Err(ResponseError::Serenity(err));
        }

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
                                    "Shard ID {}\nLatency: {}",
                                    ctx.ctx.shard_id,
                                    pretty_duration::pretty_duration(&(end - start), None)
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
                                        &handler.start_time.elapsed(),
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
