use async_trait::async_trait;
use serenity::all::{CommandInteraction, CreateCommand, CreateEmbed};

use crate::{
    commands::Command,
    models::{
        context::{Context, ContextReply},
        permissions::Permission,
        response::{Response, ResponseResult},
    },
};

pub struct PrivacyCommand;

#[async_trait]
impl Command for PrivacyCommand {
    fn name(&self) -> &'static str {
        "privacy"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("privacy").description("Get Reaper's Privacy Policy")
    }

    fn required_permission(&self) -> Option<Permission> {
        None
    }

    async fn router(&self, ctx: &Context<'_>, command: &CommandInteraction) -> ResponseResult {
        ctx.reply(
            command,
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
