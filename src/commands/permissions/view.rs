use std::collections::HashMap;

use serde_json::Value;
use serenity::{prelude::Context, model::prelude::{interaction::application_command::ApplicationCommandInteraction, command::CommandOptionType, RoleId, UserId, GuildId}};
use tracing::{error, warn};
use crate::{Handler, commands::{structs::CommandError, utils::send_message}, mongo::structs::{User, Permissions, Role}};

pub async fn user_run(handler: &Handler, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    let guild_id: i64 = cmd.guild_id.expect("Could not obtain a guild ID. Was this command executed in a guild?").0 as i64;
    let mut user_id: i64 = 0;
    let mut user: Option<User> = None;
    let mut user_roles: Vec<RoleId> = vec![];

    for option in cmd.data.options[0].options.iter() {
        match option.kind {
            CommandOptionType::User => {
                match Value::to_string(&option.value.clone().unwrap()).replace("\"", "").parse::<i64>() {
                    Ok(id) => {
                        user_id = id;
                        match ctx.cache.member(
                            GuildId{0: guild_id as u64},
                            UserId{0: user_id as u64}
                        ) {
                            Some(member) => {
                                user_roles = member.roles.clone();
                            },
                            None => {
                                warn!("Could not obtain a member from the cache. Attempting to obtain through a HTTP request.");
                                match ctx.http.get_member(guild_id as u64, user_id as u64).await {
                                    Ok(member) => {
                                        user_roles = member.roles.clone();
                                    },
                                    Err(err) => {
                                        error!("Could not obtain a member through a HTTP request: {}", err);
                                        return Err(CommandError {
                                            message: "Could not obtain the member given.".to_string(),
                                            command_error: None
                                        });
                                    }
                                }
                            }
                        }
                    },
                    Err(err) => {
                        error!("Failed to parse user ID. This is because: {}", err);
                        return Err(CommandError {
                            message: "Failed to parse user ID".to_string(),
                            command_error: None
                        });
                    }
                }
                match handler.database.get_user(
                    guild_id.clone(),
                    user_id.clone()
                ).await {
                    Ok(usr) => user = Some(usr),
                    Err(err) => {
                        return Err(CommandError {
                            message: format!("An error occurred while fetching the user from the database. The error was: {}", err),
                            command_error: None
                        });
                    }
                }
            },
            _ => {warn!("Option type not handled: {:?}", option.kind)}
        }
    }

    if user.is_none() {
        user_id = cmd.member.as_ref().expect("Could not obtain invoking member. Was this command executed in a guild?").user.id.0 as i64;
        user_roles = cmd.member.as_ref().expect("Could not obtain invoking member. Was this command executed in a guild?").roles.clone();
        match handler.database.get_user(
            guild_id.clone(),
            user_id
        ).await {
            Ok(usr) => user = Some(usr),
            Err(err) => {
                return Err(CommandError {
                    message: format!("An error occurred while fetching the user from the database. The error was: {}", err),
                    command_error: None
                });
            }
        }
    }

    let mut role_permissions: HashMap<String, Permissions> = HashMap::new();
    user_roles.push(RoleId{0: guild_id as u64});
    for role in user_roles.iter() {
        match handler.database.get_role(
            guild_id,
            role.0 as i64
        ).await {
            Ok(role) => {
                for permission in role.permissions.iter() {
                    if !user.as_ref().unwrap().permissions.contains(&permission) {
                        role_permissions.insert(role.id.to_string(), permission.clone());
                    }
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

    let mut message_content = format!("<@{}>", user_id).to_string();
    if user.as_ref().unwrap().permissions.is_empty() && role_permissions.is_empty() {
        message_content.push_str(" has no permissions");
    }

    if !user.as_ref().unwrap().permissions.is_empty() && role_permissions.is_empty() {
        message_content.push_str(" has the following permissions:\n");
        for permission in user.as_ref().unwrap().permissions.iter() {
            message_content.push_str(&format!("`{}`\n", permission.to_string()));
        }
    }

    if user.as_ref().unwrap().permissions.is_empty() && !role_permissions.is_empty() {
        message_content.push_str(" has no permissions, but they have inherited these permissions from their roles:\n");
        for (id, permission) in role_permissions.iter() {
            message_content.push_str(&format!("`{}` from <@&{}>\n", permission.to_string(), id));
        }
    }

    if !user.as_ref().unwrap().permissions.is_empty() && !role_permissions.is_empty() {
        message_content.push_str(" has the following permissions:\n");
        for permission in user.as_ref().unwrap().permissions.iter() {
            message_content.push_str(&format!("`{}`\n", permission.to_string()));
        }
        message_content.push_str("\nThey have also inherited these permissions from their roles:\n");
        for (id, permission) in role_permissions.iter() {
            message_content.push_str(&format!("`{}` from <@&{}>\n", permission.to_string(), id));
        }
    }

    return send_message(&ctx, cmd, message_content).await;
}

pub async fn role_run(handler: &Handler, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    let mut role_id: i64 = 0;
    let mut role: Option<Role> = None;

    for option in cmd.data.options[0].options[0].options.iter() {
        match option.kind {
            CommandOptionType::Role => {
                match Value::to_string(&option.value.clone().unwrap()).replace("\"", "").parse::<i64>() {
                    Ok(id) => {
                        role_id = id
                    },
                    Err(err) => {
                        error!("Failed to parse role ID. This is because: {}", err);
                        return Err(CommandError {
                            message: "Failed to parse role ID".to_string(),
                            command_error: None
                        });
                    }
                }
                match handler.database.get_role(
                    cmd.guild_id.expect("Could not obtain a guild ID. Was this command executed in a guild?").0 as i64,
                    role_id.clone()
                ).await {
                    Ok(rl) => role = Some(rl),
                    Err(err) => {
                        return Err(CommandError {
                            message: format!("An error occurred while fetching the role from the database. The error was: {}", err),
                            command_error: None
                        });
                    }
                }
            },
            _ => {warn!("Option type not handled: {:?}", option.kind)}
        }
    }

    if role.is_none() {
        return send_message(&ctx, cmd, "You must specify a role to view the permission for".to_string()).await;
    }

    let mut message_content = format!("<@&{}>", role_id).to_string();
    if role.as_ref().unwrap().permissions.is_empty() {
        message_content.push_str(" has no permissions");
    } else {
        message_content.push_str(" has the following permissions:\n");
        for permission in role.as_ref().unwrap().permissions.iter() {
            message_content.push_str(&format!("`{}`\n", permission.to_string()));
        }
    }

    return send_message(&ctx, cmd, message_content).await;
}