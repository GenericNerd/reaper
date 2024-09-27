use serenity::all::{
    CommandInteraction, CommandOptionType, CreateAttachment, CreateCommand, CreateCommandOption,
};
use tracing::error;

use crate::{
    common::options::Options,
    models::{
        command::{Command, CommandContext, CommandContextReply},
        handler::Handler,
        response::{Response, ResponseError, ResponseResult},
    },
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
            .description("Get the current level and rank of a user in the server")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::User,
                    "user",
                    "The user to get the rank of",
                )
                .required(false),
            )
    }

    async fn router(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> ResponseResult {
        let options = Options {
            options: cmd.data.options(),
        };

        let res = match options.get_user("user").into_owned() {
            Some(user) => {
                let Ok(member) = ctx.ctx.http.get_member(ctx.guild.id, user.id).await else {
                    return Err(ResponseError::Execution(
                        "Failed to fetch member",
                        Some("Failed to fetch member".to_string()),
                    ));
                };
                handler.generate_rank_image(member).await?
            }
            None => {
                handler
                    .generate_rank_image(*cmd.member.clone().unwrap())
                    .await?
            }
        };

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
