use serenity::{
    all::{ChannelId, CommandInteraction, MessageId},
    builder::CreateEmbed,
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

pub async fn delete(
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
        "SELECT * FROM giveaways WHERE id = $1 AND guild_id = $2",
        id,
        ctx.guild.id.get() as i64
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

    let Ok(message) = ctx
        .ctx
        .http
        .get_message(
            ChannelId::new(giveaway.channel_id as u64),
            MessageId::new(giveaway.id as u64),
        )
        .await
    else {
        return Err(ResponseError::Execution(
            "Could not get giveaway message",
            Some("Please notify the developer of this issue".to_string()),
        ));
    };

    if let Err(err) = message.delete(&ctx.ctx).await {
        error!(
            "Could not delete giveaway message for giveaway {}. Failed with error: {:?}",
            giveaway.id, err
        );
        return Err(ResponseError::Execution(
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
        return Err(ResponseError::Execution(
            "Could not delete giveaway from database",
            Some("Please notify the developer of this issue".to_string()),
        ));
    }

    ctx.reply(
        cmd,
        Response::new()
            .embed(CreateEmbed::new().title("Giveaway deleted"))
            .ephemeral(true),
    )
    .await
}
