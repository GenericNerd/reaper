use serenity::{prelude::{Context}, model::{prelude::{interaction::{Interaction, application_command::ApplicationCommandInteraction}, Guild, Member}, permissions}};
use tracing::error;
use crate::{Handler, commands::{self, utils::send_message}, mongo::structs::Permissions};

use super::structs::CommandError;

impl Handler {
    pub async fn on_command(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let cmd_result: Result<(), CommandError> = match command.data.name.as_str() {
                "permissions" => {
                    commands::permissions::router::route(self, &ctx, &command).await
                },
                "strike" => {
                    commands::moderation::strike::run(self, &ctx, &command).await
                },
                "mute" => {
                    commands::moderation::mute::run(self, &ctx, &command).await
                },
                "kick" => {
                    commands::moderation::kick::run(self, &ctx, &command).await
                },
                "ban" => {
                    commands::moderation::ban::run(self, &ctx, &command).await
                }
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
                if user.permissions.contains(&permission) {
                    return Ok(true);
                }
            },
            Err(err) => {
                return Err(CommandError {
                    message: format!("An error occurred while fetching the user from the database. The error was: {}", err),
                    command_error: None
                });
            }
        }
        for role in member.roles.iter() {
            match self.database.get_role(
                guild.id.0 as i64,
                role.0 as i64
            ).await {
                Ok(role) => {
                    if role.permissions.contains(&permission) {
                        return Ok(true);
                    }
                },
                Err(err) => {
                    return Err(CommandError {
                        message: format!("An error occurred while fetching the role from the database. The error was: {}", err),
                        command_error: None
                    });
                }
            }
        }

        Ok(false)
    }

    pub async fn requires_permission(&self, ctx: &Context, cmd: &ApplicationCommandInteraction, permission: Permissions) -> Result<bool, CommandError> {
        match ctx.cache.guild(cmd.guild_id.expect("Could not obtain a guild ID. Was this command executed in a guild?")) {
            Some(guild) => {
                match self.has_permission(&guild, cmd.member.as_ref().unwrap(), permission.clone()).await {
                    Ok(has_permission) => {
                        if !has_permission {
                            match self.database.get_guild(guild.id.0 as i64).await {
                                Ok(guild) => {
                                    if guild.config.notify_missing_permissions {
                                        if let Err(err) = send_message(&ctx, &cmd, format!("You are missing the `{}` permission", permission.to_string())).await {
                                            return Err(CommandError {
                                                message: format!("An error occurred while sending a message to the user. The error was: {}", err),
                                                command_error: None
                                            });
                                        }
                                        return Ok(false);
                                    }
                                }
                                Err(err) => {
                                    return Err(CommandError {
                                        message: format!("An error occurred while fetching the guild from the database. The error was: {}", err),
                                        command_error: None
                                    });
                                }
                            }
                        }
                        return Ok(has_permission);
                    },
                    Err(err) => {
                        error!("An error occurred while checking permissions. The error was: {}", err);
                        let msg = send_message(&ctx, cmd, format!("An error occurred while checking your permissions. The error was: {}", err)).await;
                        if msg.is_ok() {
                            return Ok(false);
                        }
                        else {
                            return Err(msg.err().unwrap());
                        }
                    }
                }
            },
            None => {
                error!("The guild requested is not in cache, meaning permissions cannot be evaluated");
            }
        }
        Ok(false)
    }
}