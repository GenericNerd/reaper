use std::sync::atomic::Ordering;

use serenity::{
    all::{CommandInteraction, Message},
    builder::{
        CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse,
    },
};
use tracing::{debug, error};

use crate::models::{
    command::{
        CommandContext, CommandContextReply, FailedCommandContext, InteractionContext,
        InteractionContextReply,
    },
    response::{Response, ResponseError, ResponseResult},
};

#[async_trait::async_trait]
impl CommandContextReply for CommandContext {
    async fn reply_get_message(
        &self,
        cmd: &CommandInteraction,
        response: Response,
    ) -> Result<Message, ResponseError> {
        let start = std::time::Instant::now();
        let message = if self.has_responsed.load(Ordering::Relaxed) {
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

            match cmd.edit_response(&self.ctx.http, edit).await {
                Ok(message) => message,
                Err(err) => {
                    error!("Attempted to edit a response to a command, failed with error: {err}");
                    return Err(ResponseError::SerenityError(err));
                }
            }
        } else {
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
            if response.ephemeral {
                reply = reply.ephemeral(true);
            }

            match cmd
                .create_response(&self.ctx.http, CreateInteractionResponse::Message(reply))
                .await
            {
                Ok(_) => {
                    self.has_responsed.store(true, Ordering::Relaxed);
                    match cmd.get_response(&self.ctx.http).await {
                        Ok(message) => message,
                        Err(err) => {
                            error!(
                                "A message was sent, but failed to fetch, failed with error: {err}"
                            );
                            return Err(ResponseError::SerenityError(err));
                        }
                    }
                }
                Err(err) => {
                    error!("Attempted to create a response to a command, failed with error: {err}");
                    return Err(ResponseError::SerenityError(err));
                }
            }
        };
        debug!("Took {:?} to reply to a command", start.elapsed());
        return Ok(message);
    }

    async fn reply(&self, cmd: &CommandInteraction, response: Response) -> ResponseResult {
        self.reply_get_message(cmd, response).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl CommandContextReply for FailedCommandContext {
    async fn reply_get_message(
        &self,
        cmd: &CommandInteraction,
        response: Response,
    ) -> Result<Message, ResponseError> {
        let start = std::time::Instant::now();
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
        if response.ephemeral {
            reply = reply.ephemeral(true);
        }

        let message = match cmd
            .create_response(&self.ctx.http, CreateInteractionResponse::Message(reply))
            .await
        {
            Ok(_) => match cmd.get_response(&self.ctx.http).await {
                Ok(message) => message,
                Err(err) => {
                    error!("A message was sent, but failed to fetch, failed with error: {err}");
                    return Err(ResponseError::SerenityError(err));
                }
            },
            Err(err) => {
                error!("Attempted to create a response to a command, failed with error: {err}");
                return Err(ResponseError::SerenityError(err));
            }
        };

        debug!("Took {:?} to reply to a command", start.elapsed());
        return Ok(message);
    }

    async fn reply(&self, cmd: &CommandInteraction, response: Response) -> ResponseResult {
        self.reply_get_message(cmd, response).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl InteractionContextReply for InteractionContext {
    async fn reply(&self, response: Response) -> ResponseResult {
        let start = std::time::Instant::now();
        if self.has_responsed.load(Ordering::Relaxed) {
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

            if let Err(err) = self.interaction.edit_response(&self.ctx.http, edit).await {
                error!("Attempted to edit a interaction response, failed with error: {err}");
                return Err(ResponseError::SerenityError(err));
            }
        } else {
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
            if response.ephemeral {
                reply = reply.ephemeral(true);
            }

            match self
                .interaction
                .create_response(&self.ctx.http, CreateInteractionResponse::Message(reply))
                .await
            {
                Ok(_) => {
                    self.has_responsed.store(true, Ordering::Relaxed);
                }
                Err(err) => {
                    error!("Attempted to create a response to a command, failed with error: {err}");
                    return Err(ResponseError::SerenityError(err));
                }
            }
        }

        debug!("Took {:?} to reply to a command", start.elapsed());
        Ok(())
    }
}
