use serenity::{model::prelude::{command::CommandOptionType, interaction::application_command::ApplicationCommandInteraction}, builder::CreateApplicationCommand, prelude::Context};

use crate::{Handler, commands::{permissions, structs::CommandError}};

pub async fn route(handler: &Handler, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    for options in cmd.data.options.iter() {
        match options.kind {
            CommandOptionType::SubCommand => {
                match options.name.as_str() {
                    "add" => {
                        return permissions::add::user_run(&handler, &ctx, &cmd).await;
                    },
                    _ => {}
                }
            },
            CommandOptionType::SubCommandGroup => {
                for options in cmd.data.options.iter() {
                    match options.name.as_str() {
                        "add" => {

                        },
                        _ => {}
                    }
                }
            },
            _ => {}
        }
    }
    return permissions::add::user_run(&handler, &ctx, &cmd).await;
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("permissions")
        .dm_permission(false)
        .description("Modify and list permissions to users and roles")
        .create_option(|option| {
            option
                .name("add")
                .description("Add a Workless permission to a specific user")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("user")
                        .description("The user to add the permission to")
                        .kind(CommandOptionType::User)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("permission")
                        .description("The permission to add to the user")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("remove")
                .description("Remove a Workless permission from a specific user")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("user")
                        .description("The user to remove the permission from")
                        .kind(CommandOptionType::User)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("permission")
                        .description("The permission to remove from the user")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("list")
                .description("List the permissions to a user")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("user")
                        .description("The user to list the permissions of")
                        .kind(CommandOptionType::User)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("role")
                .description("Modify and list permissions to roles")
                .kind(CommandOptionType::SubCommandGroup)
                .create_sub_option(|option| {
                    option
                        .name("add")
                        .description("Add a Workless permission to a specific role")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|option| {
                            option
                                .name("role")
                                .description("The role to add the permission to")
                                .kind(CommandOptionType::Role)
                                .required(true)
                        })
                        .create_sub_option(|option| {
                            option
                                .name("permission")
                                .description("The permission to add to the role")
                                .kind(CommandOptionType::String)
                                .required(true)
                        })
                })
                .create_sub_option(|option| {
                    option
                        .name("remove")
                        .description("Remove a Workless permission from a specific role")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|option| {
                            option
                                .name("role")
                                .description("The role to remove the permission from")
                                .kind(CommandOptionType::Role)
                                .required(true)
                        })
                        .create_sub_option(|option| {
                            option
                                .name("permission")
                                .description("The permission to remove from the role")
                                .kind(CommandOptionType::String)
                                .required(true)
                        })
                })
                .create_sub_option(|option| {
                    option
                        .name("list")
                        .description("List the permissions to a role")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|option| {
                            option
                                .name("role")
                                .description("The role to list the permissions of")
                                .kind(CommandOptionType::Role)
                                .required(true)
                        })
                })
        })
}