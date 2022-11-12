use serde_json::Value;
use serenity::{builder::CreateApplicationCommand, model::prelude::{command::CommandOptionType, interaction::application_command::ApplicationCommandInteraction, UserId}, prelude::Context};
use tracing::{error, warn};
use crate::{Handler, commands::{structs::CommandError, utils::{send_message, Duration}}, mongo::structs::{Permissions, ActionType}};

pub async fn run(handler: &Handler, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    let guild = match handler.database.get_guild(cmd.guild_id.expect("Could not obtain guild ID. Was this command executed in a guild?").0 as i64).await {
        Ok(guild) => guild,
        Err(err) => {
            error!("Could not obtain guild from database: {}", err);
            return Err(
                CommandError {
                    message: "Could not obtain guild from database.".to_string(),
                    command_error: None
                }
            );
        }
    };

    match handler.requires_permission(&ctx, &cmd, Permissions::ModerationStrike).await {
        Ok(has_permission) => {
            if !has_permission {
                return Ok(())
            }
        }
        Err(err) => {
            return Err(err)
        }
    }
    let user_id: i64;
    match Value::to_string(&cmd.data.options[0].value.clone().unwrap()).replace("\"", "").parse::<i64>() {
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
    if user_id == 0 {
        return send_message(&ctx, cmd, "You must specify a user to strike".to_string()).await;
    }

    if user_id.clone() == cmd.member.as_ref().expect("Could not obtain a member. Was this command executed in a guild?").user.id.0 as i64 {
        return send_message(&ctx, &cmd, "You cannot strike yourself".to_string()).await;
    }

    let reason = match cmd.data.options[1].value.as_ref() {
        Some(reason) => reason.as_str().unwrap().to_string(),
        None => {
            return send_message(&ctx, &cmd, "You must supply a reason for the strike".to_string()).await;
        }
    };

    let duration = match cmd.data.options.get(2) {
        Some(duration) => {
            match duration.value.as_ref() {
                Some(value) => {
                    let dur = Duration::new(value.as_str().unwrap().to_string()); 
                    if dur.is_permanent() {
                        None
                    } else {
                        Some(dur.to_unix_timestamp() as i64)
                    }
                },
                None => {
                    None
                }
            }
        },
        None => None
    };

    match handler.database.get_user_actions(guild.clone().id, user_id.clone()).await {
        Ok(actions) => {
            let mut strikes: u64 = 0;
            for action in actions.iter() {
                if action.active && action.action_type == ActionType::Strike {
                    strikes += 1;
                }
            }
            match guild.config.strike_escalations.get(&strikes) {
                Some(escalation) => {
                    match escalation.action_type {
                        ActionType::Unknown => {
                            warn!("Unknown action type for escalation");
                        },
                        ActionType::Strike => {
                            warn!("Strike is not a valid action type for escalation");
                        }
                        ActionType::Mute => {

                        },
                        ActionType::Kick => {

                        },
                        ActionType::Ban => {

                        }
                    }
                },
                None => {}
            }
        },
        Err(err) => {
            error!("Could not obtain user actions from database: {}", err);
            return Err(CommandError {
                message: "Could not obtain user actions from database".to_string(),
                command_error: None
            });
        }
    }

    match handler.database.action_user(
        ActionType::Strike,
        user_id,
        guild.clone().id,
        cmd.member.as_ref().expect("Could not obtain a member. Was this command executed in a guild?").user.id.0 as i64,
        reason.clone(),
        duration
    ).await {
        Ok(action) => {
            let mut messaged_user = false;
            match ctx.cache.user(UserId(user_id as u64)) {
                Some(user) => {
                    let message_content = format!("You have been struck in {} by <@{}> for the following reason: `{}`", cmd.guild_id.expect("Could not obtain guild ID. Was this command executed in a guild?").to_partial_guild(&ctx).await.unwrap().name, action.moderator_id, reason);
                    match user.direct_message(&ctx.http, |message| {
                        message
                            .content(message_content)
                    }).await {
                        Ok(_) => messaged_user = true,
                        Err(err) => {
                            warn!("Failed to message user. This is because: {}", err);
                        }
                    }
                },
                None => {
                    return send_message(&ctx, &cmd, format!("The user with ID `{}` does not exist", user_id)).await;
                }
            }
            let mut content = format!("Striked <@{}>", user_id);
            if let Some(duration) = duration {
                content.push_str(format!(" for <t:{}:R>", duration).as_str());
            }
            content.push_str(format!(" for `{}`", reason).as_str());
            if !messaged_user {
                content.push_str(format!("\n*<@{}> could not be notified*", user_id).as_str());
            }
            return send_message(&ctx, &cmd, content).await;
        },
        Err(err) => {
            error!("Failed to strike user. This is because: {}", err);
            return send_message(&ctx, &cmd, "Failed to add a strike to this user. Report this to a developer".to_string()).await;
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("strike")
        .dm_permission(false)
        .description("Strike a user for an incorrect action")
        .create_option(|option| {
            option
                .name("user")
                .description("The user to give a strike to")
                .kind(CommandOptionType::User)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("reason")
                .description("The reason to give this strike to this user")
                .kind(CommandOptionType::String)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("duration")
                .description("The duration to strike the user for (default 30 days)")
                .kind(CommandOptionType::String)
                .required(false)
        })
}