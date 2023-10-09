use serenity::{
    all::{ActionExecution, Guild, Interaction, InteractionType, UnavailableGuild},
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
        if interaction.kind() == InteractionType::Command {
            self.on_command(ctx, interaction.command().unwrap()).await;
        }
    }

    async fn guild_create(&self, _ctx: Context, guild: Guild, is_new: Option<bool>) {
        if is_new.is_none() || !is_new.unwrap() {
            return;
        }
        self.on_guild_create(guild).await;
    }

    async fn guild_delete(
        &self,
        _ctx: Context,
        guild: UnavailableGuild,
        _full_guild: Option<Guild>,
    ) {
        if guild.unavailable {
            return;
        }
        self.on_guild_leave(guild).await;
    }

    async fn auto_moderation_action_execution(&self, ctx: Context, execution: ActionExecution) {
        self.on_automod_trigger(ctx, execution).await;
    }
}
