use async_trait::async_trait;
use diesel::{QueryDsl, RunQueryDsl};
use humanize_duration::{Truncate, prelude::DurationExt};
use serenity::all::{CommandInteraction, CreateCommand, CreateEmbed};

use crate::models::{
    bot::Bot,
    interactions::{context::Context, responder::Responder, response::Response, traits::Command},
    permission::Permission,
    response::{ReaperError, ResponseError, ResponseResult},
};

pub struct InfoCommand;

#[async_trait]
impl Command for InfoCommand {
    const ID: &'static str = "info";
    const PERMISSION: Option<Permission> = None;

    fn register(&self) -> CreateCommand {
        CreateCommand::new(Self::ID).description("Get information about the bot")
    }

    #[tracing::instrument(skip(self, ctx, interaction))]
    async fn execute(&self, ctx: &Context, interaction: &CommandInteraction) -> ResponseResult<()> {
        let context = ctx.get_populated_context()?;

        let shard_runners = Bot::instance().shard_manager().runners.lock().await;
        let Some(shard_latency) = shard_runners
            .get(&context.serenity_context.shard_id)
            .map(|runner| runner.latency)
        else {
            return Err(ResponseError::Reaper(
                ReaperError::FailedToObtainShardLatency,
            ));
        };

        let guild_count = {
            use crate::schema::moderation_configuration::dsl::*;
            moderation_configuration
                .count()
                .get_result::<i64>(&mut Bot::instance().postgres())?
        };

        let action_count = {
            use crate::schema::actions::dsl::*;
            actions
                .count()
                .get_result::<i64>(&mut Bot::instance().postgres())?
        };

        let giveaway_count = {
            use crate::schema::giveaways::dsl::*;
            giveaways
                .count()
                .get_result::<i64>(&mut Bot::instance().postgres())?
        };

        interaction.reply(&ctx, Response::new().embed(
            CreateEmbed::new()
                .title("Reaper Information")
                .fields(vec![
                    (
                        "Network",
                        format!(
                            "Shard ID {}\nLatency: {}",
                            context.serenity_context.shard_id,
                            shard_latency.map_or("Pending".to_string(),|latency| latency.human(Truncate::Millis).to_string())
                        ),
                        true,
                    ),
                    (
                        "Information",
                        format!(
                            "Serving {guild_count} guilds\nHandled {action_count} actions\nRunning {giveaway_count} giveaways"
                        ),
                        true,
                    ),
                    (
                        "Meta",
                        format!(
                            "Version: {}\nUptime: {}",
                            env!("CARGO_PKG_VERSION"),
                            Bot::instance().start_time().elapsed().human(Truncate::Second).to_string()
                        ),
                        true,
                    ),
                ])
                .color(0xeb_966d),
            ),
        ).await.map(|_| ())
    }
}
