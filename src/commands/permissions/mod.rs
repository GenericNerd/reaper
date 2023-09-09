use serenity::{
    all::CommandOptionType,
    builder::{CreateCommand, CreateCommandOption},
};

pub mod user;

pub fn register() -> CreateCommand {
    CreateCommand::new("permissions")
        .dm_permission(false)
        .description("View and modify permissions for users and roles")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "user",
                "View and modify permissions for users",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::User,
                    "user",
                    "The user to view or modify permissions for",
                )
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "role",
                "View and modify permissions for roles",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Role,
                    "role",
                    "The role to view or modify permissions for",
                )
                .required(true),
            ),
        )
}
