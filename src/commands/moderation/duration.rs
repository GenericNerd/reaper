use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("duration")
        .dm_permission(false)
        .description("Change the duration of a mute")
        .create_option(|option| {
            option
                .name("duration")
                .description("The new duration")
                .kind(CommandOptionType::String)
                .required(true)
        })
}