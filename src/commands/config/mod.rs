use serenity::{
    all::{CommandInteraction, CommandOptionType, CreateCommandOption},
    builder::{CreateCommand, CreateEmbed},
};

use crate::{
    common::options::Options,
    models::{
        command::{Command, CommandContext, CommandContextReply},
        handler::Handler,
        permissions::Permission,
        response::{Response, ResponseError, ResponseResult},
    },
};

const EMBED_COLOR: i32 = 0x5539cc;

mod logging;
mod moderation;
mod role_recovery;

pub struct ConfigError {
    pub error: ResponseError,
    pub stages_to_skip: Option<usize>,
}

impl From<ResponseError> for ConfigError {
    fn from(error: ResponseError) -> Self {
        Self {
            error,
            stages_to_skip: None,
        }
    }
}

impl From<sqlx::Error> for ConfigError {
    fn from(value: sqlx::Error) -> Self {
        Self {
            error: ResponseError::Execution("Database Error", Some(format!("`{value}`"))),
            stages_to_skip: None,
        }
    }
}

impl From<serenity::Error> for ConfigError {
    fn from(value: serenity::Error) -> Self {
        Self {
            error: ResponseError::Serenity(value),
            stages_to_skip: None,
        }
    }
}

#[async_trait::async_trait]
trait ConfigStage: Send + Sync {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError>;
}

pub struct ConfigCommand;

#[async_trait::async_trait]
impl Command for ConfigCommand {
    fn name(&self) -> &'static str {
        "config"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("config")
            .dm_permission(false)
            .description("Configure Reaper for this server")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "category",
                    "Pick a category to configure",
                )
                .add_string_choice("Moderation", "moderation")
                .add_string_choice("Logging", "logging")
                .add_string_choice("Role Recovery", "role_recovery")
                .required(false),
            )
    }

    async fn router(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> ResponseResult {
        if !ctx.user_permissions.contains(&Permission::ConfigEdit) {
            return Err(ResponseError::Execution(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::ConfigEdit)),
            ));
        }

        let options = Options {
            options: cmd.data.options(),
        };

        let category = options.get_string("category").into_owned();

        let stages: Vec<Box<dyn ConfigStage>> = vec![
            Box::new(moderation::ModerationEnter),
            Box::new(moderation::ModerationMuteRole),
            Box::new(moderation::ModerationDefaultStrikeDuration),
            Box::new(moderation::ModerationEscalations),
            Box::new(moderation::ModerationFooter),
            Box::new(logging::LoggingEnter),
            Box::new(logging::LoggingLogActions),
            Box::new(logging::LoggingLogMessages),
            Box::new(logging::LoggingLogVoice),
            Box::new(logging::LoggingChannelEnter),
            Box::new(logging::LoggingChannelSingle),
            Box::new(logging::LoggingChannelMultipleActions),
            Box::new(logging::LoggingChannelMultipleMessages),
            Box::new(logging::LoggingChannelMultipleVoice),
            Box::new(role_recovery::RoleRecovery),
        ];

        let mut current_stage = match &category {
            Some(category) => match category.as_str() {
                "moderation" => 1,
                "logging" => 6,
                "role_recovery" => 14,
                _ => 0,
            },
            None => 0,
        };

        let ending_stage = match &category {
            Some(category) => match category.as_str() {
                "moderation" => 5,
                "logging" => 14,
                _ => stages.len(),
            },
            None => stages.len(),
        };

        while current_stage < ending_stage {
            match stages[current_stage].execute(handler, ctx, cmd).await {
                Ok(Some(stages_to_skip)) => current_stage += stages_to_skip,
                Ok(None) => current_stage += 1,
                Err(error) => {
                    if let ResponseError::Execution(title, description) = error.error {
                        ctx.reply(
                            cmd,
                            Response::new().components(vec![]).embed(
                                CreateEmbed::new()
                                    .title(format!("Error! - {title}"))
                                    .description(description.unwrap_or(String::new()))
                                    .color(0xff0000),
                            ),
                        )
                        .await?;
                        tokio::time::sleep(std::time::Duration::new(5, 0)).await;
                        current_stage += error.stages_to_skip.unwrap_or(0);
                    } else {
                        return Err(error.error);
                    }
                }
            }
        }

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title("Configuration Complete!")
                            .color(0x00ff00),
                    )
                    .components(vec![]),
            )
            .await?;
        tokio::time::sleep(std::time::Duration::new(5, 0)).await;
        if let Err(err) = message.delete(&ctx.ctx.http).await {
            return Err(ResponseError::Serenity(err));
        }
        Ok(())
    }
}
