use crate::models::command::Command;

pub mod config;
pub mod giveaway;
pub mod moderation;
pub mod permissions;

pub fn get_command_list() -> Vec<Box<dyn Command>> {
    let mut commands = moderation::get_moderation_commands();
    commands.push(Box::new(permissions::PermissionsCommand));
    commands.push(Box::new(giveaway::GiveawayCommand));
    commands.push(Box::new(config::ConfigCommand));

    commands
}
