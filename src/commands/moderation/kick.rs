use serenity::{builder::CreateApplicationCommand, model::prelude::{command::CommandOptionType, interaction::application_command::ApplicationCommandInteraction}, prelude::Context};

use crate::{Handler, commands::structs::CommandError, mongo::structs::Permissions};

pub async fn run(handler: &Handler, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    match handler.requires_permission(&ctx, &cmd, Permissions::ModerationKick).await {
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
        .name("kick")
        .dm_permission(false)
        .description("Kick a user from the guild")
        .create_option(|option| {
            option
                .name("user")
                .description("The user to kick")
                .kind(CommandOptionType::User)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("reason")
                .description("The reason for the kick")
                .kind(CommandOptionType::String)
                .required(true)
        })
}