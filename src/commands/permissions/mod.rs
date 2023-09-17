use crate::models::command::Context;
use serenity::{
    all::{CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption, CreateEmbed},
};

use crate::models::{
    command::{Command, CommandContext, CommandResult},
    handler::Handler,
    permissions::Permission,
    response::Response,
};

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
    ) -> CommandResult {
        if !ctx.user_permissions.contains(&Permission::PermissionsView) {
            return ctx.reply(
                cmd,
                Response::new()
                    .embed(CreateEmbed::new()
                        .title("You do not have permission to do this!")
                        .description(format!(
                            "You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.",
                            Permission::PermissionsView.to_string()
                        ))
                        .color(0xf00)
                    )
                )
                .await;
        }

        for option in cmd.data.options.iter() {
            match option.name.as_str() {
                "user" => return user::user(handler, ctx, cmd).await,
                _ => continue,
            }
        }

        return ctx
            .reply(
                cmd,
                Response::new().embed(
                    CreateEmbed::new()
                        .title("Invalid command!")
                        .description("You must specify a subcommand to use this command!")
                        .color(0xf00),
                ),
            )
            .await;
    }
}
