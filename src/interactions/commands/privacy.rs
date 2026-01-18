use async_trait::async_trait;
use serenity::all::{CommandInteraction, CreateCommand, CreateEmbed};

use crate::models::{
    interactions::{context::Context, responder::Responder, response::Response, traits::Command},
    permission::Permission,
    response::ResponseResult,
};

pub struct PrivacyCommand;

#[async_trait]
impl Command for PrivacyCommand {
    const ID: &'static str = "privacy";
    const PERMISSION: Option<Permission> = None;

    fn register(&self) -> CreateCommand {
        CreateCommand::new(Self::ID).description("Get information about the bot's privacy policy")
    }

    async fn execute(&self, ctx: &Context, command: &CommandInteraction) -> ResponseResult<()> {
        command.reply(
            ctx,
            Response::new()
                .embed(
                    CreateEmbed::new()
                        .title("Privacy Policy")
                        .description(
                            "You can view Reaper's Privacy Policy [here](https://github.com/GenericNerd/reaper/blob/development/PRIVACY.md).".to_string()
                        )
                        .color(0xeb_966d)
                ).ephemeral(true)
        ).await.map(|_| ())
    }
}
