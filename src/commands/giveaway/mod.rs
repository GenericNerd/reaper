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

pub mod delete;
pub mod end;
pub mod interaction;
pub mod new;
pub mod reroll;

pub struct GiveawayCommand;

#[async_trait::async_trait]
impl Command for GiveawayCommand {
    fn name(&self) -> &'static str {
        "giveaway"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("giveaway")
            .description("Giveaway commands")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "new",
                    "Start a new giveaway",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "prize",
                        "The prize for the giveaway",
                    )
                    .required(true),
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "duration",
                        "The duration of the giveaway",
                    )
                    .required(true),
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Integer,
                        "winners",
                        "The number of winners for the giveaway (default: 1)",
                    )
                    .required(false),
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Role,
                        "role",
                        "The role to require to enter the giveaway",
                    )
                    .required(false),
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "description",
                        "The description for the giveaway",
                    )
                    .required(false),
                ),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "reroll",
                    "Reroll a giveaway",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "id",
                        "The message ID of the giveaway to reroll",
                    )
                    .required(true),
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Integer,
                        "winners",
                        "The number of winners for the giveaway (default: 1)",
                    )
                    .required(false),
                ),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::SubCommand, "end", "End a giveaway")
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "id",
                            "The message ID of the giveaway to delete",
                        )
                        .required(true),
                    ),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "delete",
                    "Delete a giveaway",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "id",
                        "The message ID of the giveaway to delete",
                    )
                    .required(true),
                ),
            )
            .dm_permission(false)
    }

    async fn router(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> ResponseResult {
        for option in &cmd.data.options {
            let permission = match option.name.as_str() {
                "new" => Permission::GiveawayCreate,
                "reroll" => Permission::GiveawayReroll,
                "end" => Permission::GiveawayEnd,
                "delete" => Permission::GiveawayDelete,
                _ => continue,
            };

            if !ctx.user_permissions.contains(&permission) {
                return Err(ResponseError::Execution(
                    "You do not have permission to do this!",
                    Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", permission.to_string())),
                ));
            }

            match option.name.as_str() {
                "new" => return new::new(handler, ctx, cmd).await,
                "reroll" => return reroll::reroll(handler, ctx, cmd).await,
                "end" => return end::end(handler, ctx, cmd).await,
                "delete" => return delete::delete(handler, ctx, cmd).await,
                _ => continue,
            }
        }

        Err(ResponseError::Execution(
            "Invalid command",
            Some("You must specify a subcommand to use this command!".to_string()),
        ))
    }
}
