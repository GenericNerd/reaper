use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("remove")
        .dm_permission(false)
        .description("Remove an action")
        .create_option(|option| {
            option
                .name("uuid")
                .description("The UUID of the action to remove")
                .kind(CommandOptionType::String)
                .required(true)
        })
}