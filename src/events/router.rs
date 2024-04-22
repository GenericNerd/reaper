use serenity::{
    all::{
        ActionExecution, ChannelId, Guild, GuildId, GuildMemberUpdateEvent, Interaction,
        InteractionType, Member, Message, MessageId, MessageUpdateEvent, Reaction,
        UnavailableGuild, VoiceState,
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

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        self.on_reaction_add(ctx, reaction).await;
    }

    async fn auto_moderation_action_execution(&self, ctx: Context, execution: ActionExecution) {
        Box::pin(self.on_automod_trigger(ctx, execution)).await;
    }

    async fn message(&self, _ctx: Context, new_message: Message) {
        if new_message.author.bot {
            return;
        }

        if new_message.guild_id.is_none() {
            return;
        }

        self.on_message(new_message).await;
    }

    async fn message_update(
        &self,
        ctx: Context,
        _old_if_available: Option<Message>,
        _new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        if event.content.is_none() {
            return;
        }
        if event.attachments.is_none() {
            return;
        }
        if event.guild_id.is_none() {
            return;
        }

        self.on_message_edit(ctx, event).await;
    }

    async fn message_delete(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        deleted_message_id: MessageId,
        guild_id: Option<GuildId>,
    ) {
        if guild_id.is_none() {
            return;
        }

        self.on_message_delete(
            ctx,
            guild_id.unwrap().get() as i64,
            channel_id.get() as i64,
            deleted_message_id.get() as i64,
        )
        .await;
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        if new.guild_id.is_none() {
            return;
        }

        match old {
            Some(old) => {
                if old.channel_id == new.channel_id {
                    return;
                }

                match old.channel_id {
                    Some(_) => {
                        if new.channel_id.is_some() {
                            // Moved
                            self.voice_move(ctx, old, new).await;
                        } else {
                            // Left
                            self.voice_leave(ctx, old).await;
                        }
                    }
                    None => {
                        if new.channel_id.is_some() {
                            // Joined
                            self.voice_join(ctx, new).await;
                        }
                    }
                }
            }
            None => {
                // Joined
                if new.channel_id.is_some() {
                    self.voice_join(ctx, new).await;
                }
            }
        }
    }
}
