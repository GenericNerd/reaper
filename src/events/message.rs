use rand::Rng;
use serenity::all::{Context, Message as DiscordMessage, RoleId};
use tracing::error;

use crate::models::{
    handler::Handler,
    message::{Message, XPMessageQuery},
};

async fn calculate_multiplier(
    handler: &Handler,
    guild_id: i64,
    message: &DiscordMessage,
    user_roles: Vec<RoleId>,
    stack_multipliers: bool,
    multiplier_cap: Option<f32>,
) -> f32 {
    let channel_multiplier = match sqlx::query!(
        "SELECT * FROM xp_channel_multipliers WHERE guild_id = $1 AND channel = $2",
        guild_id,
        message.channel_id.get() as i64
    )
    .fetch_optional(&handler.main_database)
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
            0.0
        }
    };

    let mut role_multipliers = vec![];
    for role in user_roles {
        let role_multiplier = match sqlx::query!(
            "SELECT * FROM xp_role_multipliers WHERE guild_id = $1 AND role = $2",
            guild_id,
            role.get() as i64
        )
        .fetch_optional(&handler.main_database)
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
                0.0
            }
        };

        role_multipliers.push(role_multiplier);
    }

    if stack_multipliers {
        let mut multiplier = 1.0;
        for role_multiplier in role_multipliers {
            multiplier += role_multiplier;
        }
        multiplier += channel_multiplier;
        multiplier.max(multiplier_cap.unwrap_or(0.0))
    } else {
        let mut multiplier = channel_multiplier;
        for role_multiplier in role_multipliers {
            multiplier = role_multiplier.max(multiplier);
        }
        multiplier.max(multiplier_cap.unwrap_or(0.0))
    }
}

impl Handler {
    pub async fn on_message(&self, ctx: Context, message: DiscordMessage) {
        let guild_id = match message.guild_id {
            Some(guild_id) => guild_id.get() as i64,
            None => return,
        };
        let member = match message.member(&ctx.http).await {
            Ok(member) => member,
            Err(err) => {
                error!("Failed to fetch member. Failed with error: {:?}", err);
                return;
            }
        };

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

        let roles = message.member(&ctx).await.unwrap().roles;
        let blacklisted_roles = match sqlx::query!(
            "SELECT role FROM xp_role_blacklists WHERE guild_id = $1",
            guild_id
        )
        .fetch_all(&self.main_database)
        .await
        {
            Ok(blacklisted_roles) => blacklisted_roles,
            Err(err) => {
                error!(
                    "Failed to fetch XP role blacklists. Failed with error: {:?}",
                    err
                );
                return;
            }
        }
        .iter()
        .map(|blacklisted_role| RoleId::new(blacklisted_role.role as u64))
        .collect::<Vec<_>>();

        for blacklisted_role in blacklisted_roles {
            if roles.contains(&blacklisted_role) {
                return;
            }
        }

        if match sqlx::query!(
            "SELECT channel FROM xp_channel_blacklists WHERE guild_id = $1 AND channel = $2",
            guild_id,
            message.channel_id.get() as i64
        )
        .fetch_optional(&self.main_database)
        .await
        {
            Ok(blacklisted_channels) => blacklisted_channels,
            Err(err) => {
                error!(
                    "Failed to fetch XP channel blacklists. Failed with error: {:?}",
                    err
                );
                return;
            }
        }
        .is_some()
        {
            return;
        }

        let base_xp = if let Some(set_xp_per_message) = xp_configuration.set_xp_per_message {
            set_xp_per_message
        } else {
            let min_xp = xp_configuration.min_xp_per_message.unwrap();
            let max_xp = xp_configuration.max_xp_per_message.unwrap();

            rand::thread_rng().gen_range(min_xp..max_xp)
        };

        let multiplier = calculate_multiplier(
            self,
            guild_id,
            &message,
            roles,
            xp_configuration.stack_multipliers,
            xp_configuration.multiplier_cap,
        )
        .await;

        let user_xp = match sqlx::query!(
            "SELECT xp FROM user_xp WHERE guild_id = $1 AND user_id = $2",
            guild_id,
            message.author.id.get() as i64
        )
        .fetch_optional(&self.main_database)
        .await
        {
            Ok(user_xp) => {
                if let Some(user_xp) = user_xp {
                    user_xp.xp
                } else {
                    // This exception is written in for Troll. If the user doesn't have our XP,
                    // we will get the closest level from the role reward they have
                    let mut inserted = None;
                    if guild_id == 690072854582264086 {
                        let guild_rewards = match sqlx::query!(
                            "SELECT level, role FROM xp_rewards WHERE guild_id = $1",
                            guild_id
                        )
                        .fetch_all(&self.main_database)
                        .await
                        {
                            Ok(guild_rewards) => guild_rewards,
                            Err(err) => {
                                error!("Failed to fetch XP rewards. Failed with error: {:?}", err);
                                return;
                            }
                        };

                        // Find intersection between user roles and guild rewards
                        let user_role = guild_rewards
                            .iter()
                            .filter(|reward| {
                                member.roles.contains(&RoleId::new(reward.role as u64))
                            })
                            .collect::<Vec<_>>();
                        let user_role = user_role.first();

                        if let Some(user_role) = user_role {
                            let user_xp =
                                (50 * (user_role.level * user_role.level)) + (25 * user_role.level);
                            match sqlx::query!(
                                "INSERT INTO user_xp (guild_id, user_id, xp) VALUES ($1, $2, $3)",
                                guild_id,
                                message.author.id.get() as i64,
                                user_xp
                            )
                            .execute(&self.main_database)
                            .await
                            {
                                Ok(_) => inserted = Some(user_xp),
                                Err(err) => {
                                    error!(
                                        "Failed to insert user XP. Failed with error: {:?}",
                                        err
                                    );
                                    return;
                                }
                            }
                        }
                    }
                    if let Some(xp_value) = inserted {
                        xp_value
                    } else {
                        match sqlx::query!(
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
                        }
                    }
                }
            }
            Err(err) => {
                error!("Failed to fetch user XP. Failed with error: {:?}", err);
                return;
            }
        };

        let new_xp = user_xp + i64::from(base_xp * multiplier as i32);

        if let Some(max_level) = xp_configuration.max_level {
            if new_xp > i64::from((50 * (max_level * max_level)) + (25 * max_level)) {
                return;
            }
        }

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

        if new_level == 0 || new_level == old_level {
            return;
        }

        self.user_level_up(&ctx, member, message.channel_id, new_level)
            .await;
    }
}
