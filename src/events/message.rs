#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
use rand::Rng;
use serenity::all::{Context, Message as DiscordMessage, RoleId};
use tracing::error;

use crate::models::{
    handler::Handler,
    message::{Message, XPMessageQuery},
};

impl Handler {
    pub async fn on_message(&self, ctx: Context, message: DiscordMessage) {
        let guild_id = message.guild_id.unwrap().get() as i64;

        let attachment_url = message
            .attachments
            .first()
            .map(|attachment| attachment.url.to_string());

        let xp_configuration = match sqlx::query!(
            "SELECT * FROM xp_configuration WHERE guild_id = $1",
            guild_id
        )
        .fetch_optional(&self.main_database)
        .await
        {
            Ok(xp_configuration) => xp_configuration,
            Err(err) => {
                error!(
                    "Failed to fetch XP configuration. Failed with error: {:?}",
                    err
                );
                return;
            }
        };

        let message_exists = match XPMessageQuery::on_cooldown(
            &self.redis_database,
            guild_id,
            message.author.id.get() as i64,
        )
        .await
        {
            Ok(message_exists) => message_exists,
            Err(err) => {
                error!("Failed to check if XP message exists: {:?}", err);
                true
            }
        };

        let xp_duration = if message_exists {
            None
        } else {
            xp_configuration
                .as_ref()
                .map(|xp_configuration| i64::from(xp_configuration.message_cooldown))
        };

        if let Err(err) = Message::new(
            &self.redis_database,
            guild_id,
            message.author.id.get() as i64,
            message.channel_id.get() as i64,
            message.id.get() as i64,
            message.content.clone(),
            attachment_url,
            xp_duration,
        )
        .await
        {
            error!("Failed to create message: {:?}", err);
        };

        if message_exists {
            return;
        }

        let Some(xp_configuration) = xp_configuration else {
            return;
        };

        let base_xp = if let Some(set_xp_per_message) = xp_configuration.set_xp_per_message {
            set_xp_per_message
        } else {
            let min_xp = xp_configuration.min_xp_per_message.unwrap();
            let max_xp = xp_configuration.max_xp_per_message.unwrap();

            rand::thread_rng().gen_range(min_xp..max_xp)
        };

        let channel_multiplier = match sqlx::query!(
            "SELECT * FROM xp_channel_multipliers WHERE guild_id = $1 AND channel = $2",
            guild_id,
            message.channel_id.get() as i64
        )
        .fetch_optional(&self.main_database)
        .await
        {
            Ok(channel_multiplier) => match channel_multiplier {
                Some(channel_multiplier) => channel_multiplier.multiplier,
                None => 0.0,
            },
            Err(err) => {
                error!(
                    "Failed to fetch XP channel multiplier. Failed with error: {:?}",
                    err
                );
                return;
            }
        };

        let roles = message.member(&ctx).await.unwrap().roles;
        let mut role_multipliers = vec![];
        for role in roles {
            let role_multiplier = match sqlx::query!(
                "SELECT * FROM xp_role_multipliers WHERE guild_id = $1 AND role = $2",
                guild_id,
                role.get() as i64
            )
            .fetch_optional(&self.main_database)
            .await
            {
                Ok(role_multiplier) => match role_multiplier {
                    Some(role_multiplier) => role_multiplier.multiplier,
                    None => 0.0,
                },
                Err(err) => {
                    error!(
                        "Failed to fetch XP role multiplier. Failed with error: {:?}",
                        err
                    );
                    return;
                }
            };

            role_multipliers.push(role_multiplier);
        }

        let multiplier = if xp_configuration.stack_multipliers {
            let mut multiplier = 1.0;
            for role_multiplier in role_multipliers {
                multiplier += role_multiplier;
            }
            multiplier += channel_multiplier;
            multiplier.max(xp_configuration.multiplier_cap.unwrap_or(0.0))
        } else {
            let mut multiplier = channel_multiplier;
            for role_multiplier in role_multipliers {
                multiplier = role_multiplier.max(multiplier);
            }
            multiplier.max(xp_configuration.multiplier_cap.unwrap_or(0.0))
        };

        let user_xp = match sqlx::query!(
            "SELECT xp FROM user_xp WHERE guild_id = $1 AND user_id = $2",
            guild_id,
            message.author.id.get() as i64
        )
        .fetch_optional(&self.main_database)
        .await
        {
            Ok(user_xp) => match user_xp {
                Some(user_xp) => user_xp.xp,
                None => match sqlx::query!(
                    "INSERT INTO user_xp (guild_id, user_id, xp) VALUES ($1, $2, 0)",
                    guild_id,
                    message.author.id.get() as i64
                )
                .execute(&self.main_database)
                .await
                {
                    Ok(_) => 0,
                    Err(err) => {
                        error!("Failed to insert user XP. Failed with error: {:?}", err);
                        return;
                    }
                },
            },
            Err(err) => {
                error!("Failed to fetch user XP. Failed with error: {:?}", err);
                return;
            }
        };

        let new_xp = user_xp + i64::from(base_xp * multiplier as i32);
        if let Err(err) = sqlx::query!(
            "UPDATE user_xp SET xp = $1 WHERE guild_id = $2 AND user_id = $3",
            new_xp,
            guild_id,
            message.author.id.get() as i64
        )
        .execute(&self.main_database)
        .await
        {
            error!("Failed to update user XP. Failed with error: {:?}", err);
        }

        // THE QUADRATIC FORMULA?!
        let old_level =
            ((-25.0 + f64::sqrt((625 + (200 * user_xp)) as f64)) / 100.0).floor() as i64;
        let new_level = ((-25.0 + f64::sqrt((625 + (200 * new_xp)) as f64)) / 100.0).floor() as i64;

        if new_level != old_level {
            return;
        }

        let rewards = match sqlx::query!(
            "SELECT role FROM xp_rewards WHERE guild_id = $1 AND level = $2",
            guild_id,
            new_level
        )
        .fetch_all(&self.main_database)
        .await
        {
            Ok(rewards) => rewards,
            Err(err) => {
                error!("Failed to fetch XP rewards. Failed with error: {:?}", err);
                return;
            }
        }
        .iter()
        .map(|reward| RoleId::new(reward.role as u64))
        .collect::<Vec<_>>();

        if !rewards.is_empty() {
            if !xp_configuration.stack_rewards {
                // TODO: This only gets 1 role whereas there can be multiple rewards - fix this
                let _role = sqlx::query!("SELECT role FROM xp_rewards WHERE guild_id = $1 AND level < $2 ORDER BY level DESC LIMIT 1", guild_id, new_level)
                .fetch_one(&self.main_database)
                .await
                .unwrap().role;
            }

            if let Err(err) = message
                .member(&ctx)
                .await
                .unwrap()
                .add_roles(&ctx.http, &rewards)
                .await
            {
                error!("Failed to add roles to user. Failed with error: {:?}", err);
            };
        }

        let level_up_configuration = match sqlx::query!(
            "SELECT * FROM xp_level_up_messages WHERE guild_id = $1",
            guild_id
        )
        .fetch_one(&self.main_database)
        .await
        {
            Ok(level_up_configuration) => level_up_configuration,
            Err(err) => {
                error!(
                    "Failed to fetch XP level up configuration. Failed with error: {:?}",
                    err
                );
                return;
            }
        };

        if !level_up_configuration.enabled {
            // return;
        }

        // TODO:
        // 3h. If level up messages are enabled, check if level up messages are DM or channel messages
        // 3i. If level up messages are DM, send the message to the user
        // 3j. If level up messages are channel, check if it's a specific channel
        // 3l. If it's a specific channel, send the message to the channel
        // 3m. If it's not a specific channel, send the message in the current channel

        // let mut content = level_up_configuration.message.unwrap();
        // content = content.replace("{user.name}", &message.author.name);
        // content = content.replace("{user.mention}", &format!("<@{}>", &message.author.id));
        // content = content.replace("{user.level}", &new_level.to_string());
    }
}
