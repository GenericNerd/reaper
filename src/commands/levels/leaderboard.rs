use serenity::all::{CommandInteraction, CreateAttachment, CreateCommand};
use tracing::error;

use crate::models::{
    command::{Command, CommandContext, CommandContextReply},
    handler::Handler,
    response::{Response, ResponseError, ResponseResult},
};

pub struct LeaderboardCommand;

#[async_trait::async_trait]
impl Command for LeaderboardCommand {
    fn name(&self) -> &'static str {
        "leaderboard"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("leaderboard")
            .dm_permission(false)
            .description("View the leaderboard for this server")
    }

    async fn router(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> ResponseResult {
        let Some(guild_id) = cmd.guild_id else {
            return Err(ResponseError::Execution(
                "Unavailable command",
                Some("Please run this command in a server".to_string()),
            ));
        };

        let res = handler
            .generate_leaderboard_image(&ctx.ctx, guild_id.get() as i64)
            .await?;

        let Ok(attachment) = CreateAttachment::path(res.clone()).await else {
            return Err(ResponseError::Execution(
                "Failed to get attachment",
                Some("Failed to get attachment".to_string()),
            ));
        };

        ctx.reply(cmd, Response::new().attachments(attachment))
            .await?;

        if let Err(err) = tokio::fs::remove_file(res).await {
            error!("Failed to remove image file: {:?}", err);
            return Err(ResponseError::Execution(
                "Failed to remove image file from disk",
                Some("Failed to remove image file from disk".to_string()),
            ));
        }

        Ok(())
    }
}
