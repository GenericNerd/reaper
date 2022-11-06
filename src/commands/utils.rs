use serenity::{model::prelude::interaction::{application_command::ApplicationCommandInteraction, InteractionResponseType}, prelude::Context};

use super::structs::CommandError;

pub async fn send_message(ctx: &Context, cmd: &ApplicationCommandInteraction, content: String) -> Result<(), CommandError> {
    match cmd.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message
                    .content(content)
            })
    }).await {
        Ok(_) => {return Ok(())},
        Err(err) => {
            return Err(CommandError {
                message: format!("An error occurred while sending the response to the user. The error was: {}", err),
                command_error: Some(err)
            });
        }
    }
}