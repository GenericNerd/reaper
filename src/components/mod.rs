use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use serenity::all::ComponentInteraction;

use crate::models::{
    context::Context, interactions::Interaction, permissions::Permission, response::ResponseResult,
};

pub mod config;

#[async_trait]
pub trait Component: Send + Sync {
    fn name(&self) -> &'static str;
    fn required_permission(&self) -> Option<Permission>;
    async fn router(
        &self,
        ctx: &Context<'_>,
        component: &ComponentInteraction,
        interaction: &Interaction,
    ) -> ResponseResult;
}

fn get_components_vec() -> Vec<Arc<dyn Component>> {
    vec![config::Config::new()]
}

pub fn get_components_available() -> HashMap<&'static str, Arc<dyn Component>> {
    let mut components = HashMap::new();

    for component in get_components_vec() {
        components.insert(component.name(), component);
    }

    components
}
