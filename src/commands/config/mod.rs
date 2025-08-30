use serenity::all::{
    ButtonStyle, CommandInteraction, CommandOptionType, CreateActionRow, CreateButton,
    CreateCommand, CreateCommandOption, CreateEmbed,
};

use crate::{
    commands::Command,
    models::{
        bot::Bot,
        context::{Context, ContextReply},
        interactions::InteractionBuilder,
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
        let interaction_builders = vec![
            InteractionBuilder::new(
                "Yes".to_string(),
                user,
                serde_json::json!({"interaction": "config", "category": "moderation", "step": "mute_role"}),
            ),
            InteractionBuilder::new(
                "No".to_string(),
                user,
                serde_json::json!({"interaction": "config", "category": "logging", "step": "enter"}),
            ),
        ];
        let interactions = Bot::global()
            .register_interactions(interaction_builders)
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
                .components(vec![CreateActionRow::Buttons(
                    interactions
                        .into_iter()
                        .map(|interaction| {
                            CreateButton::new(interaction.id.to_string())
                                .style(match interaction.action.as_str() {
                                    "Yes" => ButtonStyle::Success,
                                    "No" => ButtonStyle::Secondary,
                                    _ => panic!("Invalid interaction action"),
                                })
                                .label(interaction.action)
                        })
                        .collect(),
                )]),
        )
        .await
        .map(|_| ())
    }
}
