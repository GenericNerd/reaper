use serenity::all::{
    ButtonStyle, ComponentInteraction, CreateActionRow, CreateButton, CreateEmbed,
};

use crate::{
    components::config::{ConfigStage, EMBED_COLOR},
    models::{
        bot::Bot,
        context::{Context, ContextReply},
        interactions::{
            InteractionBuilder, InteractionKind,
            config::{ConfigInteraction, LoggingStage},
        },
        response::{Response, ResponseResult},
        user::User,
    },
};

const LOGGING_TITLE: &str = "Configuration - Logging";

#[derive(Debug)]
pub struct LoggingEnter;
#[async_trait::async_trait]
impl ConfigStage for LoggingEnter {
    fn key(&self) -> (&'static str, &'static str) {
        ("logging", "enter")
    }

    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)> {
        None
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        component: &ComponentInteraction,
        _data: &ConfigInteraction,
    ) -> ResponseResult {
        let user = User::from(component.user.id);
        let interactions = [
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
            component,
            Response::new()
                .embed(
                    CreateEmbed::new()
                        .title(LOGGING_TITLE)
                        .description("Would you like to configure logging?")
                        .color(EMBED_COLOR),
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
