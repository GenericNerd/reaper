use rand::prelude::SliceRandom;
use serenity::{
    all::{ChannelId, CommandInteraction, Message, MessageId},
    builder::{CreateEmbed, EditMessage},
};
use tracing::error;

use crate::{
    common::options::Options,
    models::{
        command::{CommandContext, CommandContextReply},
        giveaway::{DatabaseGiveaway, Giveaway},
        handler::Handler,
        response::{Response, ResponseError, ResponseResult},
    },
};

pub async fn end_giveaway(
    handler: &Handler,
    ctx: &CommandContext,
    giveaway: &Giveaway,
    message: &mut Message,
) -> ResponseResult {
    let entries = match sqlx::query!(
        "SELECT user_id FROM giveaway_entry WHERE id = $1",
        giveaway.id
    )
    .fetch_all(&handler.main_database)
    .await
    {
        Ok(entries) => entries
            .iter()
            .map(|entry| entry.user_id)
            .collect::<Vec<i64>>(),
        Err(err) => {
            error!(
                "Could not get entries for giveaway {}. Failed with error: {:?}",
                giveaway.id, err
            );
            return Err(ResponseError::ExecutionError(
                "Could not get giveaway entries",
                Some("Please notify the developer of this issue".to_string()),
            ));
        }
    };

    let winners = if entries.len() > giveaway.winners as usize {
        entries
            .choose_multiple(&mut rand::thread_rng(), giveaway.winners as usize)
            .map(|entry| format!("<@{entry}>"))
            .collect::<Vec<String>>()
    } else {
        entries
            .iter()
            .map(|entry| format!("<@{entry}>"))
            .collect::<Vec<String>>()
    };

    if let Err(err) = message
        .edit(
            &ctx.ctx.http,
            EditMessage::new()
                .embed(
                    CreateEmbed::new()
                        .title(format!("{} giveaway", giveaway.prize))
                        .description(if winners.is_empty() {
                            format!(
                                "Congratulations to {} for winning the giveaway!",
                                winners.join(", ")
                            )
                        } else {
                            "No one won the giveaway.".to_string()
                        })
                        .color(0x4752c4),
                )
                .components(vec![]),
        )
        .await
    {
        error!(
            "Could not update giveaway message to end giveaway {}. Failed with error: {:?}",
            giveaway.id, err
        );
        return Err(ResponseError::SerenityError(err));
    }

    if winners.is_empty() {
        if let Err(err) = message
            .reply(
                &ctx.ctx.http,
                format!(
                    "Congratulations to {} for winning the giveaway!",
                    winners.join(", ")
                ),
            )
            .await
        {
            error!(
                "Could not send giveaway winner message for giveaway {}. Failed with error: {:?}",
                giveaway.id, err
            );
            return Err(ResponseError::SerenityError(err));
        }
    }

    Ok(())
}

pub async fn end(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
) -> ResponseResult {
    let options = Options {
        options: cmd.data.options(),
    };

    let Some(id) = options.get_integer("id") else {
        return Err(ResponseError::ExecutionError(
            "Could not get giveaway ID",
            Some("Please notify the developer of this issue".to_string()),
        ));
    };

    let giveaway = match sqlx::query_as!(
        DatabaseGiveaway,
        "SELECT * FROM giveaways WHERE id = $1",
        id
    )
    .fetch_one(&handler.main_database)
    .await
    {
        Ok(giveaway) => Giveaway::from(giveaway),
        Err(err) => {
            error!(
                "Could not get giveaway {} from database. Failed with error: {:?}",
                id, err
            );
            return Err(ResponseError::ExecutionError(
                "This giveaway could not be found",
                Some("Please use the message ID for the giveaway ID".to_string()),
            ));
        }
    };

    let Ok(mut message) = ctx
        .ctx
        .http
        .get_message(
            ChannelId::new(giveaway.channel_id as u64),
            MessageId::new(giveaway.id as u64),
        )
        .await
    else {
        error!(
            "Could not get giveaway message for giveaway {}",
            giveaway.id
        );
        return Err(ResponseError::ExecutionError(
            "Could not get giveaway message",
            Some("Please notify the developer of this issue".to_string()),
        ));
    };

    match end_giveaway(handler, ctx, &giveaway, &mut message).await {
        Ok(_) => {
            ctx.reply(
                cmd,
                Response::new()
                    .embed(CreateEmbed::new().title("Successfully ended giveaway"))
                    .ephemeral(true),
            )
            .await
        }
        Err(err) => Err(err),
    }
}
