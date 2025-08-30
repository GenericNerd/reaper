use serenity::all::{CommandInteraction, CreateCommand};

use crate::{
    commands::Command,
    models::{
        context::Context, permissions::Permission, response::{ResponseError, ResponseResult}
    },
};

#[derive(Debug)]
pub struct FailCommand;

#[async_trait::async_trait]
impl Command for FailCommand {
    fn name(&self) -> &'static str {
        "fail"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("fail").description("A command that always fails")
    }

    fn required_permission(&self) -> Option<Permission> {
        None
    }

    async fn router(&self, _ctx: &Context<'_>, _command: &CommandInteraction) -> ResponseResult {
        Err(ResponseError::Execution(
            "Failed on purpose".to_string(),
            None,
        ))
    }
}
