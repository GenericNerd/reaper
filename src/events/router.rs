use serenity::all::{
    ActionExecution, Context, EventHandler, Guild as SerenityGuild, GuildId, Interaction,
    Member as SerenityMember, Ready, UnavailableGuild, User as SerenityUser,
};

use crate::{
    events::EventRouter,
    models::{guild::Guild, member::Member, user::User},
};

#[serenity::async_trait]
impl EventHandler for EventRouter {
    async fn ready(&self, ctx: Context, ready: Ready) {
        self.on_ready(ctx, ready).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        self.interaction_create(ctx, interaction).await;
    }

    async fn auto_moderation_action_execution(&self, ctx: Context, execution: ActionExecution) {
        self.automod_action_execution(ctx, execution).await;
    }

    async fn guild_create(&self, _ctx: Context, guild: SerenityGuild, is_new: Option<bool>) {
        let Some(new_guild) = is_new else {
            return;
        };
        if !new_guild {
            return;
        }
        let guild = Guild::from(guild.id);
        self.guild_create(guild).await;
    }

    async fn guild_delete(
        &self,
        _ctx: Context,
        guild: UnavailableGuild,
        _full_guild: Option<SerenityGuild>,
    ) {
        let guild = Guild::from(guild.id);
        self.guild_leave(guild).await;
    }

    async fn guild_member_addition(&self, ctx: Context, member: SerenityMember) {
        let member = Member::from(member);
        self.member_join(ctx, member).await;
    }

    async fn guild_member_removal(
        &self,
        _ctx: Context,
        guild_id: GuildId,
        user: SerenityUser,
        _member: Option<SerenityMember>,
    ) {
        let guild = Guild::from(guild_id);
        let user = User::from(user.id);
        self.member_leave(guild, user).await;
    }
}
