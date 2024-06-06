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
    match sqlx::query!(
        "SELECT id FROM giveaways WHERE id = $1 AND guild_id = $2",
        giveaway.id,
        ctx.guild.id.get() as i64
    )
    .fetch_optional(&handler.main_database)
    .await
    {
        Ok(val) => {
            if val.is_none() {
                return Err(ResponseError::Execution(
                    "This giveaway could not be found",
                    Some("Please use the message ID for the giveaway ID".to_string()),
                ));
            }
        }
        Err(err) => {
            error!(
                "Could not get giveaway {} from database. Failed with error: {:?}",
                giveaway.id, err
            );
            return Err(ResponseError::Execution(
                "This giveaway could not be found",
                Some("Please use the message ID for the giveaway ID".to_string()),
            ));
        }
    }

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
            .collect::<Vec<_>>(),
        Err(err) => {
            error!(
                "Could not get entries for giveaway {}. Failed with error: {:?}",
                giveaway.id, err
            );
            return Err(ResponseError::Execution(
                "Could not get giveaway entries",
                Some("Please notify the developer of this issue".to_string()),
            ));
        }
    };

    let winners = if entries.len() > usize::try_from(giveaway.winners).unwrap() {
        entries
            .choose_multiple(
                &mut rand::thread_rng(),
                usize::try_from(giveaway.winners).unwrap(),
            )
            .map(|entry| format!("<@{entry}>"))
            .collect::<Vec<_>>()
    } else {
        entries
            .iter()
            .map(|entry| format!("<@{entry}>"))
            .collect::<Vec<_>>()
    };

    if let Err(err) = message
        .edit(
            &ctx.ctx.http,
            EditMessage::new()
                .embed(
                    CreateEmbed::new()
                        .title(format!("{} giveaway", giveaway.prize))
                        .description(if winners.is_empty() {
                            "No one won the giveaway.".to_string()
                        } else {
                            "The giveaway is now over, congratulations to the winners!".to_string()
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
        return Err(ResponseError::Serenity(err));
    }

    if !winners.is_empty() {
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
            return Err(ResponseError::Serenity(err));
        }
    }

    if let Err(err) = sqlx::query!("DELETE FROM giveaways WHERE id = $1", giveaway.id)
        .execute(&handler.main_database)
        .await
    {
        error!(
            "Could not delete giveaway {} from database. Failed with error: {:?}",
            giveaway.id, err
        );
        return Err(ResponseError::Execution(
            "Could not delete giveaway from database",
            Some("Please notify the developer of this issue".to_string()),
        ));
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

    let Some(id_string) = options.get_string("id").into_owned() else {
        return Err(ResponseError::Execution(
            "Could not get giveaway ID",
            Some("Please notify the developer of this issue".to_string()),
        ));
    };

    let Ok(id) = id_string.parse::<i64>() else {
        return Err(ResponseError::Execution(
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
            return Err(ResponseError::Execution(
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
        return Err(ResponseError::Execution(
            "Could not get giveaway message",
            Some("Please notify the developer of this issue".to_string()),
        ));
    };

    match end_giveaway(handler, ctx, &giveaway, &mut message).await {
        Ok(()) => {
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
