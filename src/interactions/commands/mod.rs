use crate::models::interactions::traits::CommandHandler;

mod info;
mod privacy;

pub fn commands() -> Vec<Box<dyn CommandHandler>> {
    vec![
        Box::new(info::InfoCommand),
        Box::new(privacy::PrivacyCommand),
    ]
}
