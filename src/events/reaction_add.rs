use serenity::{
    all::{ChannelId, Reaction},
    builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage},
    prelude::Context,
};
use tracing::error;

use crate::models::{
    boards::{BoardConfiguration, BoardEmote, BoardEntry, BoardIgnoredChannel},
    handler::Handler,
};

impl Handler {
    pub async fn on_reaction_add(&self, ctx: Context, reaction: Reaction) {
        let Some(guild_id) = reaction.guild_id else {
            return;
        };
        let guild_int = guild_id.get() as i64;
        let channel_int = reaction.channel_id.get() as i64;

        let board_configuration = match sqlx::query_as!(
            BoardConfiguration,
            "SELECT * FROM boards WHERE guild_id = $1 AND channel_id = $2",
            guild_int,
            channel_int
        )
        .fetch_optional(&self.main_database)
        .await
        {
            Ok(board_configuration) => match board_configuration {
                Some(board_configuration) => board_configuration,
                None => return,
            },
            Err(err) => {
                error!(
                    "Could not fetch board configuration. Failed with error: {:?}",
                    err
                );
                return;
            }
        };

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
            Ok(entry) => if entry.is_some() {
                return;
            },
            Err(err) => {
                error!(
                    "Could not check whether the message was already in this board. Failed with error: {:?}",
                    err
                );
                return;
            }
        };

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

        if emotes.contains(&reaction.emoji.to_string()) {
            return;
        }

        let Ok(message) = reaction.message(&ctx.http).await else {
            return;
        };

        match message
            .reaction_users(&ctx.http, reaction.emoji, None, None)
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
            .footer(CreateEmbedFooter::new(message.timestamp.to_string()));
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

        if let Err(err) = sqlx::query!(
            "INSERT INTO board_entries (channel_id, guild_id, message_id) VALUES ($1, $2, $3)",
            channel_int,
            guild_int,
            message.id.get() as i64
        )
        .execute(&self.main_database)
        .await
        {
            error!("Could not insert board entry. Failed with error: {:?}", err);
        }
    }
}
