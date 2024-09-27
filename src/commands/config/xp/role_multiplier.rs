use serenity::all::CommandInteraction;

use crate::{
    commands::config::{ConfigError, ConfigStage},
    models::{command::CommandContext, handler::Handler},
};

pub struct RoleMultiplierEnter;
#[async_trait::async_trait]
impl ConfigStage for RoleMultiplierEnter {
    async fn execute(
        &self,
        _handler: &Handler,
        _ctx: &CommandContext,
        _cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        Ok(None)
    }
}

pub struct RoleMultiplierManage;
#[async_trait::async_trait]
impl ConfigStage for RoleMultiplierManage {
    async fn execute(
        &self,
        _handler: &Handler,
        _ctx: &CommandContext,
        _cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        Ok(None)
    }
}
