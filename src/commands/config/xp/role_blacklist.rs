use serenity::all::CommandInteraction;

use crate::{
    commands::config::{ConfigError, ConfigStage},
    models::{command::CommandContext, handler::Handler},
};

pub struct RoleBlacklistEnter;
#[async_trait::async_trait]
impl ConfigStage for RoleBlacklistEnter {
    async fn execute(
        &self,
        _handler: &Handler,
        _ctx: &CommandContext,
        _cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        Ok(None)
    }
}

pub struct RoleBlacklistManage;
#[async_trait::async_trait]
impl ConfigStage for RoleBlacklistManage {
    async fn execute(
        &self,
        _handler: &Handler,
        _ctx: &CommandContext,
        _cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        Ok(None)
    }
}
