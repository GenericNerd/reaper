use serenity::all::{
    ButtonStyle, CommandInteraction, CommandOptionType, CreateActionRow, CreateButton,
    CreateCommand, CreateCommandOption, CreateEmbed,
};

use crate::{
    commands::Command,
    models::{
        bot::Bot,
        context::{Context, ContextReply},
        interactions::{
            InteractionBuilder, InteractionKind,
            config::{ConfigInteraction, LoggingStage, ModerationStage},
        },
        permissions::Permission,
        response::{Response, ResponseResult},
        user::User,
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
        let user = User::from(cmd.user.id);
        // TODO: Read command option, add field to data to determine whether to run whole config or just a specific category
        let interactions = [
            InteractionBuilder::new(
                InteractionKind::Config {
                    category: ConfigInteraction::Moderation {
                        stage: ModerationStage::MuteRole,
                    },
                },
                user,
                Some(InteractionBuilder::one_hour_expiry()),
            )
            .build(),
            InteractionBuilder::new(
                InteractionKind::Config {
                    category: ConfigInteraction::Logging {
                        stage: LoggingStage::Enter,
                    },
                },
                user,
                Some(InteractionBuilder::one_hour_expiry()),
            )
            .build(),
        ];

        Bot::global()
            .interaction_state()
            .register(interactions.to_vec())
            .await?;

        ctx.reply(
            cmd,
            Response::new()
                .embed(
                    CreateEmbed::new()
                        .title("Moderation")
                        .description("Would you like to configure moderation?")
                        .color(0x5539CC),
                )
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new(interactions[0].id.to_string())
                        .style(ButtonStyle::Success)
                        .label("Yes"),
                    CreateButton::new(interactions[1].id.to_string())
                        .style(ButtonStyle::Secondary)
                        .label("No"),
                ])]),
        )
        .await
        .map(|_| ())
    }
}
