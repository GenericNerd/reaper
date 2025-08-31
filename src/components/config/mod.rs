use dashmap::DashMap;
use serenity::all::ComponentInteraction;

use crate::{
    components::Component,
    models::{
        context::Context,
        interactions::Interaction,
        permissions::Permission,
        response::{ResponseError, ResponseResult},
    },
};

pub const EMBED_COLOR: u32 = 0x5539CC;

mod moderation;

#[async_trait::async_trait]
pub trait ConfigStage: Send + Sync {
    fn key(&self) -> (&'static str, &'static str);
    async fn router(&self, ctx: &Context<'_>, component: &ComponentInteraction) -> ResponseResult;
}

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
) -> ResponseResult {
    stage.router(ctx, component).await
}

#[async_trait::async_trait]
impl Component for Config {
    fn name(&self) -> &'static str {
        "config"
    }

    fn required_permission(&self) -> Option<Permission> {
        Some(Permission::ConfigEdit)
    }

    async fn router(
        &self,
        ctx: &Context<'_>,
        component: &ComponentInteraction,
        interaction: &Interaction,
    ) -> ResponseResult {
        let Some(category) = interaction
            .data
            .get("category")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
        else {
            return Err(ResponseError::Execution(
                "Invalid category".to_string(),
                Some("Please notify the developers regarding this issue".to_string()),
            ));
        };
        let Some(step) = interaction
            .data
            .get("step")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
        else {
            return Err(ResponseError::Execution(
                "Invalid step".to_string(),
                Some("Please notify the developers regarding this issue".to_string()),
            ));
        };

        let handler = self
            .handlers
            .get(&(category, step))
            .ok_or(ResponseError::Execution(
                "Invalid step".to_string(),
                Some("Please notify the developers regarding this issue".to_string()),
            ))?;

        handler.router(ctx, component).await
    }
}
