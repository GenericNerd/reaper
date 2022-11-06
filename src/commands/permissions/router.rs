use serenity::{model::prelude::{command::CommandOptionType, interaction::application_command::ApplicationCommandInteraction}, builder::CreateApplicationCommand, prelude::Context};
use tracing::error;
use crate::{Handler, commands::{permissions, structs::CommandError, utils::send_message}, mongo::structs::Permissions};

pub async fn route(handler: &Handler, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    for option in cmd.data.options.iter() {
        match option.kind {
            CommandOptionType::SubCommand => {
                match option.name.as_str() {
                    "add" => {
                        match ctx.cache.guild(cmd.guild_id.expect("Could not obtain a guild ID. Was this command executed in a guild?")) {
                            Some(guild) => {
                                match handler.has_permission(&guild, cmd.member.as_ref().unwrap(), Permissions::PermissionsAdd).await {
                                    Ok(permission) => {
                                        if permission {
                                            return permissions::add::user_run(&handler, &ctx, &cmd).await;
                                        }
                                        else {
                                            // Get from guild config whether to show an error
                                        }
                                    },
                                    Err(err) => {
                                        error!("An error occurred while checking permissions. The error was: {}", err);
                                        return send_message(&ctx, cmd, format!("An error occurred while checking your permissions. The error was: {}", err)).await;
                                    }
                                }
                            },
                            None => {
                                error!("The guild requested is not in cache, meaning permissions cannot be evaluated")
                            }
                        }
                    },
                    "view" => {
                        return permissions::view::user_run(&handler, &ctx, &cmd).await;
                    }
                    "list" => {
                        return permissions::list::run(&ctx, &cmd).await;
                    }
                    _ => {}
                }
            },
            CommandOptionType::SubCommandGroup => {
                for options in option.options.iter() {
                    match options.name.as_str() {
                        "add" => {
                            return permissions::add::role_run(&handler, &ctx, &cmd).await;
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
        .description("Modify and view permissions to users and roles")
        .create_option(|option| {
            option
                .name("add")
                .description("Add a Reaper permission to a specific user")
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
                .description("Remove a Reaper permission from a specific user")
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
                .name("view")
                .description("View the permissions to a user")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("user")
                        .description("The user to view the permissions of")
                        .kind(CommandOptionType::User)
                })
        })
        .create_option(|option| {
            option
                .name("list")
                .description("List all available permissions")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("role")
                .description("Modify and view permissions to roles")
                .kind(CommandOptionType::SubCommandGroup)
                .create_sub_option(|option| {
                    option
                        .name("add")
                        .description("Add a Reaper permission to a specific role")
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
                        .description("Remove a Reaper permission from a specific role")
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
                        .name("view")
                        .description("View the permissions to a role")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|option| {
                            option
                                .name("role")
                                .description("The role to view the permissions of")
                                .kind(CommandOptionType::Role)
                                .required(true)
                        })
                })
        })
}