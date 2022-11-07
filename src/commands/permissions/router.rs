use serenity::{model::prelude::{command::CommandOptionType, interaction::application_command::ApplicationCommandInteraction}, builder::CreateApplicationCommand, prelude::Context};
use crate::{Handler, commands::{permissions, structs::CommandError}, mongo::structs::Permissions};

pub async fn route(handler: &Handler, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    for option in cmd.data.options.iter() {
        match option.kind {
            CommandOptionType::SubCommand => {
                match option.name.as_str() {
                    "add" => {
                        match handler.requires_permission(&ctx, &cmd, Permissions::PermissionsAdd).await {
                            Ok(has_permission) => {
                                if has_permission {
                                    return permissions::add::user_run(&handler, &ctx, &cmd).await;
                                }
                            },
                            Err(err) => {
                                return Err(err);
                            }
                        }
                    },
                    "view" => {
                        match handler.requires_permission(&ctx, &cmd, Permissions::PermissionsView).await {
                            Ok(has_permission) => {
                                if has_permission {
                                    return permissions::view::user_run(&handler, &ctx, &cmd).await;
                                }
                            },
                            Err(err) => {
                                return Err(err);
                            }
                        }
                    }
                    "list" => {
                        match handler.requires_permission(&ctx, &cmd, Permissions::PermissionsList).await {
                            Ok(has_permission) => {
                                if has_permission {
                                    return permissions::list::run(&ctx, &cmd).await;
                                }
                            },
                            Err(err) => {
                                return Err(err);
                            }
                        }
                    }
                    _ => {}
                }
            },
            CommandOptionType::SubCommandGroup => {
                for options in option.options.iter() {
                    match options.name.as_str() {
                        "add" => {
                            match handler.requires_permission(&ctx, &cmd, Permissions::PermissionsAdd).await {
                                Ok(has_permission) => {
                                    if has_permission {
                                        return permissions::add::role_run(&handler, &ctx, &cmd).await;
                                    }
                                },
                                Err(err) => {
                                    return Err(err);
                                }
                            }
                        },
                        _ => {}
                    }
                }
            },
            _ => {}
        }
    }
    Ok(())
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