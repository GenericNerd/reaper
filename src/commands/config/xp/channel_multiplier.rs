use serenity::all::CommandInteraction;

use crate::{
    commands::config::{ConfigError, ConfigStage},
    models::{command::CommandContext, handler::Handler},
};

pub struct ChannelMultiplierEnter;
#[async_trait::async_trait]
impl ConfigStage for ChannelMultiplierEnter {
    async fn execute(
        &self,
        _handler: &Handler,
        _ctx: &CommandContext,
        _cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        Ok(None)
    }
}

pub struct ChannelMultiplierManage;
#[async_trait::async_trait]
impl ConfigStage for ChannelMultiplierManage {
    async fn execute(
        &self,
        _handler: &Handler,
        _ctx: &CommandContext,
        _cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        Ok(None)
    }
}
