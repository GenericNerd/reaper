use crate::models::command::Command;

pub mod config;
pub mod giveaway;
pub mod global;
pub mod info;
pub mod moderation;
pub mod permissions;
pub mod privacy;

pub fn get_command_list() -> Vec<Box<dyn Command>> {
    let mut commands = moderation::get_moderation_commands();
    commands.push(Box::new(permissions::PermissionsCommand));
    commands.push(Box::new(giveaway::GiveawayCommand));
    commands.push(Box::new(config::ConfigCommand));
    commands.push(Box::new(info::InfoCommand));
    commands.push(Box::new(privacy::PrivacyCommand));

    commands
}
