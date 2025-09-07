use serenity::all::{CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption};

use crate::{
    commands::Command,
    components::config::{Config, ConfigEntry},
    models::{
        context::Context,
        interactions::{
            InteractionBuilder, InteractionKind,
            config::{ConfigInteraction, LoggingStage, ModerationStage, XPStage},
        },
        options::Options,
        permissions::Permission,
        response::ResponseResult,
    },
};

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
                .add_string_choice("Levelling", "xp")
                .add_string_choice("Role Recovery", "role_recovery")
                .required(false),
            )
    }

    fn required_permission(&self) -> Option<Permission> {
        Some(Permission::ConfigEdit)
    }

    async fn router(&self, ctx: &Context<'_>, cmd: &CommandInteraction) -> ResponseResult {
        let config = Config::new();
        let category = Options::from_command(cmd)
            .get_string("category")
            .map(|cat| match cat.as_str() {
                "moderation" => ConfigInteraction::Moderation {
                    stage: ModerationStage::Footer,
                },
                "logging" => ConfigInteraction::Logging {
                    stage: LoggingStage::Categories {
                        actions: None,
                        messages: None,
                        voice: None,
                    },
                },
                "xp" => ConfigInteraction::XP {
                    stage: XPStage::RandomOrSet,
                },
                _ => ConfigInteraction::Moderation {
                    stage: ModerationStage::Enter,
                },
            });
        let single_category = category.is_some();

        config
            .internal_router(
                ctx,
                &ConfigEntry::Command(cmd.clone()),
                &InteractionBuilder::new(
                    InteractionKind::Config {
                        category: category.unwrap_or(ConfigInteraction::Moderation {
                            stage: ModerationStage::Enter,
                        }),
                        single_category,
                    },
                    ctx.get_populated_context()?.user,
                    Some(InteractionBuilder::one_hour_expiry()),
                )
                .build(),
            )
            .await
    }
}
