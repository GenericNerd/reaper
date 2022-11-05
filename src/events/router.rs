use serenity::{prelude::{EventHandler, Context}, model::prelude::interaction::Interaction};

#[serenity::async_trait]
impl EventHandler for crate::Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        self.on_command(ctx, interaction).await;
    }
}