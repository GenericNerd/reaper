use serenity::{prelude::{SerenityError, Context}, model::prelude::interaction::Interaction};
use tracing::error;
use crate::Handler;

impl Handler {
    pub async fn on_command(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let cmd_result: Result<(), SerenityError> = match command.data.name.as_str() {
                "permissions" => {
                    Ok(())
                },
                _ => {Ok(())}
            };
            match cmd_result {
                Ok(_) => {},
                Err(err) => {
                    error!("An error occurred while executing the {} command. The error was: {}", command.data.name, err);
                }
            }
        }
    }
}