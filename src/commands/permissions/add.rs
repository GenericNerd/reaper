use serde_json::Value;
use serenity::{model::{prelude::{interaction::{application_command::{ApplicationCommandInteraction}}, command::CommandOptionType}}, prelude::Context};
use tracing::{warn, error};

use crate::{Handler, mongo::structs::{User, Permissions}, commands::{structs::CommandError, utils::send_message}};

pub async fn user_run(handler: &Handler, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    let mut user_id: i64;
    let mut user: Option<User> = None;
    let mut permission: Option<String> = None;

    for option in cmd.data.options.iter() {
        if option.kind == CommandOptionType::SubCommand {
            for option in option.options.iter() {
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
                                })
                            }
                        }
                        match handler.database.get_user(
                            cmd.guild_id.unwrap().0 as i64,
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
        }
        
    }

    if user.is_none() {
        return send_message(&ctx, cmd, "You must specify a user to add the permission to".to_string()).await;
    }
    if permission.is_none() {
        return send_message(&ctx, cmd, "You must supply a permission to add to the user".to_string()).await;
    }

    let mut permissions = user.clone().unwrap().permissions.clone();
    
    // I'm not really a fan of this
    // I was having issues with code stalling if trying to do .contains() in a vector of permissions

    let mut permission_strings: Vec<String> = Vec::new();
    for perm in permissions.iter() {
        permission_strings.push(perm.to_string());
    }
    if permission_strings.contains(&permission.clone().unwrap()) {
        return send_message(&ctx, cmd, format!("<@{}> already has the `{}` permission", user.unwrap().id, permission.clone().unwrap())).await;
    }
    permissions.push(Permissions::from(permission.clone().unwrap()));

    match handler.database.update_user_permissions(
        cmd.guild_id.unwrap().0 as i64,
        user.clone().unwrap().id,
        permissions
    ).await {
        Ok(_) => {
            return send_message(&ctx, cmd, format!("Successfully added the permission `{}` to <@{}>", permission.unwrap(), user.unwrap().id)).await;
        },
        Err(_) => {
            return send_message(&ctx, cmd, "An error occured while updating permissions. Please contact a developer".to_string()).await;
        }
    }
}