use serenity::all::{CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption};

use crate::models::{
    command::{Command, CommandContext},
    handler::Handler,
    response::{ResponseError, ResponseResult},
};

pub mod feature;
pub mod guild;
pub mod user;

pub struct GlobalCommand;

#[async_trait::async_trait]
impl Command for GlobalCommand {
    fn name(&self) -> &'static str {
        "global"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("global")
            .dm_permission(false)
            .description("Globally kill a feature, guild, or user from using Reaper")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommandGroup,
                    "feature",
                    "Kill a feature",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "kill",
                        "Globally kill a feature",
                    )
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "feature",
                            "The feature to globally kill",
                        )
                        .required(true),
                    ),
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "revive",
                        "Revive a feature",
                    )
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "feature",
                            "The feature to revive",
                        )
                        .required(true),
                    ),
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "status",
                        "Check the status of a feature",
                    )
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "feature",
                            "The feature to check the status of",
                        )
                        .required(true),
                    ),
                ),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommandGroup,
                    "guild",
                    "Kill a guild",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "kill",
                        "Globally kill a guild",
                    )
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "guild",
                            "The guild to globally kill",
                        )
                        .required(true),
                    ),
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "revive",
                        "Revive a guild",
                    )
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "guild",
                            "The guild to revive",
                        )
                        .required(true),
                    ),
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "status",
                        "Check the status of a guild",
                    )
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "guild",
                            "The guild to check the status of",
                        )
                        .required(true),
                    ),
                ),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::SubCommandGroup, "user", "Kill a user")
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::SubCommand,
                            "kill",
                            "Globally kill a user",
                        )
                        .add_sub_option(
                            CreateCommandOption::new(
                                CommandOptionType::User,
                                "user",
                                "The user to globally kill",
                            )
                            .required(true),
                        ),
                    )
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::SubCommand,
                            "revive",
                            "Revive a user",
                        )
                        .add_sub_option(
                            CreateCommandOption::new(
                                CommandOptionType::User,
                                "user",
                                "The user to revive",
                            )
                            .required(true),
                        ),
                    )
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::SubCommand,
                            "status",
                            "Check the status of a user",
                        )
                        .add_sub_option(
                            CreateCommandOption::new(
                                CommandOptionType::User,
                                "user",
                                "The user to check the status of",
                            )
                            .required(true),
                        ),
                    ),
            )
    }

    async fn router(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> ResponseResult {
        let has_global_kill_role = match ctx
            .ctx
            .http
            .get_member(handler.global_kill_guild, cmd.user.id)
            .await
        {
            Ok(member) => member.roles.contains(&handler.global_kill_role),
            Err(_) => false,
        };

        if !has_global_kill_role {
            return Err(ResponseError::Execution(
                "You do not have permission to do this!",
                Some(format!(
                    "Glboal kills can only be performed by people with the <@&{}> role.",
                    handler.global_kill_role.get()
                )),
            ));
        }

        for option in &cmd.data.options {
            match option.name.as_str() {
                "feature" => return feature::router(handler, ctx, cmd, option).await,
                "guild" => return guild::router(handler, ctx, cmd).await,
                "user" => return user::router(handler, ctx, cmd).await,
                _ => continue,
            }
        }

        Err(ResponseError::Execution(
            "Invalid command",
            Some("You must specify a subcommand to use this command.".to_string()),
        ))
    }
}

pub fn get_kill_commands() -> Vec<Box<dyn Command>> {
    vec![Box::new(GlobalCommand)]
}
