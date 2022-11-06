use serenity::{prelude::{Context}, model::{prelude::{interaction::Interaction, Guild, Member}, permissions}};
use tracing::error;
use crate::{Handler, commands, mongo::structs::Permissions};

use super::structs::CommandError;

impl Handler {
    pub async fn on_command(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let cmd_result: Result<(), CommandError> = match command.data.name.as_str() {
                "permissions" => {
                    commands::permissions::router::route(self, &ctx, &command).await
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

    pub async fn has_permission(&self, guild: &Guild, member: &Member, permission: Permissions) -> Result<bool, CommandError> {
        if member.user.id == guild.owner_id {
            return Ok(true);
        }
        if member.permissions.is_some() {
            if member.permissions.unwrap().contains(permissions::Permissions::ADMINISTRATOR) {
                return Ok(true);
            }
        }
        match self.database.get_user(
            guild.id.0 as i64,
            member.user.id.0 as i64
        ).await {
            Ok(user) => {
                return Ok(user.permissions.contains(&permission));
            },
            Err(err) => {
                return Err(CommandError {
                    message: format!("An error occurred while fetching the user from the database. The error was: {}", err),
                    command_error: None
                });
            }
        }
    }
}