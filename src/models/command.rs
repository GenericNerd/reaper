use std::sync::atomic::AtomicBool;

use serenity::{
    all::{CommandInteraction, Message, PartialGuild},
    builder::CreateCommand,
    prelude::Context as IncomingContext,
};

use super::{
    handler::Handler,
    permissions::Permission,
    response::{Response, ResponseError, ResponseResult},
};

#[async_trait::async_trait]
pub trait CommandContextReply {
    async fn reply(&self, cmd: &CommandInteraction, response: Response) -> ResponseResult;
    async fn reply_get_message(
        &self,
        cmd: &CommandInteraction,
        response: Response,
    ) -> Result<Message, ResponseError>;
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
    ) -> ResponseResult;
}
