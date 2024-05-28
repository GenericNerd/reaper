use serenity::all::{CommandInteraction, CreateCommand, CreateEmbed};

use crate::models::{
    command::{Command, CommandContext, CommandContextReply},
    handler::Handler,
    response::{Response, ResponseResult},
};

pub struct PrivacyCommand;

#[async_trait::async_trait]
impl Command for PrivacyCommand {
    fn name(&self) -> &'static str {
        "privacy"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("privacy").description("Get Reaper's Privacy Policy")
    }

    async fn router(
        &self,
        _handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> ResponseResult {
        ctx.reply(
            cmd,
            Response::new()
                .embed(
                    CreateEmbed::new()
                        .title("Privacy Policy")
                        .description(
                            format!("You can view Reaper's Privacy Policy [here](https://github.com/GenericNerd/reaper/blob/development/PRIVACY.md).")
                        )
                        .color(0xeb966d)
                ).ephemeral(true)
        ).await
    }
}
