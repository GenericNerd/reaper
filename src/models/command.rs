use std::sync::atomic::AtomicBool;

use serenity::{
    all::{CommandInteraction, PartialGuild},
    builder::CreateCommand,
    prelude::Context as IncomingContext,
    Error as SerenityError,
};

use super::{handler::Handler, permissions::Permission, response::Response};

#[derive(Debug)]
pub enum CommandError {
    SerenityError(SerenityError),
    ExecutionError(&'static str),
    Other(Box<dyn std::error::Error + Sync + Send>),
}

pub type CommandResult = Result<(), CommandError>;

#[async_trait::async_trait]
pub trait Context {
    async fn reply(&self, cmd: &CommandInteraction, response: Response) -> CommandResult;
}

pub struct CommandContext {
    pub ctx: IncomingContext,
    pub has_responsed: AtomicBool,
    pub user_permissions: Vec<Permission>,
    pub guild: PartialGuild,
}

pub struct FailedCommandContext {
    pub ctx: IncomingContext,
}

#[async_trait::async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &'static str;
    fn register(&self) -> CreateCommand;
    async fn router(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        command: &CommandInteraction,
    ) -> CommandResult;
}
