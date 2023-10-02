use crate::models::command::Command;

pub mod ban;
pub mod kick;
pub mod mute;
pub mod search;
pub mod strike;

pub fn get_moderation_commands() -> Vec<Box<dyn Command>> {
    vec![
        Box::new(ban::BanCommand),
        Box::new(kick::KickCommand),
        Box::new(mute::MuteCommand),
        Box::new(search::SearchCommand),
        Box::new(strike::StrikeCommand),
    ]
}
