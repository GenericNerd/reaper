use serenity::all::{CommandInteraction, CreateCommand};

use crate::models::{
    command::{Command, CommandContext},
    handler::Handler,
    response::ResponseResult,
};

pub struct XPCommand;

#[async_trait::async_trait]
impl Command for XPCommand {
    fn name(&self) -> &'static str {
        "xp"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("xp")
            .dm_permission(false)
            .description("Set, reset or add to someone's XP")
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
