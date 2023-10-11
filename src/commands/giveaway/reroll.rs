use rand::seq::SliceRandom;
use serenity::all::CommandInteraction;
use tracing::error;

use crate::{
    common::options::Options,
    models::{
        command::{CommandContext, CommandContextReply},
        handler::Handler,
        response::{Response, ResponseError, ResponseResult},
    },
};

pub async fn reroll(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
) -> ResponseResult {
    let options = Options {
        options: cmd.data.options(),
    };

    let Some(id_string) = options.get_string("id").into_owned() else {
        return Err(ResponseError::ExecutionError(
            "Could not get giveaway ID",
            Some("Please notify the developer of this issue".to_string()),
        ));
    };

    let Ok(id) = id_string.parse::<i64>() else {
        return Err(ResponseError::ExecutionError(
            "Could not get giveaway ID",
            Some("Please notify the developer of this issue".to_string()),
        ));
    };

    let winners = options.get_integer("winners").unwrap_or(1);

    if winners < 1 {
        return Err(ResponseError::ExecutionError(
            "Not enough winners",
            Some("You must have at least one winner".to_string()),
        ));
    } else if winners > 50 {
        return Err(ResponseError::ExecutionError(
            "Too many winners",
            Some("You can only have a maximum of 50 winners".to_string()),
        ));
    }

    if let Ok(row) = sqlx::query!("SELECT id FROM giveaways WHERE id = $1", id)
        .fetch_optional(&handler.main_database)
        .await
    {
        if row.is_some() {
            return Err(ResponseError::ExecutionError(
                "The giveaway has not finished",
                Some(
                    "Please end the giveaway or wait for it to finish before rerolling".to_string(),
                ),
            ));
        }
    }

    let entries = match sqlx::query!("SELECT user_id FROM giveaway_entry WHERE id = $1", id)
        .fetch_all(&handler.main_database)
        .await
    {
        Ok(entries) => entries
            .iter()
            .map(|entry| format!("<@{}>", entry.user_id))
            .collect::<Vec<String>>(),
        Err(err) => {
            error!(
                "Could not get giveaway entries for giveaway {} from database. Failed with error: {:?}",
                id, err
            );
            return Err(ResponseError::ExecutionError(
                "Could not get giveaway entries",
                Some("Please notify the developer of this issue".to_string()),
            ));
        }
    };

    let winners = if entries.len() > usize::try_from(winners).unwrap() {
        entries
            .choose_multiple(&mut rand::thread_rng(), usize::try_from(winners).unwrap())
            .map(|entry| entry.to_string())
            .collect::<Vec<String>>()
    } else {
        entries
    };

    if winners.is_empty() {
        return ctx
            .reply(
                cmd,
                Response::new()
                    .content("No one won the giveaway.".to_string())
                    .ephemeral(true),
            )
            .await;
    }

    ctx.reply(
        cmd,
        Response::new().content(format!(
            "Congratulations to {} for winning the giveaway!",
            winners.join(", ")
        )),
    )
    .await
}
