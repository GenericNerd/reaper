use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serenity::all::{CommandInteraction, ComponentInteraction, CreateCommand, ModalInteraction};

use crate::models::{
    interactions::{context::Context, responder::Responder},
    permission::Permission,
    response::ResponseResult,
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
pub trait RawComponent: InteractionHandler {
    async fn execute_raw(
        &self,
        ctx: Context,
        component: &ComponentInteraction,
        state: serde_json::Value,
    ) -> ResponseResult<()>;
}

#[async_trait]
pub trait Component: InteractionHandler {
    type State: DeserializeOwned + Send + Sync;

    async fn execute(
        &self,
        ctx: Context,
        interaction: &impl Responder,
        state: Self::State,
    ) -> ResponseResult<()>;
}

#[async_trait]
impl<T: Component + Send + Sync> RawComponent for T {
    async fn execute_raw(
        &self,
        ctx: Context,
        component: &ComponentInteraction,
        state: serde_json::Value,
    ) -> ResponseResult<()> {
        let typed_state: T::State = serde_json::from_value(state)?;
        self.execute(ctx, component, typed_state).await
    }
}

#[async_trait]
pub trait RawModal: InteractionHandler {
    async fn execute_raw(
        &self,
        ctx: Context,
        modal: &ModalInteraction,
        state: serde_json::Value,
    ) -> ResponseResult<()>;
}

#[async_trait]
pub trait Modal: InteractionHandler {
    type State: DeserializeOwned + Send + Sync;

    async fn execute(
        &self,
        ctx: Context,
        interaction: &impl Responder,
        state: Self::State,
    ) -> ResponseResult<()>;
}

#[async_trait]
impl<T: Modal + Send + Sync> RawModal for T {
    async fn execute_raw(
        &self,
        ctx: Context,
        modal: &ModalInteraction,
        state: serde_json::Value,
    ) -> ResponseResult<()> {
        let typed_state: T::State = serde_json::from_value(state)?;
        self.execute(ctx, modal, typed_state).await
    }
}
