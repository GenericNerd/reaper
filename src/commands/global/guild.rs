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
    guild: i64,
) -> ResponseResult {
    if sqlx::query!(
        "SELECT guild_id FROM guild_kills WHERE guild_id = $1",
        guild
    )
    .fetch_optional(&handler.main_database)
    .await?
    .is_some()
    {
        return Err(ResponseError::Execution("Guild is already killed", None));
    };

    sqlx::query!(
        "INSERT INTO guild_kills (guild_id, killed_by) VALUES ($1, $2)",
        guild,
        cmd.user.id.get() as i64,
    )
    .execute(&handler.main_database)
    .await?;

    ctx.reply(
        cmd,
        Response::new().embed(
            CreateEmbed::new()
                .title("Guild killed")
                .description(format!("`{guild}` has been killed"))
                .color(0xff0000),
        ),
    )
    .await
}

async fn revive(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
    guild: i64,
) -> ResponseResult {
    if sqlx::query!(
        "SELECT guild_id FROM guild_kills WHERE guild_id = $1",
        guild
    )
    .fetch_optional(&handler.main_database)
    .await?
    .is_none()
    {
        return Err(ResponseError::Execution("Guild is already active", None));
    };

    sqlx::query!("DELETE FROM guild_kills WHERE guild_id = $1", guild)
        .execute(&handler.main_database)
        .await?;

    ctx.reply(
        cmd,
        Response::new().embed(
            CreateEmbed::new()
                .title("Guild revived")
                .description(format!("`{guild}` has been revived"))
                .color(0x00ff00),
        ),
    )
    .await
}

async fn status(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
    guild: i64,
) -> ResponseResult {
    let guild_flag = sqlx::query!(
        "SELECT killed_by FROM guild_kills WHERE guild_id = $1",
        guild
    )
    .fetch_optional(&handler.main_database)
    .await?;

    ctx.reply(
        cmd,
        Response::new().embed(
            CreateEmbed::new()
                .title("Guild status")
                .description(format!("Viewing `{guild}`"))
                .fields(vec![
                    (
                        "Active",
                        if guild_flag.is_none() {
                            "✅".to_string()
                        } else {
                            "❌".to_string()
                        },
                        true,
                    ),
                    (
                        "Killed by",
                        match &guild_flag {
                            Some(guild) => format!("<@{}>", guild.killed_by),
                            None => "N/A".to_string(),
                        },
                        true,
                    ),
                ])
                .color(if guild_flag.is_none() {
                    0x00ff00
                } else {
                    0xff0000
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

    let Some(guild_id) = options.get_string("guild").into_owned() else {
        return Err(ResponseError::Execution(
            "Invalid command option",
            Some("The command option you provided is invalid".to_string()),
        ));
    };

    let Ok(guild) = guild_id.parse() else {
        return Err(ResponseError::Execution(
            "Invalid ID",
            Some("The ID you provided was not a valid integer".to_string()),
        ));
    };

    match &option.value {
        CommandDataOptionValue::SubCommandGroup(group) => {
            match group.first().unwrap().name.as_str() {
                "kill" => kill(handler, ctx, cmd, guild).await,
                "revive" => revive(handler, ctx, cmd, guild).await,
                "status" => status(handler, ctx, cmd, guild).await,
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
