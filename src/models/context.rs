use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tracing::error;

use serenity::all::{
    CommandInteraction, ComponentInteraction, Context as SerenityContext, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage, CreateModal,
    EditAttachments, EditInteractionResponse, Message, PartialGuild,
};

use crate::models::{
    guild::Guild,
    permissions::Permission,
    response::{ExecutionError, InternalError, ReaperError, Response, ResponseError},
    user::User,
};

pub trait ContextReply<T> {
    async fn reply(&self, interaction: &T, response: Response) -> Result<Message, ResponseError>;
    async fn error_message(
        &self,
        interaction: &T,
        error: ResponseError,
    ) -> Result<Message, ResponseError>;
    async fn error_message_ref(
        &self,
        interaction: &T,
        error: &ResponseError,
    ) -> Result<Message, ResponseError>;
}

pub trait ContextComponentReplies<T> {
    async fn modal(&self, interaction: &T, modal: CreateModal) -> Result<(), ResponseError>;
}

#[derive(Debug, Clone)]
pub struct UnpopulatedContext<'a> {
    pub ctx: &'a SerenityContext,
}

#[derive(Debug, Clone)]
pub struct PopulatedContext<'a> {
    pub ctx: &'a SerenityContext,
    pub has_responded: Arc<AtomicBool>,
    pub user: User,
    pub user_permissions: Vec<Permission>,
    pub highest_role: u16,
    pub partial_guild: PartialGuild,
    pub guild: Guild,
}

#[derive(Debug, Clone)]
pub enum Context<'a> {
    Unpopulated(UnpopulatedContext<'a>),
    Populated(Box<PopulatedContext<'a>>),
}

impl Context<'_> {
    pub fn get_populated_context(&self) -> Result<&PopulatedContext<'_>, ResponseError> {
        if let Self::Populated(context) = self {
            return Ok(context);
        }
        Err(ResponseError::Execution(ExecutionError::Internal(
            InternalError::FailedToObtainContext,
        )))
    }
}

fn error_message(error: &ResponseError) -> CreateEmbed {
    match error {
        ResponseError::Execution(err) => CreateEmbed::new()
            .title(err.title())
            .description(err.description().clone().unwrap_or(String::new()))
            .color(0xff_0000),
        ResponseError::Sqlx(err) => CreateEmbed::new()
            .title("A database error occurred while executing the command")
            .description(format!("```{err:?}```"))
            .footer(CreateEmbedFooter::new(
                "Please report this issue to developers",
            ))
            .color(0xff_0000),
        ResponseError::Serenity(err) => CreateEmbed::new()
            .title("A Discord error occurred while executing the command")
            .description(format!("```{err:?}```"))
            .footer(CreateEmbedFooter::new(
                "Please report this issue to developers if this persists",
            ))
            .color(0xff_0000),
        ResponseError::Redis(err) => CreateEmbed::new()
            .title("A Redis error occurred while executing the command")
            .description(format!("```{err:?}```"))
            .footer(CreateEmbedFooter::new(
                "Please report this issue to developers",
            ))
            .color(0xff_0000),
    }
}

impl ContextReply<CommandInteraction> for Context<'_> {
    #[tracing::instrument(skip(self, cmd, response), fields(command_name = cmd.data.name))]
    async fn reply(
        &self,
        cmd: &CommandInteraction,
        response: Response,
    ) -> Result<Message, ResponseError> {
        let ctx = match self {
            Context::Unpopulated(ctx) => ctx.ctx,
            Context::Populated(ctx) => ctx.ctx,
        };
        let has_responded = match self {
            Context::Unpopulated(_) => false,
            Context::Populated(ctx) => ctx.has_responded.load(Ordering::Relaxed),
        };

        if has_responded {
            let mut edit = EditInteractionResponse::new();
            if let Some(content) = response.content {
                edit = edit.content(content);
            }
            if let Some(embeds) = response.embeds {
                edit = edit.embeds(embeds);
            }
            if let Some(allowed_mentions) = response.allowed_mentions {
                edit = edit.allowed_mentions(allowed_mentions);
            }
            if let Some(components) = response.components {
                edit = edit.components(components);
            }
            if let Some(attachments) = response.attachments {
                edit = edit.attachments(EditAttachments::new().add(attachments));
            }

            return match cmd.edit_response(&ctx.http, edit).await {
                Ok(msg) => Ok(msg),
                Err(err) => {
                    error!("Attempted to edit a response to a command, failed with error: {err}");
                    Err(ResponseError::Serenity(Box::new(err)))
                }
            };
        }

        let mut reply = CreateInteractionResponseMessage::new();
        if let Some(content) = response.content {
            reply = reply.content(content);
        }
        if let Some(embeds) = response.embeds {
            reply = reply.embeds(embeds);
        }
        if let Some(allowed_mentions) = response.allowed_mentions {
            reply = reply.allowed_mentions(allowed_mentions);
        }
        if let Some(components) = response.components {
            reply = reply.components(components);
        }
        if let Some(attachments) = response.attachments {
            reply = reply.add_file(attachments);
        }
        reply = reply.ephemeral(response.ephemeral);

        match cmd
            .create_response(&ctx.http, CreateInteractionResponse::Message(reply))
            .await
        {
            Ok(()) => {
                if let Context::Populated(context) = self {
                    context.has_responded.store(true, Ordering::Relaxed);
                }
                match cmd.get_response(&ctx.http).await {
                    Ok(message) => Ok(message),
                    Err(err) => {
                        error!("A message was sent, but failed to fetch. Failed with error: {err}");
                        Err(ResponseError::Serenity(Box::new(err)))
                    }
                }
            }
            Err(err) => {
                error!("Failed to create response to command: {err}");
                Err(ResponseError::Serenity(Box::new(err)))
            }
        }
    }

    async fn error_message(
        &self,
        interaction: &CommandInteraction,
        error: ResponseError,
    ) -> Result<Message, ResponseError> {
        let embed = error_message(&error);

        self.reply(
            interaction,
            Response::new()
                .embed(embed)
                .ephemeral(true)
                .components(vec![]),
        )
        .await
    }

    async fn error_message_ref(
        &self,
        interaction: &CommandInteraction,
        error: &ResponseError,
    ) -> Result<Message, ResponseError> {
        let embed = error_message(error);

        self.reply(
            interaction,
            Response::new()
                .embed(embed)
                .ephemeral(true)
                .components(vec![]),
        )
        .await
    }
}

