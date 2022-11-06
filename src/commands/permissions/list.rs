use serenity::{prelude::Context, model::prelude::interaction::application_command::ApplicationCommandInteraction};
use crate::{mongo::structs::Permissions, commands::{structs::CommandError, utils::send_message}};

pub async fn run(ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(), CommandError> {
    let mut message_content = "The following permissions are available:\n".to_string();
    for variant in Permissions::variants().iter() {
        message_content.push_str(&format!("`{}`\n", variant.to_string()));
    }
    return send_message(&ctx, cmd, message_content).await;
}