use std::collections::HashMap;

use serenity::{all::ComponentInteraction, async_trait};

use crate::models::{context::Context, permissions::Permission, response::ResponseResult};

#[async_trait]
pub trait Component: Send + Sync {
    fn name(&self) -> &'static str;
    fn required_permission(&self) -> Option<Permission>;
    async fn router(&self, ctx: &Context<'_>, component: &ComponentInteraction) -> ResponseResult;
}

fn get_components_vec() -> Vec<Box<dyn Component>> {
    vec![]
}

pub fn get_components_available() -> HashMap<&'static str, Box<dyn Component>> {
    let mut components = HashMap::new();

    for component in get_components_vec() {
        components.insert(component.name(), component);
    }

    components
}
