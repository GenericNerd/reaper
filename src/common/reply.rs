use std::sync::atomic::Ordering;

use serenity::{
    all::CommandInteraction,
    builder::{
        CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse,
    },
};
use tracing::{debug, error};

use crate::models::{
    command::{CommandContext, CommandError, CommandResult, Context, FailedCommandContext},
    response::Response,
};

#[async_trait::async_trait]
impl Context for CommandContext {
    async fn reply(&self, cmd: &CommandInteraction, response: Response) -> CommandResult {
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

            if let Err(err) = cmd.edit_response(&self.ctx.http, edit).await {
                error!("Attempted to edit a response to a command, failed with error: {err}");
                return Err(CommandError::SerenityError(err));
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

            match cmd
                .create_response(&self.ctx.http, CreateInteractionResponse::Message(reply))
                .await
            {
                Ok(_) => self.has_responsed.store(true, Ordering::Relaxed),
                Err(err) => {
                    error!("Attempted to create a response to a command, failed with error: {err}");
                    return Err(CommandError::SerenityError(err));
                }
            };
        }
        debug!("Took {:?} to reply to a command", start.elapsed());
        return Ok(());
    }
}

#[async_trait::async_trait]
impl Context for FailedCommandContext {
    async fn reply(&self, cmd: &CommandInteraction, response: Response) -> CommandResult {
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

        if let Err(err) = cmd
            .create_response(&self.ctx.http, CreateInteractionResponse::Message(reply))
            .await
        {
            error!("Attempted to create a response to a command, failed with error: {err}");
            return Err(CommandError::SerenityError(err));
        }

        debug!("Took {:?} to reply to a command", start.elapsed());
        return Ok(());
    }
}
