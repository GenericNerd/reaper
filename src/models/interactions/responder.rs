use std::sync::atomic::Ordering;

use async_trait::async_trait;
use serenity::all::{
    CommandInteraction, ComponentInteraction, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage, EditAttachments,
    EditInteractionResponse, Message, ModalInteraction,
};
use tracing::error;

use crate::models::{
    interactions::{context::Context, response::Response},
    response::{ResponseError, ResponseResult},
};

fn error_message(error: &ResponseError) -> CreateEmbed {
    match error {
        ResponseError::Reaper(err) => CreateEmbed::new()
            .title("An internal error occurred")
            .description(format!("The error was: ```{err}```\nPlease try again and if this issue persists, please contact support."))
            .color(0xff_0000),
        ResponseError::Diesel(err) => CreateEmbed::new()
            .title("A database error occurred while executing the command")
            .description(format!("```{err:?}```")).footer(CreateEmbedFooter::new(
                "Please report this issue to developers if this persists",
            ))
            .color(0xff_0000),
        ResponseError::Json(err) => CreateEmbed::new()
            .title("A JSON serialization error occurred while executing the command")
            .description(format!("```{err:?}```")).footer(CreateEmbedFooter::new(
                "Please report this issue to developers if this persists",
            ))
            .color(0xff_0000),
        ResponseError::Serenity(err) => CreateEmbed::new()
            .title("A Discord error occurred while executing the command")
            .description(format!("```{err:?}```"))
            .footer(CreateEmbedFooter::new(
                "Please report this issue to developers if this persists",
            ))
            .color(0xff_0000),
    }
}

#[async_trait]
pub trait Responder: Send + Sync {
    async fn ack(&self, ctx: &Context) -> ResponseResult<()>;
    async fn reply(&self, ctx: &Context, response: Response) -> ResponseResult<Message>;
    async fn error_message(&self, ctx: &Context, error: &ResponseError) -> ResponseResult<Message>;
}

#[async_trait]
impl Responder for CommandInteraction {
    #[tracing::instrument(skip(self, ctx))]
    async fn ack(&self, ctx: &Context) -> ResponseResult<()> {
        let context = match ctx {
            Context::Unpopulated(ctx) => &ctx.serenity_context,
            Context::Populated(ctx) => {
                ctx.has_responded.store(true, Ordering::Relaxed);
                &ctx.serenity_context
            }
        };
        self.defer(&context.http).await.map_err(ResponseError::from)
    }

