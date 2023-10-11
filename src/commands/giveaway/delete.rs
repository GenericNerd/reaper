use serenity::{
    all::{ChannelId, CommandInteraction, MessageId},
    builder::CreateInteractionResponse,
};
use tracing::error;

use crate::{
    common::options::Options,
    models::{
        command::CommandContext,
        giveaway::{DatabaseGiveaway, Giveaway},
        handler::Handler,
        response::{ResponseError, ResponseResult},
    },
};

pub async fn delete(
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

    let Ok(message) = ctx
        .ctx
        .http
        .get_message(
            ChannelId::new(giveaway.channel_id as u64),
            MessageId::new(giveaway.id as u64),
        )
        .await
    else {
        return Err(ResponseError::ExecutionError(
            "Could not get giveaway message",
            Some("Please notify the developer of this issue".to_string()),
        ));
    };

    if let Err(err) = message.delete(&ctx.ctx).await {
        error!(
            "Could not delete giveaway message for giveaway {}. Failed with error: {:?}",
            giveaway.id, err
        );
        return Err(ResponseError::ExecutionError(
            "Could not delete giveaway message",
            Some("Please notify the developer of this issue".to_string()),
        ));
    };

    if let Err(err) = sqlx::query!("DELETE FROM giveaways WHERE id = $1", giveaway.id)
        .execute(&handler.main_database)
        .await
    {
        error!(
            "Could not delete giveaway {} from database. Failed with error: {:?}",
            giveaway.id, err
        );
        return Err(ResponseError::ExecutionError(
            "Could not delete giveaway from database",
            Some("Please notify the developer of this issue".to_string()),
        ));
    }

    if let Err(err) = cmd
        .create_response(&ctx.ctx.http, CreateInteractionResponse::Acknowledge)
        .await
    {
        error!(
            "Could not acknowledge command interaction. Failed with error: {:?}",
            err
        );
    };

    Ok(())
}
