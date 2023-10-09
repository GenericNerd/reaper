use serenity::{
    all::{
        ActionExecution, Guild, GuildMemberUpdateEvent, Interaction, InteractionType, Member,
        UnavailableGuild,
    },
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

    async fn guild_member_update(
        &self,
        _ctx: Context,
        _old_if_available: Option<Member>,
        new: Option<Member>,
        event: GuildMemberUpdateEvent,
    ) {
        let Some(member) = new else {
            return;
        };
        self.on_member_update(member, event).await;
    }

    async fn guild_member_addition(&self, ctx: Context, member: Member) {
        self.on_member_join(ctx, member).await;
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
