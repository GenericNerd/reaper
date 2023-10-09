use crate::{
    commands::{moderation::get_moderation_commands, permissions::PermissionsCommand},
    models::command::Command,
};

pub mod moderation;
pub mod permissions;

pub fn get_command_list() -> Vec<Box<dyn Command>> {
    let mut commands = get_moderation_commands();
    commands.push(Box::new(PermissionsCommand));

    commands
}
