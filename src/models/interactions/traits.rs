use async_trait::async_trait;
use serenity::all::{CommandInteraction, ComponentInteraction, CreateCommand, ModalInteraction};

use crate::models::{
    interactions::context::Context, permission::Permission, response::ResponseResult,
};

pub trait InteractionHandler: Send + Sync {
    fn id(&self) -> &'static str;
    fn permission(&self) -> Option<Permission>;
}

#[async_trait]
pub trait Command: InteractionHandler {
    fn register(&self) -> CreateCommand;
    async fn execute(&self, ctx: Context, command: &CommandInteraction) -> ResponseResult<()>;
}

#[async_trait]
pub trait Component: InteractionHandler {
    async fn execute(&self, ctx: Context, component: &ComponentInteraction) -> ResponseResult<()>;
}

#[async_trait]
pub trait Modal: InteractionHandler {
    async fn execute(&self, ctx: Context, modal: &ModalInteraction) -> ResponseResult<()>;
}
