use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, CreateEmbed, User,
};

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
    user: User,
) -> ResponseResult {
    if sqlx::query!(
        "SELECT user_id FROM user_kills WHERE user_id = $1",
        user.id.get() as i64
    )
    .fetch_optional(&handler.main_database)
    .await?
    .is_some()
    {
        return Err(ResponseError::Execution("User is already killed", None));
    };

    sqlx::query!(
        "INSERT INTO user_kills (user_id, killed_by) VALUES ($1, $2)",
        user.id.get() as i64,
        cmd.user.id.get() as i64,
    )
    .execute(&handler.main_database)
    .await?;

    ctx.reply(
        cmd,
        Response::new().embed(
            CreateEmbed::new()
                .title("User killed")
                .description(format!("<@{}> has been killed", user.id.get()))
                .color(0xff0000),
        ),
    )
    .await
}

async fn revive(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
    user: User,
) -> ResponseResult {
    if sqlx::query!(
        "SELECT user_id FROM user_kills WHERE user_id = $1",
        user.id.get() as i64
    )
    .fetch_optional(&handler.main_database)
    .await?
    .is_none()
    {
        return Err(ResponseError::Execution("User is already active", None));
    };

    sqlx::query!(
        "DELETE FROM user_kills WHERE user_id = $1",
        user.id.get() as i64,
    )
    .execute(&handler.main_database)
    .await?;

    ctx.reply(
        cmd,
        Response::new().embed(
            CreateEmbed::new()
                .title("User revived")
                .description(format!("<@{}> has been revived", user.id.get()))
                .color(0x00ff00),
        ),
    )
    .await
}

async fn status(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
    user: User,
) -> ResponseResult {
    let user_flag = sqlx::query!(
        "SELECT killed_by FROM user_kills WHERE user_id = $1",
        user.id.get() as i64
    )
    .fetch_optional(&handler.main_database)
    .await?;

    ctx.reply(
        cmd,
        Response::new().embed(
            CreateEmbed::new()
                .title("User status")
                .description(format!("Viewing <@{}>", user.id.get()))
                .fields(vec![
                    (
                        "Active",
                        if user_flag.is_none() {
                            "✅".to_string()
                        } else {
                            "❌".to_string()
                        },
                        true,
                    ),
                    (
                        "Killed by",
                        match &user_flag {
                            Some(user) => format!("<@{}>", user.killed_by),
                            None => "N/A".to_string(),
                        },
                        true,
                    ),
                ])
                .color(if user_flag.is_none() {
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

    let Some(user) = options.get_user("user").into_owned() else {
        return Err(ResponseError::Execution(
            "Invalid command option",
            Some("The command option you provided is invalid".to_string()),
        ));
    };

    match &option.value {
        CommandDataOptionValue::SubCommandGroup(group) => {
            match group.first().unwrap().name.as_str() {
                "kill" => kill(handler, ctx, cmd, user).await,
                "revive" => revive(handler, ctx, cmd, user).await,
                "status" => status(handler, ctx, cmd, user).await,
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
