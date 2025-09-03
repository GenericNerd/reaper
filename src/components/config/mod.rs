use std::fmt::Debug;

use dashmap::DashMap;
use serenity::all::ComponentInteraction;
use tracing::debug;

use crate::{
    components::Component,
    models::{
        context::{Context, ContextReply},
        interactions::{
            Interaction, InteractionBuilder, InteractionKind, config::ConfigInteraction,
        },
        permissions::Permission,
        response::{ExecutionError, InternalError, ResponseError, ResponseResult},
        user::User,
    },
};

pub const EMBED_COLOR: u32 = 0x5539CC;

mod logging;
mod moderation;

#[async_trait::async_trait]
pub trait ConfigStage: Debug + Send + Sync {
    fn key(&self) -> (&'static str, &'static str);
    async fn router(
        &self,
        ctx: &Context<'_>,
        component: &ComponentInteraction,
        data: &ConfigInteraction,
    ) -> ResponseResult;
    fn on_error_go_to_stage(&self) -> Option<(&'static str, &'static str)>;
}

#[derive(Debug)]
pub struct Config {
    handlers: DashMap<(String, String), Box<dyn ConfigStage>>,
}

impl Config {
    pub fn new() -> Self {
        let handlers: Vec<Box<dyn ConfigStage>> = vec![
            Box::new(moderation::MuteRole),
            Box::new(moderation::SelectedMuteRole),
            Box::new(moderation::DefaultStrikeDuration),
            Box::new(moderation::ChangeDefaultStrikeDuration),
            Box::new(moderation::Escalations),
            Box::new(moderation::AddEscalation),
            Box::new(moderation::RemoveEscalation),
            Box::new(moderation::SubmitEscalations),
            Box::new(logging::LoggingEnter),
        ];
        Self {
            handlers: handlers
                .into_iter()
                .map(|h| ((h.key().0.to_string(), h.key().1.to_string()), h))
                .collect::<DashMap<_, _>>(),
        }
    }
}

pub async fn advance_to<S: ConfigStage>(
    stage: S,
    ctx: &Context<'_>,
    component: &ComponentInteraction,
    data: &ConfigInteraction,
) -> ResponseResult {
    stage.router(ctx, component, data).await
}

pub fn interaction_builder(user: User, interaction: ConfigInteraction) -> InteractionBuilder {
    InteractionBuilder::new(
        InteractionKind::Config {
            category: interaction,
        },
        user,
        Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
    )
}

#[async_trait::async_trait]
impl Component for Config {
    fn name(&self) -> &'static str {
        "config"
    }

    fn required_permission(&self) -> Option<Permission> {
        Some(Permission::ConfigEdit)
    }

    #[tracing::instrument(skip(ctx, component), fields(interaction_id = interaction.id.to_string()))]
    async fn router(
        &self,
        ctx: &Context<'_>,
        component: &ComponentInteraction,
        interaction: &Interaction,
    ) -> ResponseResult {
        // TODO: Remove once more interactions are added
        #[allow(irrefutable_let_patterns)]
        let InteractionKind::Config { category } = &interaction.kind else {
            return Err(ResponseError::Execution(ExecutionError::Internal(
                InternalError::InvalidInteractionType,
            )));
        };
        let stage = match &category {
            ConfigInteraction::Moderation { stage } => stage.to_string(),
            ConfigInteraction::Logging { stage } => stage.to_string(),
        };
        debug!(
            "Running config category {} stage {}",
            category.to_string(),
            stage
        );

        let mut current_key = (category.to_string(), stage);

        loop {
            let handler = self
                .handlers
                .get(&current_key)
                .ok_or(ResponseError::Execution(ExecutionError::Internal(
                    InternalError::InvalidConfigurationStep,
                )))?;

            match handler.router(ctx, component, category).await {
                Ok(res) => return Ok(res),
                Err(err) => {
                    if let ResponseError::Execution(_) = err {
                        ctx.error_message_ref(component, &err).await?;
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        if let Some((next_cat, next_stage)) = handler.on_error_go_to_stage() {
                            current_key = (next_cat.to_string(), next_stage.to_string());
                            continue;
                        } else {
                            return Err(err);
                        }
                    } else {
                        return Err(err);
                    }
                }
            }
        }
    }
}
