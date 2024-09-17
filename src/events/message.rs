use serenity::all::Message as DiscordMessage;
use tracing::error;

use crate::models::{handler::Handler, message::Message};

impl Handler {
    pub async fn on_message(&self, message: DiscordMessage) {
        let attachment_url = message
            .attachments
            .first()
            .map(|attachment| attachment.url.to_string());

        // TODO:
        // 1a. Get the guild configuration - is it enabled (not found = false)?
        // 1b. If guild is not enabled, don't care
        // 1c. If enabled, check if the message exists in the Redis database
        // 1d. If it does, don't care
        // 1e. If it doesn't, insert it to the Redis database
        // 2a. Is the guild using set or random XP?
        // 2b. If using set, skip to 5e
        // 2c. If using random XP, get the lower and upper XP limits - generate a random number between them
        // 2d. Check if this qualifies for role or channel multipliers
        // 2e. If it doesn't skip to 2i
        // 2f. If it does, check if the guild stacks multipliers or not
        // 2g. If it does, pick the highest multiplier
        // 2h. If it doesn't, stack the multipliers
        // 2i. Get the current XP of the user
        // 2j. If the XP is above the maximum level, don't care
        // 2k. Add the number of XP (+ multipliers) to the user's XP
        // 3a. If the user is levelling up, if they aren't, skip this step
        // 3b. Check if the guild has level up messages enabled
        // 3c. Check if the user's level meets a XP reward
        // 3d. If the user's level does not meet a reward, skip to 3l
        // 3e. If the user's level meets a reward, check if the guild stacks rewards or not
        // 3f. If it does, add the reward to the user's profile
        // 3g. If it doesn't, find the current reward, remove it and add the new reward
        // 3h. If level up messages are not enabled, don't care
        // 3i. If level up messages are enabled, check if level up messages are DM or channel messages
        // 3j. If level up messages are DM, send the message to the user
        // 3k. If level up messages are channel, check if it's a specific channel
        // 3l. Get the content of the level up message and replace each variable with the appropriate value
        // 3m. If it's a specific channel, send the message to the channel
        // 3n. If it's not a specific channel, send the message in the current channel

        if let Err(err) = Message::new(
            &self.redis_database,
            message.guild_id.unwrap().get() as i64,
            message.author.id.get() as i64,
            message.channel_id.get() as i64,
            message.id.get() as i64,
            message.content,
            attachment_url,
        )
        .await
        {
            error!("Failed to create message: {:?}", err);
        };
    }
}
