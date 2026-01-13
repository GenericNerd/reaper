use serenity::all::{Context, EventHandler, Interaction, Ready};
use tracing::info;

use crate::events::EventRouter;

#[serenity::async_trait]
impl EventHandler for EventRouter {
    #[tracing::instrument(skip(self, _ctx, ready))]
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!(ready = ?ready, "Ready");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        self.on_interaction(ctx, interaction).await;
    }
}
