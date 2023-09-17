use serenity::{
    all::{Interaction, InteractionType},
    model::prelude::Ready,
    prelude::{Context, EventHandler},
};

use crate::models::handler::Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        self.on_ready(ctx, ready).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction.kind() {
            InteractionType::Command => self.on_command(ctx, interaction.command().unwrap()).await,
            _ => {}
        }
    }
}
