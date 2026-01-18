use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serenity::all::{CommandInteraction, ComponentInteraction, CreateCommand, ModalInteraction};

use crate::models::{
    interactions::{context::Context, responder::Responder},
    permission::Permission,
    response::ResponseResult,
};

#[async_trait]
pub trait CommandHandler: Send + Sync {
    fn id(&self) -> &'static str;
    fn permission(&self) -> Option<Permission>;
    fn register(&self) -> CreateCommand;
    async fn execute(&self, ctx: &Context, command: &CommandInteraction) -> ResponseResult<()>;
}

#[async_trait]
pub trait Command: Send + Sync {
    const ID: &'static str;
    const PERMISSION: Option<Permission>;

    fn register(&self) -> CreateCommand;
    async fn execute(&self, ctx: &Context, command: &CommandInteraction) -> ResponseResult<()>;
}

#[async_trait]
pub trait ComponentHandler: Send + Sync {
    fn id(&self) -> &'static str;
    fn permission(&self) -> Option<Permission>;
    async fn execute(
        &self,
        ctx: &Context,
        component: &ComponentInteraction,
        state: serde_json::Value,
    ) -> ResponseResult<()>;
}

#[async_trait]
pub trait Component: Send + Sync {
    const ID: &'static str;
    const PERMISSION: Option<Permission>;

    type State: DeserializeOwned + Send + Sync;

    async fn execute(
        &self,
        ctx: &Context,
        interaction: &impl Responder,
        state: Self::State,
    ) -> ResponseResult<()>;
}

#[async_trait]
pub trait ModalHandler: Send + Sync {
    fn id(&self) -> &'static str;
    fn permission(&self) -> Option<Permission>;
    async fn execute(
        &self,
        ctx: &Context,
        modal: &ModalInteraction,
        state: serde_json::Value,
    ) -> ResponseResult<()>;
}

#[async_trait]
pub trait Modal: Send + Sync {
    const ID: &'static str;
    const PERMISSION: Option<Permission>;

    type State: DeserializeOwned + Send + Sync;

    async fn execute(
        &self,
        ctx: &Context,
        interaction: &impl Responder,
        state: Self::State,
    ) -> ResponseResult<()>;
}

#[async_trait]
impl<T: Command> CommandHandler for T {
    fn id(&self) -> &'static str {
        T::ID
    }
    fn permission(&self) -> Option<Permission> {
        T::PERMISSION
    }
    fn register(&self) -> CreateCommand {
        <Self as Command>::register(self)
    }
    async fn execute(&self, ctx: &Context, command: &CommandInteraction) -> ResponseResult<()> {
        <Self as Command>::execute(self, ctx, command).await
    }
}

#[async_trait]
impl<T: Component> ComponentHandler for T {
    fn id(&self) -> &'static str {
        T::ID
    }
    fn permission(&self) -> Option<Permission> {
        T::PERMISSION
    }
    async fn execute(
        &self,
        ctx: &Context,
        component: &ComponentInteraction,
        state: serde_json::Value,
    ) -> ResponseResult<()> {
        let typed_state: T::State = serde_json::from_value(state)?;
        <Self as Component>::execute(self, ctx, component, typed_state).await
    }
}

#[async_trait]
impl<T: Modal> ModalHandler for T {
    fn id(&self) -> &'static str {
        T::ID
    }
    fn permission(&self) -> Option<Permission> {
        T::PERMISSION
    }
    async fn execute(
        &self,
        ctx: &Context,
        modal: &ModalInteraction,
        state: serde_json::Value,
    ) -> ResponseResult<()> {
        let typed_state: T::State = serde_json::from_value(state)?;
        <Self as Modal>::execute(self, ctx, modal, typed_state).await
    }
}
