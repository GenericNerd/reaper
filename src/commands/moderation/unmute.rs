use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("unmute")
        .dm_permission(false)
        .description("Unmute a user")
        .create_option(|option| {
            option
                .name("user")
                .description("The user to unmute")
                .kind(CommandOptionType::User)
                .required(true)
        })
}