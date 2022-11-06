use serde_json::Value;
use serenity::{prelude::Context, model::prelude::{interaction::application_command::ApplicationCommandInteraction, command::CommandOptionType}};
use tracing::{error, warn};
use crate::{Handler, commands::{structs::CommandError, utils::send_message}, mongo::structs::User};

pub async fn user_run(handler: &Handler, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    let mut user_id: i64 = 0;
    let mut user: Option<User> = None;

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
                    _ => {warn!("Option type not handled: {:?}", option.kind)}
                }
            }
        }
    }

    if user.is_none() {
        user_id = cmd.member.as_ref().expect("Could not obtain invoking member. Was this command executed in a guild?").user.id.0 as i64;
        match handler.database.get_user(
            cmd.guild_id.expect("Could not obtain a guild ID. Was this command executed in a guild?").0 as i64,
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

    if user.as_ref().unwrap().permissions.is_empty() {
        return send_message(&ctx, cmd, format!("<@{}> has no permissions", user.unwrap().id)).await;
    }

    let mut message_content = format!("<@{}> has the following permissions:\n", user_id).to_string();
    for permission in user.unwrap().permissions.iter() {
        message_content.push_str(&format!("`{}`\n", permission.to_string()));
    }
    return send_message(&ctx, cmd, message_content).await;
}