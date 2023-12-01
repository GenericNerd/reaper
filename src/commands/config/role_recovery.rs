use serenity::{
    all::{ButtonStyle, CommandInteraction},
    builder::{CreateActionRow, CreateButton, CreateEmbed},
};

use crate::models::{
    command::{CommandContext, CommandContextReply},
    handler::Handler,
    response::{Response, ResponseError},
};

use super::{ConfigError, ConfigStage, EMBED_COLOR};

const ROLE_RECOVERY_TITLE: &str = "Configuration - Role Recovery";

pub struct RoleRecovery;
#[async_trait::async_trait]
impl ConfigStage for RoleRecovery {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(ROLE_RECOVERY_TITLE)
                            .description("Would you like to enable role recovery?\nThis will allow users to recover their roles if they leave and rejoin the server.")
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("yes")
                            .label("Yes")
                            .style(ButtonStyle::Success),
                        CreateButton::new("no")
                            .label("No")
                            .style(ButtonStyle::Secondary),
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));
        if let Some(interaction) = collector.await {
            interaction
                .create_response(
                    &ctx.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;
            match interaction.data.custom_id.as_str() {
                "yes" => {
                    sqlx::query!(
                        "UPDATE guild_role_recovery_config SET enabled = true WHERE guild_id = $1",
                        cmd.guild_id.unwrap().get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                "no" => {
                    sqlx::query!(
                        "UPDATE guild_role_recovery_config SET enabled = false WHERE guild_id = $1",
                        cmd.guild_id.unwrap().get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}
