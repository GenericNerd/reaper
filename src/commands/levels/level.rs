use serenity::all::{CommandInteraction, CreateCommand};

use crate::models::{
    command::{Command, CommandContext},
    handler::Handler,
    response::ResponseResult,
};

pub struct LevelCommand;

#[async_trait::async_trait]
impl Command for LevelCommand {
    fn name(&self) -> &'static str {
        "level"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("level")
            .dm_permission(false)
            .description("Set, reset or add to someone's level")
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
