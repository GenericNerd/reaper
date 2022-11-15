use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("unban")
        .dm_permission(false)
        .description("Unmute a user")
        .create_option(|option| {
            option
                .name("user")
                .description("The user to unban")
                .kind(CommandOptionType::User)
                .required(true)
        })
}