use serde_json::Value;
use serenity::{builder::CreateApplicationCommand, model::prelude::{command::CommandOptionType, interaction::application_command::ApplicationCommandInteraction, UserId}, prelude::Context};
use tracing::{error, warn};
use crate::{Handler, commands::{structs::CommandError, utils::{send_message, Duration}}, mongo::structs::{Permissions, ActionType}};

impl Handler {
    pub async fn mute_user(&self, ctx: &Context, guild_id: i64, user_id: i64, reason: String) -> Result<(), CommandError> {
        match self.database.get_guild(guild_id.clone()).await {
            Ok(guild) => {
                match guild.config.moderation {
                    Some(moderation_config) => {
                        match ctx.http.add_member_role(guild_id as u64, user_id as u64, moderation_config.mute_role as u64, Some(reason.as_str())).await {
                            Ok(_) => {
                                return Ok(())
                            },
                            Err(err) => {
                                error!("Failed to mute user: {}", err);
                                return Err(CommandError {
                                    message: "Could not mute user".to_string(),
                                    command_error: None
                                });
                            }
                        }
                    },
                    None => {
                        warn!("No moderation settings exist for guild {}, but a mute was requested", guild_id);
                        return Err(CommandError {
                            message: "Moderation is not configured for this guild".to_string(),
                            command_error: None
                        });
                    }
                }
            },
            Err(err) => {
                error!("Could not obtain guild from database: {}", err);
                return Err(
                    CommandError {
                        message: "Could not obtain guild from database.".to_string(),
                        command_error: None
                    }
                );
            }
        }
    }
}

pub async fn run(handler: &Handler, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    match handler.requires_permission(&ctx, &cmd, Permissions::ModerationMute).await {
        Ok(has_permission) => {
            if !has_permission {
                return Ok(())
            }
        }
        Err(err) => {
            return Err(err)
        }
    }
    let moderator = cmd.member.as_ref().expect("Could not obtain a member. Was this command executed in a guild?");

    #[allow(unused_assignments)]
    let mut user_id: i64 = 0;
    #[warn(unused_assignments)]
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
        return send_message(&ctx, cmd, "You must specify a user to mute".to_string()).await;
    }

    if user_id == moderator.user.id.0 as i64 {
        return send_message(&ctx, cmd, "You cannot mute yourself".to_string()).await;
    }

    let reason = match cmd.data.options[1].value.as_ref() {
        Some(reason) => reason.as_str().unwrap().to_string(),
        None => {
            return send_message(&ctx, cmd, "You must specify a reason".to_string()).await;
        }
    };

    let duration = match cmd.data.options[2].value.as_ref() {
        Some(duration) => {
            Duration::new(duration.as_str().unwrap().to_string())
        },
        None => {
            return send_message(&ctx, cmd, "You must specify a duration".to_string()).await;
        }
    };

    if let Err(err) = handler.mute_user(
        &ctx,
        cmd.guild_id.expect("Could not obtain guild ID. Was this command executed in a guild?").0 as i64,
        user_id,
        format!("Muted by {} ({}) for {}", moderator.user.name, moderator.user.id.0, reason)
    ).await {
        return Err(err);
    }

    match handler.database.action_user(
        ActionType::Mute,
        user_id,
        moderator.guild_id.0 as i64,
        moderator.user.id.0 as i64,
        reason.clone(),
        Some(duration.to_unix_timestamp() as i64)
    ).await {
        Ok(action) => {
            let mut messaged_user = false;
            let user: Option<serenity::model::user::User>;
            match ctx.cache.user(UserId(user_id as u64)) {
                Some(usr) => {
                    user = Some(usr);
                },
                None => {
                    warn!("Could not obtain user from cache. Attempting to obtain through a HTTP request");
                    match ctx.http.get_user(user_id as u64).await {
                        Ok(usr) => {
                            user = Some(usr);
                        },
                        Err(err) => {
                            warn!("Could not obtain user from HTTP request: {}", err);
                            user = None;
                        }
                    }
                    
                }
            }
            if user.is_some() {
                let message_content = format!("You have been struck in {} by <@{}> for the following reason: `{}`", cmd.guild_id.expect("Could not obtain guild ID. Was this command executed in a guild?").to_partial_guild(&ctx).await.unwrap().name, action.moderator_id, reason);
                match user.unwrap().direct_message(&ctx.http, |message| {
                    message
                        .content(message_content)
                }).await {
                    Ok(_) => messaged_user = true,
                    Err(err) => {
                        warn!("Failed to message user. This is because: {}", err);
                    }
                }
            }
            
            let mut content = format!("Muted <@{}> for <t:{}:R> for `{}`", user_id, duration.to_unix_timestamp(), reason);
            if !messaged_user {
                content.push_str(format!("\n*<@{}> could not be notified*", user_id).as_str());
            }
            return send_message(&ctx, &cmd, content).await;
        },
        Err(err) => {
            error!("Failed to mute user. This is because: {}", err);
            return send_message(&ctx, cmd, "Failed to mute user. Report this to a developer".to_string()).await;
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("mute")
        .dm_permission(false)
        .description("Mute a user for an incorrect action for a given amount of time")
        .create_option(|option| {
            option
                .name("user")
                .description("The user to mute")
                .kind(CommandOptionType::User)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("reason")
                .description("The reason for this mute")
                .kind(CommandOptionType::String)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("duration")
                .description("The duration to mute the user for")
                .kind(CommandOptionType::String)
                .required(true)
        })
}