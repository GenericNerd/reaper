use std::time::Duration;

use serenity::{
    all::{ButtonStyle, ChannelId, MessageId, RoleId},
    builder::{CreateActionRow, CreateButton, CreateEmbed, EditMessage},
    futures::StreamExt,
};
use tracing::error;

use crate::models::{
    command::{CommandContext, InteractionContext, InteractionContextReply},
    giveaway::Giveaway,
    handler::Handler,
    response::{Response, ResponseError},
};

use super::end::end_giveaway;

fn generate_embed(giveaway: &Giveaway, entry_count: i64) -> CreateEmbed {
    CreateEmbed::new()
        .title(format!("{} giveaway", giveaway.prize))
        .description(match &giveaway.description {
            Some(description) => format!(
                "{description}\n\nWinners: {}\nEntries: {}\n\nGiveaway ends at <t:{}:F>",
                giveaway.winners,
                entry_count + 1,
                giveaway.duration.unix_timestamp()
            ),
            None => format!(
                "Winners: {}\nEntries: {}\n\nGiveaway ends at <t:{}:F>",
                giveaway.winners,
                entry_count + 1,
                giveaway.duration.unix_timestamp()
            ),
        })
        .color(0xfdca4c)
}

pub async fn new_giveaway_entry_handler(handler: Handler, ctx: CommandContext, giveaway: Giveaway) {
    let message_id = MessageId::new(giveaway.id as u64);
    let channel_id = ChannelId::new(giveaway.channel_id as u64);

    let Ok(mut message) = ctx.ctx.http.get_message(channel_id, message_id).await else {
        error!("Failed to get message for giveaway {}", giveaway.id);
        return;
    };

    let mut interaction_stream = message
        .await_component_interactions(&ctx.ctx)
        .timeout(Duration::new(
            (giveaway.duration.unix_timestamp() - time::OffsetDateTime::now_utc().unix_timestamp())
                as u64,
            0,
        ))
        .stream();

    // This will run while the giveaway is still active
    while let Some(interaction) = interaction_stream.next().await {
        let interaction_context = InteractionContext::new(ctx.ctx.clone(), &interaction);

        if let Some(restriction) = giveaway.role_restriction {
            let role = RoleId::new(restriction as u64);
            if !interaction.member.unwrap().roles.contains(&role) {
                if let Err(err) = interaction_context
                    .error_message(ResponseError::ExecutionError(
                        "You do not have permission to enter this giveaway",
                        None,
                    ))
                    .await
                {
                    error!(
                        "Could not notify user of failed giveaway entry. Failed with error: {:?}",
                        err
                    );
                }
                continue;
            }
        }

        match sqlx::query!(
            "SELECT id FROM giveaway_entry WHERE id = $1 AND user_id = $2",
            giveaway.id,
            interaction_context.interaction.user.id.get() as i64
        )
        .fetch_one(&handler.main_database)
        .await
        {
            Ok(_) => {
                if let Err(err) = interaction_context
                    .error_message(ResponseError::ExecutionError(
                        "You've already entered this giveaway",
                        None,
                    ))
                    .await
                {
                    error!(
                        "Could not notify user of failed giveaway entry. Failed with error: {:?}",
                        err
                    );
                }
                continue;
            }
            Err(err) => {
                error!("Could not check if user has already entered giveaway {}. Failed with error: {:?}", giveaway.id, err);
            }
        };

        let entry_count = match sqlx::query!(
            "SELECT COUNT(*) FROM giveaway_entry WHERE id=$1",
            giveaway.id,
        )
        .fetch_one(&handler.main_database)
        .await
        {
            Ok(entry_record) => entry_record.count.unwrap(),
            Err(err) => {
                error!(
                    "Could not get entry count for giveaway {}. Failed with error: {:?}",
                    giveaway.id, err
                );
                if let Err(err) = interaction_context
                    .error_message(ResponseError::ExecutionError(
                        "Entry count could not be obtained",
                        Some("Please contact the developer for assistance.".to_string()),
                    ))
                    .await
                {
                    error!(
                        "Could not notify user of failed giveaway entry. Failed with error: {:?}",
                        err
                    );
                }
                return;
            }
        };

        match sqlx::query!(
            "INSERT INTO giveaway_entry (id, user_id) VALUES ($1, $2)",
            giveaway.id,
            interaction_context.interaction.user.id.get() as i64
        )
        .execute(&handler.main_database)
        .await
        {
            Ok(_) => {
                if let Err(err) = interaction_context
                    .reply(
                        Response::new()
                            .embed(
                                CreateEmbed::new()
                                    .title("You're in the running!")
                                    .description("You've entered this giveaway. Good luck!")
                                    .color(0x00ff00),
                            )
                            .ephemeral(true),
                    )
                    .await
                {
                    error!(
                        "Could not acknowledge giveaway entry. Failed with error: {:?}",
                        err
                    );
                }
            }
            Err(err) => {
                error!(
                    "Could not insert giveaway entry for giveaway {}. Failed with error: {:?}",
                    giveaway.id, err
                );

                if let Err(err) = interaction_context
                    .error_message(ResponseError::ExecutionError(
                        "Failed to enter giveaway",
                        Some(
                            "We failed to enter you into the giveaway, please try again later."
                                .to_string(),
                        ),
                    ))
                    .await
                {
                    error!(
                        "Could not notify user of failed giveaway entry. Failed with error: {:?}",
                        err
                    );
                }
                continue;
            }
        }

        if let Err(err) = message
            .edit(
                &ctx.ctx.http,
                EditMessage::new()
                    .embed(generate_embed(&giveaway, entry_count))
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                        "enter",
                    )
                    .label("Enter")
                    .style(ButtonStyle::Primary)])]),
            )
            .await
        {
            error!(
                "Could not update giveaway message for giveaway {}. Failed with error: {:?}",
                giveaway.id, err
            );
            return;
        }
    }

    if let Ok(giveaway) = sqlx::query!("SELECT id FROM giveaways WHERE id = $1", giveaway.id)
        .fetch_optional(&handler.main_database)
        .await
    {
        if giveaway.is_none() {
            return;
        }
    }

    if let Err(err) = end_giveaway(&handler, &ctx, &giveaway, &mut message).await {
        error!(
            "Could not end giveaway {}. Failed with error: {:?}",
            giveaway.id, err
        );
    };
}