impl ContextReply<ComponentInteraction> for Context<'_> {
    #[tracing::instrument(skip(self, interaction, response))]
    async fn reply(
        &self,
        interaction: &ComponentInteraction,
        response: Response,
    ) -> Result<Message, ResponseError> {
        let ctx = match self {
            Context::Unpopulated(ctx) => ctx.ctx,
            Context::Populated(ctx) => ctx.ctx,
        };
        let has_responded = match self {
            Context::Unpopulated(_) => false,
            Context::Populated(ctx) => ctx.has_responded.load(Ordering::Relaxed),
        };

        if has_responded {
            let mut edit = EditInteractionResponse::new();
            if let Some(content) = response.content {
                edit = edit.content(content);
            }
            if let Some(embeds) = response.embeds {
                edit = edit.embeds(embeds);
            }
            if let Some(allowed_mentions) = response.allowed_mentions {
                edit = edit.allowed_mentions(allowed_mentions);
            }
            if let Some(components) = response.components {
                edit = edit.components(components);
            }
            if let Some(attachments) = response.attachments {
                edit = edit.attachments(EditAttachments::new().add(attachments));
            }

            return match interaction.edit_response(&ctx.http, edit).await {
                Ok(msg) => Ok(msg),
                Err(err) => {
                    error!("Attempted to edit a response to a component, failed with error: {err}");
                    Err(ResponseError::Serenity(Box::new(err)))
                }
            };
        }

        let mut reply = CreateInteractionResponseMessage::new();
        if let Some(content) = response.content {
            reply = reply.content(content);
        }
        if let Some(embeds) = response.embeds {
            reply = reply.embeds(embeds);
        }
        if let Some(allowed_mentions) = response.allowed_mentions {
            reply = reply.allowed_mentions(allowed_mentions);
        }
        if let Some(components) = response.components {
            reply = reply.components(components);
        }
        if let Some(attachments) = response.attachments {
            reply = reply.add_file(attachments);
        }
        reply = reply.ephemeral(response.ephemeral);

        match interaction
            .create_response(&ctx.http, CreateInteractionResponse::UpdateMessage(reply))
            .await
        {
            Ok(()) => {
                if let Context::Populated(context) = self {
                    context.has_responded.store(true, Ordering::Relaxed);
                }
                match interaction.get_response(&ctx.http).await {
                    Ok(message) => Ok(message),
                    Err(err) => {
                        error!("A message was sent, but failed to fetch. Failed with error: {err}");
                        Err(ResponseError::Serenity(Box::new(err)))
                    }
                }
            }
            Err(err) => {
                error!("Failed to create response to component: {err}");
                Err(ResponseError::Serenity(Box::new(err)))
            }
        }
    }

    async fn error_message(
        &self,
        interaction: &ComponentInteraction,
        error: ResponseError,
    ) -> Result<Message, ResponseError> {
        let embed = error_message(&error);

        self.reply(
            interaction,
            Response::new()
                .embed(embed)
                .ephemeral(true)
                .components(vec![]),
        )
        .await
    }

    async fn error_message_ref(
        &self,
        interaction: &ComponentInteraction,
        error: &ResponseError,
    ) -> Result<Message, ResponseError> {
        let embed = error_message(error);

        self.reply(
            interaction,
            Response::new()
                .embed(embed)
                .ephemeral(true)
                .components(vec![]),
        )
        .await
    }
}

impl ContextComponentReplies<ComponentInteraction> for Context<'_> {
    async fn modal(
        &self,
        interaction: &ComponentInteraction,
        modal: CreateModal,
    ) -> Result<(), ResponseError> {
        let ctx = match self {
            Context::Populated(ctx) => {
                ctx.has_responded.store(true, Ordering::Relaxed);
                ctx.ctx
            }
            Context::Unpopulated(ctx) => ctx.ctx,
        };

        interaction
            .create_response(&ctx.http, CreateInteractionResponse::Modal(modal))
            .await
            .map_err(|err| ResponseError::Serenity(Box::new(err)))
    }
}
