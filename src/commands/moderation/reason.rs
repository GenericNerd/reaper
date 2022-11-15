use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("reason")
        .dm_permission(false)
        .description("Change the reason of an action")
        .create_option(|option| {
            option
                .name("reason")
                .description("The new reason")
                .kind(CommandOptionType::String)
                .required(true)
        })
}