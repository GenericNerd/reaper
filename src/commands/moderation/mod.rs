use crate::models::command::Command;

pub mod strike;

pub fn get_moderation_commands() -> Vec<Box<dyn Command>> {
    vec![Box::new(strike::StrikeCommand)]
}
