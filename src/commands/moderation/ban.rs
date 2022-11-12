use serenity::{builder::CreateApplicationCommand, model::prelude::{command::CommandOptionType, interaction::application_command::ApplicationCommandInteraction}, prelude::Context};

use crate::{Handler, commands::structs::CommandError, mongo::structs::Permissions};

pub async fn run(handler: &Handler, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    match handler.requires_permission(&ctx, &cmd, Permissions::ModerationBan).await {
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
        .name("ban")
        .dm_permission(false)
        .description("Ban a user from the guild, either permanently or temporarily")
        .create_option(|option| {
            option
                .name("user")
                .description("The user to ban")
                .kind(CommandOptionType::User)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("reason")
                .description("The reason to ban this user")
                .kind(CommandOptionType::String)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("duration")
                .description("The duration to strike the user for (permanent by default)")
                .kind(CommandOptionType::String)
                .required(false)
        })
}