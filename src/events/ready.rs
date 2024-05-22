use std::sync::{atomic::AtomicBool, Arc};

use serenity::{
    all::{ChannelId, Command, PartialGuild},
    gateway::ActivityData,
    model::prelude::Ready,
    prelude::Context,
};
use tracing::{debug, error, info};

use crate::{
    commands::{get_command_list, giveaway::interaction::new_giveaway_entry_handler},
    events::expire::{expire_actions, expire_giveaways},
    models::{
        command::CommandContext,
        giveaway::{DatabaseGiveaway, Giveaway},
        handler::Handler,
    },
};

impl Handler {
    pub async fn on_ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected", ready.user.name);

        ctx.set_activity(Some(ActivityData::playing(
            "with users' emotions (but faster)",
        )));

        debug!("Starting action expiration loop");
        tokio::spawn(expire_actions(self.clone(), ctx.clone()));
        debug!("Starting giveaway expiration loop");
        tokio::spawn(expire_giveaways(self.clone()));

        match sqlx::query_as!(DatabaseGiveaway, "SELECT * FROM giveaways")
            .fetch_all(&self.main_database)
            .await
        {
            Ok(giveaways) => {
                debug!("Adding {} giveaways to giveaway list", giveaways.len());
                for giveaway in giveaways {
                    let Ok(channel) = ctx
                        .http
                        .get_channel(ChannelId::new(giveaway.channel_id as u64))
                        .await
                    else {
                        error!("Failed to get channel for giveaway {}", giveaway.id);
                        continue;
                    };
                    let Some(guild_id) = channel.guild().map(|guild| guild.guild_id) else {
                        error!("Failed to get guild for giveaway {}", giveaway.id);
                        continue;
                    };

                    let mut temp_guild = guild_id
                        .to_guild_cached(&ctx.cache)
                        .map(|guild| PartialGuild::from(guild.clone()));
                    if temp_guild.is_none() {
                        temp_guild = if let Ok(guild) = guild_id.to_partial_guild(&ctx.http).await {
                            Some(guild)
                        } else {
                            return;
                        }
                    }
                    let guild = temp_guild.unwrap();

                    let command_context = CommandContext {
                        ctx: ctx.clone(),
                        has_responsed: Arc::new(AtomicBool::new(false)),
                        user_permissions: vec![],
                        highest_role: u16::max_value(),
                        guild,
                    };

                    tokio::spawn(new_giveaway_entry_handler(
                        self.clone(),
                        command_context,
                        Giveaway::from(giveaway),
                    ));
                }
            }
            Err(e) => error!(
                "Failed to get giveaways from database. Failed with error: {}",
                e
            ),
        }

        debug!("Adding current commands to slash commands list");
        let mut successful_commands = vec![];
        for command in get_command_list() {
            match Command::create_global_command(&ctx.http, command.register()).await {
                Ok(_) => successful_commands.push(command.name()),
                Err(e) => error!(
                    "Attempted to register command {} but failed with error: {}",
                    command.name(),
                    e
                ),
            }
        }
        info!(
            "Successfully registered commands: {}. {} is ready!",
            successful_commands.join(", "),
            ready.user.name
        );
    }
}
