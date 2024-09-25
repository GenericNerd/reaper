use serenity::all::{CommandInteraction, CreateCommand};

use crate::models::{
    command::{Command, CommandContext},
    handler::Handler,
    response::ResponseResult,
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
        _handler: &Handler,
        _ctx: &CommandContext,
        _cmd: &CommandInteraction,
    ) -> ResponseResult {
        Ok(())
    }
}
