use serenity::all::CommandInteraction;

use crate::{
    commands::config::{ConfigError, ConfigStage},
    models::{command::CommandContext, handler::Handler},
};

pub struct ChannelBlacklistEnter;
#[async_trait::async_trait]
impl ConfigStage for ChannelBlacklistEnter {
    async fn execute(
        &self,
        _handler: &Handler,
        _ctx: &CommandContext,
        _cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        Ok(None)
    }
}

pub struct ChannelBlacklistManage;
#[async_trait::async_trait]
impl ConfigStage for ChannelBlacklistManage {
    async fn execute(
        &self,
        _handler: &Handler,
        _ctx: &CommandContext,
        _cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        Ok(None)
    }
}
