use serenity::{
    all::{CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption},
};

use crate::models::{
    command::{Command, CommandContext},
    handler::Handler,
    permissions::Permission,
    response::{ResponseError, ResponseResult},
};

pub mod role;
pub mod user;

pub struct PermissionsCommand;

#[async_trait::async_trait]
impl Command for PermissionsCommand {
    fn name(&self) -> &'static str {
        "permissions"
    }

    fn register(&self) -> CreateCommand {
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

    async fn router(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> ResponseResult {
        if !ctx.user_permissions.contains(&Permission::PermissionsView) {
            return Err(ResponseError::ExecutionError(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::PermissionsView.to_string())),
            ));
        }

        for option in &cmd.data.options {
            match option.name.as_str() {
                "user" => return user::user(handler, ctx, cmd).await,
                "role" => return role::role(handler, ctx, cmd).await,
                _ => continue,
            }
        }

        Err(ResponseError::ExecutionError(
            "Invalid command",
            Some("You must specify a subcommand to use this command!".to_string()),
        ))
    }
}
