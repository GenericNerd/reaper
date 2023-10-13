use serenity::{
    all::{CommandInteraction, ComponentInteraction, Message, PartialGuild},
    builder::{CreateCommand, CreateEmbed},
    prelude::Context as IncomingContext,
};
use std::sync::{atomic::AtomicBool, Arc};
use strum::IntoEnumIterator;

use crate::database::postgres::permissions::get_user;

use super::{
    handler::Handler,
    permissions::Permission,
    response::{Response, ResponseError, ResponseResult},
};

#[async_trait::async_trait]
pub trait CommandContextReply {
    async fn reply(&self, cmd: &CommandInteraction, response: Response) -> ResponseResult;
    async fn reply_get_message(
        &self,
        cmd: &CommandInteraction,
        response: Response,
    ) -> Result<Message, ResponseError>;
}

#[async_trait::async_trait]
pub trait InteractionContextReply {
    async fn reply(&self, response: Response) -> ResponseResult;
}

#[derive(Clone)]
pub struct CommandContext {
    pub ctx: IncomingContext,
    pub has_responsed: Arc<AtomicBool>,
    pub user_permissions: Vec<Permission>,
    pub guild: PartialGuild,
}

pub struct InteractionContext {
    pub ctx: IncomingContext,
    pub interaction: ComponentInteraction,
    pub has_responsed: Arc<AtomicBool>,
    pub user_permissions: Vec<Permission>,
}

impl InteractionContext {
    pub async fn new(
        handler: &Handler,
        ctx: IncomingContext,
        interaction: &ComponentInteraction,
    ) -> Self {
        let Some(guild_id) = interaction.guild_id else {
            return Self {
                ctx,
                interaction: interaction.clone(),
                has_responsed: Arc::new(AtomicBool::new(false)),
                user_permissions: vec![],
            };
        };

        let mut temp_guild = guild_id
            .to_guild_cached(&ctx.cache)
            .map(|guild| PartialGuild::from(guild.clone()));
        if temp_guild.is_none() {
            temp_guild = if let Ok(guild) = guild_id.to_partial_guild(&ctx.http).await {
                Some(guild)
            } else {
                None
            }
        }

        let guild = temp_guild.unwrap();

        if guild.owner_id == interaction.user.id {
            return Self {
                ctx,
                interaction: interaction.clone(),
                has_responsed: Arc::new(AtomicBool::new(false)),
                user_permissions: Permission::iter().collect::<Vec<_>>(),
            };
        }

        Self {
            ctx,
            interaction: interaction.clone(),
            has_responsed: Arc::new(AtomicBool::new(false)),
            user_permissions: get_user(
                handler,
                guild_id.0.get() as i64,
                interaction.user.id.0.get() as i64,
            )
            .await,
        }
    }

    pub async fn error_message(&self, error: ResponseError) -> ResponseResult {
        let embed = match error {
            ResponseError::Execution(title, description) => CreateEmbed::new()
                .title(title)
                .description(description.unwrap_or(String::new()))
                .color(0xff0000),
            ResponseError::Serenity(err) => CreateEmbed::new()
                .title("A Discord error occured while executing the command")
                .description(format!("```{err:?}```"))
                .color(0xff0000),
            ResponseError::Redis(_) => return Ok(()),
        };

        self.reply(Response::new().embed(embed).ephemeral(true))
            .await
    }
}

pub struct FailedCommandContext {
    pub ctx: IncomingContext,
}

#[async_trait::async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &'static str;
    fn register(&self) -> CreateCommand;
    async fn router(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        command: &CommandInteraction,
    ) -> ResponseResult;
}
