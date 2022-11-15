use serde_json::Value;
use serenity::{builder::CreateApplicationCommand, model::prelude::{command::CommandOptionType, interaction::application_command::ApplicationCommandInteraction}, prelude::Context};
use tracing::error;
use crate::{commands::{structs::CommandError, utils::send_message}, Handler, mongo::structs::{Permissions, ActionType}};

pub async fn run(handler: &Handler, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    let guild_id = cmd.guild_id.expect("Could not obtain guild ID. Was this command executed in a guild?").0 as i64;
    let mut user_id: Option<i64> = None;
    let mut search_expired: bool = false;
    if cmd.data.options.get(0).is_none() {
        match handler.requires_permission(&ctx, &cmd, Permissions::ModerationSearchSelf).await {
            Ok(has_permission) => {
                if !has_permission {
                    return Ok(())
                }
            }
            Err(err) => {
                return Err(err)
            }
        }
        user_id = Some(cmd.member.as_ref().expect("Could not obtain a member. Was this command executed in a guild?").user.id.0 as i64);
    }
    else {
        match cmd.data.options[0].kind {
            CommandOptionType::User => {
                match Value::to_string(&cmd.data.options[0].value.clone().unwrap()).replace("\"", "").parse::<i64>() {
                    Ok(id) => {
                        user_id = Some(id);
                    }
                    Err(err) => {
                        error!("Could not parse user ID: {}", err);
                        return Err(CommandError {
                            message: "Could not parse user ID".to_string(),
                            command_error: None
                        });
                    }
                }
            },
            CommandOptionType::Boolean => {
                search_expired = cmd.data.options[0].value.as_ref().unwrap().as_bool().unwrap();
            },
            _ => {}
        }
        
        if cmd.data.options.get(1).is_some() {
            match cmd.data.options[1].kind {
                CommandOptionType::User => {
                    match Value::to_string(&cmd.data.options[1].value.clone().unwrap()).replace("\"", "").parse::<i64>() {
                        Ok(id) => {
                            user_id = Some(id);
                        }
                        Err(err) => {
                            error!("Could not parse user ID: {}", err);
                            return Err(CommandError {
                                message: "Could not parse user ID".to_string(),
                                command_error: None
                            });
                        }
                    }
                },
                CommandOptionType::Boolean => {
                    search_expired = cmd.data.options[1].value.as_ref().unwrap().as_bool().unwrap();
                },
                _ => {}
            }
        }
    }

    let permission: Permissions;
    if user_id.unwrap() == cmd.member.as_ref().unwrap().user.id.0 as i64 {
        if search_expired {
            permission = Permissions::ModerationSearchSelfExpired;
        }
        else {
            permission = Permissions::ModerationSearchSelf;
        }
    }
    else {
        if search_expired {
            permission = Permissions::ModerationSearchOthersExpired;
        }
        else {
            permission = Permissions::ModerationSearchOthers;
        }
    }
    match handler.requires_permission(&ctx, &cmd, permission).await {
        Ok(has_permission) => {
            if !has_permission {
                return Ok(())
            }
        }
        Err(err) => {
            return Err(err)
        }
    }

    match handler.database.get_user_actions(guild_id, user_id.unwrap()).await {
        Ok(actions) => {
            let mut message = format!("<@{}>'s history:\n", user_id.unwrap());
            let mut valid_actions = 0;
            for action in &actions {
                if search_expired || action.active {
                    valid_actions += 1;
                    let mut action_type = match action.action_type {
                        ActionType::Strike => "Strike",
                        ActionType::Mute => "Mute",
                        ActionType::Ban => "Ban",
                        ActionType::Kick => "Kick",
                        _ => {"Unknown"}
                    }.to_string();
                    if !action.active {
                        action_type.push_str(" (Expired)");
                    }

                    message.push_str(&format!("\n**{}**\n{}\n*Issued by: <@{}>*\n", action_type, action.reason, action.moderator_id));
                    if action.expiry.is_some() {
                        message.push_str(&format!("*Expires: <t:{}:R>*\n", action.expiry.unwrap()));
                    }
                    message.push_str(&format!("*UUID: `{}`*\n", action.uuid));
                }
            }
            if valid_actions == 0 && !search_expired {
                message.push_str("No any active history");
            }
            else if valid_actions == 0 && search_expired {
                message.push_str("No history");
            }

            return send_message(&ctx, &cmd, message).await;
        },
        Err(err) => {
            error!("Could not obtain user actions from database: {}", err);
            return Err(CommandError {
                message: "Could not obtain user actions from database".to_string(),
                command_error: None
            });
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("search")
        .dm_permission(false)
        .description("Search a user for their moderation history")
        .create_option(|option| {
            option
                .name("user")
                .description("The user to search")
                .kind(CommandOptionType::User)
                .required(false)
        })
        .create_option(|option| {
            option
                .name("expired")
                .description("View expired actions")
                .kind(CommandOptionType::Boolean)
                .required(false)
        })
}