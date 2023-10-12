use serenity::{
    all::{ChannelId, Reaction},
    builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage},
    prelude::Context,
};
use tracing::{debug, error};

use crate::models::{
    boards::{BoardConfiguration, BoardEmote, BoardEntry, BoardIgnoredChannel},
    handler::Handler,
};

impl Handler {
    pub async fn on_reaction_add(&self, ctx: Context, reaction: Reaction) {
        let start = std::time::Instant::now();

        let Some(guild_id) = reaction.guild_id else {
            return;
        };
        let guild_int = guild_id.get() as i64;
        let channel_int = reaction.channel_id.get() as i64;

        let board_configurations = match sqlx::query_as!(
            BoardConfiguration,
            "SELECT * FROM boards WHERE guild_id = $1",
            guild_int
        )
        .fetch_all(&self.main_database)
        .await
        {
            Ok(board_configurations) => board_configurations,
            Err(err) => {
                error!(
                    "Could not fetch board configuration. Failed with error: {:?}",
                    err
                );
                return;
            }
        };

        debug!("Board configurations fetched in {:?}", start.elapsed());

        for board_configuration in board_configurations {
            let ignored_channels = match sqlx::query_as!(
                BoardIgnoredChannel,
                "SELECT * FROM board_ignored_channels WHERE guild_id = $1 AND channel_id = $2",
                guild_int,
                board_configuration.channel_id
            )
            .fetch_all(&self.main_database)
            .await
            {
                Ok(channel_ignores) => {
                    let mut ignored_channels = vec![];
                    for channel_ignore in channel_ignores {
                        ignored_channels.push(channel_ignore.channel_id);
                    }
                    ignored_channels
                }
                Err(err) => {
                    error!(
                        "Could not fetch board ignored channels. Failed with error: {:?}",
                        err
                    );
                    return;
                }
            };

            if ignored_channels.contains(&channel_int) {
                return;
            }

            debug!(
                "Checked whether message was in an ignored channel in {:?}",
                start.elapsed()
            );

            match sqlx::query_as!(
                BoardEntry,
                "SELECT * FROM board_entries WHERE guild_id = $1 AND channel_id = $2 AND message_id = $3",
                guild_int,
                board_configuration.channel_id,
                reaction.message_id.get() as i64,
            )
            .fetch_optional(&self.main_database)
            .await
            {
                Ok(entry) => {
                    if entry.is_some() {
                        return;    
                    }
                },
                Err(err) => {
                    error!(
                        "Could not check whether the message was already in this board. Failed with error: {:?}",
                        err
                    );
                    return;
                }
            };

            debug!(
                "Checked whether message was already on the board in {:?}",
                start.elapsed()
            );

            let emotes = match sqlx::query_as!(
                BoardEmote,
                "SELECT * FROM board_emotes WHERE guild_id = $1 AND channel_id = $2",
                guild_int,
                board_configuration.channel_id
            )
            .fetch_all(&self.main_database)
            .await
            {
                Ok(emotes) => emotes
                    .iter()
                    .map(|emote| emote.emote.to_string())
                    .collect::<Vec<String>>(),
                Err(err) => {
                    error!("Could not fetch board emotes. Failed with error: {:?}", err);
                    return;
                }
            };

            let message_reaction = &reaction.clone();

            if !emotes.contains(&message_reaction.emoji.to_string()) {
                return;
            }

            debug!("Checked emoji validity in {:?}", start.elapsed());

            let Ok(message) = message_reaction.message(&ctx.http).await else {
                return;
            };

            match message
                .reaction_users(&ctx.http, message_reaction.emoji.clone(), None, None)
                .await
            {
                Ok(reaction_users) => {
                    let reaction_count = if reaction_users.contains(&message.author)
                        && board_configuration.ignore_self_reacts
                    {
                        reaction_users.len() - 1
                    } else {
                        reaction_users.len()
                    };

                    if board_configuration.emote_quota > i32::try_from(reaction_count).unwrap() {
                        return;
                    }
                }
                Err(err) => {
                    error!(
                        "Could not fetch reaction users. Failed with error: {:?}",
                        err
                    );
                    return;
                }
            };

            debug!("Got reaction count in {:?}", start.elapsed());

            // This reaction now needs to go on a board

            let attachment = message
                .attachments
                .get(0)
                .map(|attachment| attachment.url.to_string());

            let mut embed = CreateEmbed::new()
                .author(
                    CreateEmbedAuthor::new(&message.author.name)
                        .icon_url(message.author.avatar_url().unwrap_or_default()),
                )
                .description(&message.content)
                .footer(CreateEmbedFooter::new(message.timestamp.to_string()))
                .color(0xffac33);
            if let Some(url) = attachment {
                embed = embed.image(url);
            }

            if let Err(err) = ChannelId::new(board_configuration.channel_id as u64)
                .send_message(
                    &ctx.http,
                    CreateMessage::new().content(message.link()).embed(embed),
                )
                .await
            {
                error!(
                    "Could not send message to board channel. Failed with error: {:?}",
                    err
                );
                return;
            }

            debug!("Posted message on board in {:?}", start.elapsed());

            if let Err(err) = sqlx::query!(
                "INSERT INTO board_entries (guild_id, channel_id, message_id) VALUES ($1, $2, $3)",
                guild_int,
                board_configuration.channel_id,
                message.id.get() as i64
            )
            .execute(&self.main_database)
            .await
            {
                error!("Could not insert board entry. Failed with error: {:?}", err);
            }
        }
        debug!("Finished board placements in {:?}", start.elapsed());
    }
}
