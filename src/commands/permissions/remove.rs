use serde_json::Value;
use serenity::{model::{prelude::{interaction::{application_command::{ApplicationCommandInteraction}}, command::CommandOptionType}}, prelude::Context};
use tracing::{warn, error};

use crate::{Handler, mongo::structs::{User, Permissions, Role}, commands::{structs::CommandError, utils::send_message}};

pub async fn user_run(handler: &Handler, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    let mut user_id: i64;
    let mut user: Option<User> = None;
    let mut permission: Option<String> = None;

    for option in cmd.data.options[0].options.iter() {
        match option.kind {
            CommandOptionType::User => {
                match Value::to_string(&option.value.clone().unwrap()).replace("\"", "").parse::<i64>() {
                    Ok(id) => {
                        user_id = id
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
                    cmd.guild_id.expect("Could not obtain a guild ID. Was this command executed in a guild?").0 as i64,
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
            CommandOptionType::String => {
                match Permissions::from(option.value.as_ref().unwrap().as_str().unwrap().to_string()) {
                    Permissions::Unknown => {
                        return send_message(&ctx, cmd, format!("The permission `{}` is not valid. You can run `/permissions list` to see all valid permissions", option.value.as_ref().unwrap().as_str().unwrap())).await;
                    },
                    _ => {
                        permission = Some(option.value.as_ref().unwrap().as_str().unwrap().to_string());
                    }
                }
            },
            _ => {warn!("Option type not handled: {:?}", option.kind)}
        }
    }

    if user.is_none() {
        return send_message(&ctx, cmd, "You must specify a user to remove the permission from".to_string()).await;
    }
    if permission.is_none() {
        return send_message(&ctx, cmd, "You must supply a permission to remove from the user".to_string()).await;
    }
    let mut permissions = user.clone().unwrap().permissions.clone();
    if !permissions.contains(&Permissions::from(permission.clone().unwrap())) {
        return send_message(&ctx, cmd, format!("<@{}> does not has the `{}` permission", user.unwrap().id, permission.clone().unwrap())).await;
    }
    permissions.retain(|perm| perm != &Permissions::from(permission.clone().unwrap()));
    
    match handler.database.update_user_permissions(
        cmd.guild_id.expect("Could not obtain a guild ID. Was this command executed in a guild?").0 as i64,
        user.clone().unwrap().id,
        permissions
    ).await {
        Ok(_) => {
            return send_message(&ctx, cmd, format!("Successfully removed the permission `{}` from <@{}>", permission.unwrap(), user.unwrap().id)).await;
        },
        Err(_) => {
            return send_message(&ctx, cmd, "An error occured while updating permissions. Please contact a developer".to_string()).await;
        }
    }
}

pub async fn role_run(handler: &Handler, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    let mut role_id: i64;
    let mut role: Option<Role> = None;
    let mut permission: Option<String> = None;

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
            CommandOptionType::String => {
                match Permissions::from(option.value.as_ref().unwrap().as_str().unwrap().to_string()) {
                    Permissions::Unknown => {
                        return send_message(&ctx, cmd, format!("The permission `{}` is not valid. You can run `/permissions list` to see all valid permissions", option.value.as_ref().unwrap().as_str().unwrap())).await;
                    },
                    _ => {
                        permission = Some(option.value.as_ref().unwrap().as_str().unwrap().to_string());
                    }
                }
            },
            _ => {warn!("Option type not handled: {:?}", option.kind)}
        }
    }

    if role.is_none() {
        return send_message(&ctx, cmd, "You must specify a role to remove the permission from".to_string()).await;
    }
    if permission.is_none() {
        return send_message(&ctx, cmd, "You must supply a permission to remove from the role".to_string()).await;
    }

    let mut permissions = role.clone().unwrap().permissions.clone();
    if !permissions.contains(&Permissions::from(permission.clone().unwrap())) {
        return send_message(&ctx, cmd, format!("<@&{}> does not has the `{}` permission", role.unwrap().id, permission.clone().unwrap())).await;
    }
    permissions.retain(|perm| perm != &Permissions::from(permission.clone().unwrap()));

    match handler.database.update_role_permissions(
        cmd.guild_id.expect("Could not obtain a guild ID. Was this command executed in a guild?").0 as i64,
        role.clone().unwrap().id,
        permissions
    ).await {
        Ok(_) => {
            return send_message(&ctx, cmd, format!("Successfully removed the permission `{}` from the <@&{}>", permission.unwrap(), role.unwrap().id)).await;
        },
        Err(_) => {
            return send_message(&ctx, cmd, "An error occured while updating permissions. Please contact a developer".to_string()).await;
        }
    }
}