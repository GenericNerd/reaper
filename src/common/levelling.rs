use serenity::all::{ChannelId, Context, CreateMessage, Member, RoleId};
use tracing::error;

use crate::models::handler::Handler;

impl Handler {
    async fn trigger_rewards(&self, ctx: &Context, member: Member, new_level: i64) {
        let guild_id = member.guild_id.get() as i64;

        let stack_rewards = match sqlx::query!(
            "SELECT stack_rewards FROM xp_configuration WHERE guild_id = $1",
            guild_id
        )
        .fetch_optional(&self.main_database)
        .await
        {
            Ok(stack_rewards) => match stack_rewards {
                Some(stack_rewards) => stack_rewards.stack_rewards,
                None => return,
            },
            Err(err) => {
                error!(
                    "Failed to fetch XP configuration. Failed with error: {:?}",
                    err
                );
                return;
            }
        };

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

        if rewards.is_empty() {
            return;
        }

        if !stack_rewards {
            let roles = match sqlx::query!("WITH closest_level AS (SELECT MAX(level) AS max_level FROM xp_rewards WHERE guild_id = $1 AND level < $2) SELECT role FROM xp_rewards WHERE level = (SELECT max_level FROM closest_level) AND guild_id = $1", guild_id, new_level)
            .fetch_all(&self.main_database)
            .await {
                Ok(roles) => roles.iter().map(|role| RoleId::new(role.role as u64)).collect::<Vec<_>>(),
                Err(err) => {
                    error!("Failed to fetch XP rewards. Failed with error: {:?}", err);
                    return;
                }
            };

            if let Err(err) = member.remove_roles(&ctx.http, &roles).await {
                error!(
                    "Failed to remove roles from user. Failed with error: {:?}",
                    err
                );
            };
        }

        if let Err(err) = member.add_roles(&ctx.http, &rewards).await {
            error!("Failed to add roles to user. Failed with error: {:?}", err);
        };
    }

    async fn trigger_level_up_message(
        &self,
        ctx: &Context,
        member: Member,
        summoning_channel: ChannelId,
        new_level: i64,
    ) {
        let guild_id = member.guild_id.get() as i64;

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
            return;
        }

        let mut content = level_up_configuration.message.unwrap();
        content = content.replace("{user.name}", &member.user.name);
        content = content.replace("{user.mention}", &format!("<@{}>", &member.user.id));
        content = content.replace("{user.level}", &new_level.to_string());
        let level_up_message = CreateMessage::new().content(content);
        if level_up_configuration.dm_message {
            if let Err(err) = member.user.dm(&ctx.http, level_up_message).await {
                error!("Failed to send level up message to user: {:?}", err);
            };
        } else if let Some(channel) = level_up_configuration.channel {
            if let Err(err) = ChannelId::new(channel as u64)
                .send_message(&ctx.http, level_up_message)
                .await
            {
                error!("Failed to send level up message to channel: {:?}", err);
            };
        } else if let Err(err) = summoning_channel
            .send_message(&ctx.http, level_up_message)
            .await
        {
            error!("Failed to send level up message: {:?}", err);
        }
    }

    pub async fn user_level_up(
        &self,
        ctx: &Context,
        member: Member,
        summoning_channel: ChannelId,
        new_level: i64,
    ) {
        self.trigger_rewards(ctx, member.clone(), new_level).await;
        self.trigger_level_up_message(ctx, member, summoning_channel, new_level)
            .await;
    }
}
