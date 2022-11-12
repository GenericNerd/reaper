use serenity::{builder::CreateApplicationCommand, model::prelude::{command::CommandOptionType, interaction::application_command::ApplicationCommandInteraction}, prelude::Context};

use crate::{Handler, commands::structs::CommandError, mongo::structs::Permissions};

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
    
    Ok(())
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