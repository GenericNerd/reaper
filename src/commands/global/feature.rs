use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, CreateEmbed};

use crate::{
    common::options::Options,
    models::{
        command::{CommandContext, CommandContextReply},
        handler::Handler,
        response::{Response, ResponseError, ResponseResult},
    },
};

async fn kill(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
    feature: String,
) -> ResponseResult {
    let feature_flag = match sqlx::query!(
        "SELECT active FROM global_kills WHERE feature = $1",
        feature
    )
    .fetch_optional(&handler.main_database)
    .await?
    {
        Some(row) => row.active,
        None => return Err(ResponseError::Execution("Feature not found", None)),
    };

    if !feature_flag {
        return Err(ResponseError::Execution("Feature already killed", None));
    }

    sqlx::query!(
        "UPDATE global_kills SET active = false, killed_by = $1 WHERE feature = $2",
        cmd.user.id.get() as i64,
        feature
    )
    .execute(&handler.main_database)
    .await?;

    ctx.reply(
        cmd,
        Response::new().embed(
            CreateEmbed::new()
                .title("Feature killed")
                .description(format!("`{feature}` has been killed"))
                .color(0xff0000),
        ),
    )
    .await
}

async fn revive(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
    feature: String,
) -> ResponseResult {
    let feature_flag = match sqlx::query!(
        "SELECT active FROM global_kills WHERE feature = $1",
        feature
    )
    .fetch_optional(&handler.main_database)
    .await?
    {
        Some(row) => row.active,
        None => return Err(ResponseError::Execution("Feature not found", None)),
    };

    if feature_flag {
        return Err(ResponseError::Execution("Feature already active", None));
    }

    sqlx::query!(
        "UPDATE global_kills SET active = true, killed_by = NULL WHERE feature = $1",
        feature
    )
    .execute(&handler.main_database)
    .await?;

    ctx.reply(
        cmd,
        Response::new().embed(
            CreateEmbed::new()
                .title("Feature revived")
                .description(format!("`{feature}` has been revived"))
                .color(0x00ff00),
        ),
    )
    .await
}

async fn status(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
    feature: String,
) -> ResponseResult {
    let feature_flag = match sqlx::query!(
        "SELECT active, killed_by FROM global_kills WHERE feature = $1",
        feature
    )
    .fetch_optional(&handler.main_database)
    .await?
    {
        Some(row) => row,
        None => return Err(ResponseError::Execution("Feature not found", None)),
    };

    ctx.reply(
        cmd,
        Response::new().embed(
            CreateEmbed::new()
                .title("Feature status")
                .description(format!("Viewing `{feature}`"))
                .fields(vec![
                    (
                        "Active",
                        match feature_flag.active {
                            true => "✅".to_string(),
                            false => "❌".to_string(),
                        },
                        true,
                    ),
                    (
                        "Killed by",
                        match feature_flag.killed_by {
                            Some(killed_by) => format!("<@{}>", killed_by),
                            None => "N/A".to_string(),
                        },
                        true,
                    ),
                ])
                .color(match feature_flag.active {
                    true => 0x00ff00,
                    false => 0xff0000,
                }),
        ),
    )
    .await
}

pub async fn router(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
    option: &CommandDataOption,
) -> ResponseResult {
    let options = Options {
        options: cmd.data.options(),
    };

    let feature = match options.get_string("feature").into_owned() {
        Some(feature) => feature,
        None => {
            return Err(ResponseError::Execution(
                "Invalid command option",
                Some("The command option you provided is invalid".to_string()),
            ))
        }
    };

    match &option.value {
        CommandDataOptionValue::SubCommandGroup(group) => {
            match group.first().unwrap().name.as_str() {
                "kill" => kill(handler, ctx, cmd, feature).await,
                "revive" => revive(handler, ctx, cmd, feature).await,
                "status" => status(handler, ctx, cmd, feature).await,
                _ => Err(ResponseError::Execution(
                    "Invalid command option",
                    Some("The command option you provided is invalid".to_string()),
                )),
            }
        }
        _ => Err(ResponseError::Execution(
            "Invalid command option",
            Some("The command option you provided is invalid".to_string()),
        )),
    }
}
