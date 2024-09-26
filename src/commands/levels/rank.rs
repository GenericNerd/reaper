use serenity::all::{CommandInteraction, CreateAttachment, CreateCommand};
use tracing::error;

use crate::models::{
    command::{Command, CommandContext, CommandContextReply},
    handler::Handler,
    response::{Response, ResponseError, ResponseResult},
};

pub struct RankCommand;

#[async_trait::async_trait]
impl Command for RankCommand {
    fn name(&self) -> &'static str {
        "rank"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("rank")
            .dm_permission(false)
            .description("Get your current level and rank in the server")
    }

    async fn router(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> ResponseResult {
        let res = handler.generate_image(*cmd.member.clone().unwrap()).await?;

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
        };

        Ok(())
    }
}
