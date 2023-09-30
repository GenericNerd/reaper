use crate::models::command::Command;

pub mod ban;
pub mod kick;
pub mod strike;

pub fn get_moderation_commands() -> Vec<Box<dyn Command>> {
    vec![
        Box::new(strike::StrikeCommand),
        Box::new(kick::KickCommand),
        Box::new(ban::BanCommand),
    ]
}
