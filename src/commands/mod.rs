use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use serenity::all::{CommandInteraction, CreateCommand};

use crate::models::{context::Context, permissions::Permission, response::ResponseResult};

mod config;
mod info;
mod privacy;

#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &'static str;
    fn register(&self) -> CreateCommand;
    fn required_permission(&self) -> Option<Permission>;
    async fn router(&self, ctx: &Context<'_>, command: &CommandInteraction) -> ResponseResult;
}

fn get_command_vec() -> Vec<Arc<dyn Command>> {
    vec![
        Arc::new(privacy::PrivacyCommand),
        Arc::new(info::InfoCommand),
        Arc::new(config::ConfigCommand),
    ]
}

pub fn get_commands_available() -> HashMap<&'static str, Arc<dyn Command>> {
    let mut commands = HashMap::new();

    for command in get_command_vec() {
        commands.insert(command.name(), command);
    }

    commands
}