    #[tracing::instrument(skip(self, ctx, response))]
    async fn reply(&self, ctx: &Context, response: Response) -> ResponseResult<Message> {
        let context = match ctx {
            Context::Unpopulated(ctx) => &ctx.serenity_context,
            Context::Populated(ctx) => &ctx.serenity_context,
        };
        let has_responded = match &ctx {
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

            return match self.edit_response(&context.http, edit).await {
                Ok(msg) => Ok(msg),
                Err(err) => {
                    error!(error = %err, "Attempted to edit a response to a command");
                    Err(ResponseError::Serenity(err))
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

        match self
            .create_response(&context.http, CreateInteractionResponse::Message(reply))
            .await
        {
            Ok(()) => {
                if let Context::Populated(context) = &ctx {
                    context.has_responded.store(true, Ordering::Relaxed);
                }
                match self.get_response(&context.http).await {
                    Ok(message) => Ok(message),
                    Err(err) => {
                        error!(error = %err, "A message was sent, but failed to fetch");
                        Err(ResponseError::Serenity(err))
                    }
                }
            }
            Err(err) => {
                error!(error = %err, "Failed to create response to command");
                Err(ResponseError::Serenity(err))
            }
        }
    }

    #[tracing::instrument(skip(self, ctx, error), fields(error = %error))]
    async fn error_message(&self, ctx: &Context, error: &ResponseError) -> ResponseResult<Message> {
        let embed = error_message(&error);

        self.reply(
            ctx,
            Response::new()
                .embed(embed)
                .ephemeral(true)
                .components(vec![]),
        )
        .await
    }
}

#[async_trait]
impl Responder for ComponentInteraction {
    #[tracing::instrument(skip(self, ctx))]
    async fn ack(&self, ctx: &Context) -> ResponseResult<()> {
        let context = match ctx {
            Context::Unpopulated(ctx) => &ctx.serenity_context,
            Context::Populated(ctx) => {
                ctx.has_responded.store(true, Ordering::Relaxed);
                &ctx.serenity_context
            }
        };
        self.create_response(&context.http, CreateInteractionResponse::Acknowledge)
            .await
            .map_err(ResponseError::from)
    }

    #[tracing::instrument(skip(self, ctx, response))]
    async fn reply(&self, ctx: &Context, response: Response) -> ResponseResult<Message> {
        let context = match ctx {
            Context::Unpopulated(ctx) => &ctx.serenity_context,
            Context::Populated(ctx) => &ctx.serenity_context,
        };
        let has_responded = match &ctx {
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

            return match self.edit_response(&context.http, edit).await {
                Ok(msg) => Ok(msg),
                Err(err) => {
                    error!(error = %err, "Attempted to edit a response to a command");
                    Err(ResponseError::Serenity(err))
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

        match self
            .create_response(&context.http, CreateInteractionResponse::Message(reply))
            .await
        {
            Ok(()) => {
                if let Context::Populated(context) = &ctx {
                    context.has_responded.store(true, Ordering::Relaxed);
                }
                match self.get_response(&context.http).await {
                    Ok(message) => Ok(message),
                    Err(err) => {
                        error!(error = %err, "A message was sent, but failed to fetch");
                        Err(ResponseError::Serenity(err))
                    }
                }
            }
            Err(err) => {
                error!(error = %err, "Failed to create response to command");
                Err(ResponseError::Serenity(err))
            }
        }
    }

    #[tracing::instrument(skip(self, ctx, error), fields(error = %error))]
    async fn error_message(&self, ctx: &Context, error: &ResponseError) -> ResponseResult<Message> {
        let embed = error_message(&error);

        self.reply(
            ctx,
            Response::new()
                .embed(embed)
                .ephemeral(true)
                .components(vec![]),
        )
        .await
    }
}

#[async_trait]
impl Responder for ModalInteraction {
    #[tracing::instrument(skip(self, ctx))]
    async fn ack(&self, ctx: &Context) -> ResponseResult<()> {
        let context = match ctx {
            Context::Unpopulated(ctx) => &ctx.serenity_context,
            Context::Populated(ctx) => {
                ctx.has_responded.store(true, Ordering::Relaxed);
                &ctx.serenity_context
            }
        };
        self.create_response(&context.http, CreateInteractionResponse::Acknowledge)
            .await
            .map_err(ResponseError::from)
    }

    #[tracing::instrument(skip(self, ctx, response))]
    async fn reply(&self, ctx: &Context, response: Response) -> ResponseResult<Message> {
        let context = match ctx {
            Context::Unpopulated(ctx) => &ctx.serenity_context,
            Context::Populated(ctx) => &ctx.serenity_context,
        };
        let has_responded = match &ctx {
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

            return match self.edit_response(&context.http, edit).await {
                Ok(msg) => Ok(msg),
                Err(err) => {
                    error!(error = %err, "Attempted to edit a response to a command");
                    Err(ResponseError::Serenity(err))
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

        match self
            .create_response(&context.http, CreateInteractionResponse::Message(reply))
            .await
        {
            Ok(()) => {
                if let Context::Populated(context) = &ctx {
                    context.has_responded.store(true, Ordering::Relaxed);
                }
                match self.get_response(&context.http).await {
                    Ok(message) => Ok(message),
                    Err(err) => {
                        error!(error = %err, "A message was sent, but failed to fetch");
                        Err(ResponseError::Serenity(err))
                    }
                }
            }
            Err(err) => {
                error!(error = %err, "Failed to create response to command");
                Err(ResponseError::Serenity(err))
            }
        }
    }

    #[tracing::instrument(skip(self, ctx, error), fields(error = %error))]
    async fn error_message(&self, ctx: &Context, error: &ResponseError) -> ResponseResult<Message> {
        let embed = error_message(&error);

        self.reply(
            ctx,
            Response::new()
                .embed(embed)
                .ephemeral(true)
                .components(vec![]),
        )
        .await
    }
}
