use serenity::all::{Context, EventHandler, Interaction, Ready};

use crate::events::EventRouter;

#[serenity::async_trait]
impl EventHandler for EventRouter {
    #[tracing::instrument(skip(self, ctx, ready))]
    async fn ready(&self, ctx: Context, ready: Ready) {
        self.on_ready(ctx, ready).await;
    }

    #[tracing::instrument(skip(self, ctx, interaction))]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        self.on_interaction(ctx, interaction).await;
    }
}
