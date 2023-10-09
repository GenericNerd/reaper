use crate::models::command::Command;

pub mod ban;
pub mod duration;
pub mod expire;
pub mod kick;
pub mod mute;
pub mod reason;
pub mod remove;
pub mod search;
pub mod strike;
pub mod unban;
pub mod unmute;

pub fn get_moderation_commands() -> Vec<Box<dyn Command>> {
    vec![
        Box::new(ban::BanCommand),
        Box::new(duration::DurationCommand),
        Box::new(expire::ExpireCommand),
        Box::new(kick::KickCommand),
        Box::new(mute::MuteCommand),
        Box::new(reason::ReasonCommand),
        Box::new(remove::RemoveCommand),
        Box::new(search::SearchCommand),
        Box::new(strike::StrikeCommand),
        Box::new(unban::UnbanCommand),
        Box::new(unmute::UnmuteCommand),
    ]
}
