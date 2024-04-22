use serenity::{
    all::{ButtonStyle, CommandInteraction},
    builder::{CreateActionRow, CreateButton, CreateEmbed},
};
use std::time::Instant;
use tracing::{debug, error};

use crate::{
    commands::giveaway::interaction::new_giveaway_entry_handler,
    common::{duration::Duration, options::Options},
    models::{
        command::{CommandContext, CommandContextReply},
        giveaway::Giveaway,
        handler::Handler,
        response::{Response, ResponseError, ResponseResult},
    },
};

pub async fn new(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
) -> ResponseResult {
    let start = Instant::now();

    let options = Options {
        options: cmd.data.options(),
    };

    let winners = options.get_integer("winners").unwrap_or(1);

    if winners < 1 {
        return Err(ResponseError::Execution(
            "Not enough winners",
            Some("You must have at least one winner".to_string()),
        ));
    } else if winners > 50 {
        return Err(ResponseError::Execution(
            "Too many winners",
            Some("You can only have a maximum of 50 winners".to_string()),
        ));
    }

    let Some(prize) = options.get_string("prize").into_owned() else {
        return Err(ResponseError::Execution(
            "No prize provided",
            Some("Please provide a prize to giveaway".to_string()),
        ));
    };

    let description = options.get_string("description").into_owned();

    let Some(duration) = options
        .get_string("duration")
        .into_owned()
        .map(|duration| Duration::new(duration.as_str()))
    else {
        return Err(ResponseError::Execution(
            "No duration provided",
            Some("Please provide a duration for the giveaway".to_string()),
        ));
    };

    let role = options.get_role("role").into_owned();

    debug!(
        "Got all required information to create a giveaway in {:?}",
        start.elapsed()
    );

    let message = ctx
        .reply_get_message(
            cmd,
            Response::new()
                .embed(
                    CreateEmbed::new()
                        .title(format!("{prize} giveaway"))
                        .description(match &description {
                            Some(description) => format!(
                                "{description}\n\nWinners: {winners}\nEntries: 0\n\nGiveaway ends at <t:{}:F>",
                                duration.to_timestamp().unwrap().unix_timestamp()
                            ),
                            None => format!(
                                "Winners: {winners}\nEntries: 0\n\nGiveaway ends at <t:{}:F>",
                                duration.to_timestamp().unwrap().unix_timestamp()
                            ),
                        })
                        .color(0xfdca4c),
                )
                .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                    "enter",
                )
                .label("Enter")
                .style(ButtonStyle::Primary)])]),
        )
        .await?;

    let primative_duration = time::PrimitiveDateTime::new(
        duration.to_timestamp().unwrap().date(),
        duration.to_timestamp().unwrap().time(),
    );

    let giveaway = Giveaway {
        id: message.id.get() as i64,
        channel_id: message.channel_id.get() as i64,
        prize,
        description,
        winners: i32::try_from(winners).unwrap(),
        duration: duration.to_timestamp().unwrap(),
        role_restriction: role.map(|role| role.id.get() as i64),
    };

    if let Err(err) = sqlx::query!(
        "INSERT INTO giveaways (id, channel_id, prize, description, winners, duration, role_restriction) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        giveaway.id,
        giveaway.channel_id,
        giveaway.prize,
        giveaway.description,
        giveaway.winners,
        primative_duration,
        giveaway.role_restriction
    )
    .execute(&handler.main_database)
    .await
    {
        error!("Failed to insert giveaway into database: {:?}", err);
        error!("Giveaway {:?} will not persist on restart", giveaway.id);
    };

    tokio::spawn(new_giveaway_entry_handler(
        handler.clone(),
        ctx.clone(),
        giveaway,
    ));
    Ok(())
}
