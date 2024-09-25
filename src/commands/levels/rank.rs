use serenity::all::{CommandInteraction, CreateCommand};

use crate::models::{
    command::{Command, CommandContext},
    handler::Handler,
    response::ResponseResult,
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
        _handler: &Handler,
        _ctx: &CommandContext,
        _cmd: &CommandInteraction,
    ) -> ResponseResult {
        Ok(())
    }
}
