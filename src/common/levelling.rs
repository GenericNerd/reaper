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
        let guild_role_rewards = guild_rewards
            .iter()
            .map(|reward| RoleId::new(reward.role as u64))
            .collect::<Vec<_>>();

        if guild_rewards.is_empty() {
            return;
        }

        let user_rewards = if stack_rewards {
            guild_rewards
                .iter()
                .filter(|reward| reward.level <= new_level)
                .map(|reward| RoleId::new(reward.role as u64))
                .collect::<Vec<_>>()
        } else {
            match guild_rewards
                .iter()
                .filter(|reward| reward.level <= new_level)
                .max_by_key(|reward| reward.level)
            {
                Some(closest_level) => guild_rewards
                    .iter()
                    .filter(|reward| reward.level == closest_level.level)
                    .map(|reward| RoleId::new(reward.role as u64))
                    .collect::<Vec<_>>(),
                None => {
                    vec![]
                }
            }
        };

        if member.roles == user_rewards {
            return;
        }

        let roles_to_remove = member
            .roles
            .iter()
            .filter(|role| !user_rewards.contains(role))
            .filter(|role| guild_role_rewards.contains(role))
            .copied()
            .collect::<Vec<_>>();

        let roles_to_add = user_rewards
            .iter()
            .filter(|role| !member.roles.contains(role))
            .copied()
            .collect::<Vec<_>>();

        if !roles_to_remove.is_empty() {
            if let Err(err) = member
                .remove_roles(&ctx.http, roles_to_remove.as_slice())
                .await
            {
                error!(
                    "Failed to remove roles from user. Failed with error: {:?}",
                    err
                );
                return;
            }
        }

        if !roles_to_add.is_empty() {
            if let Err(err) = member.add_roles(&ctx.http, roles_to_add.as_slice()).await {
                error!("Failed to add roles to user. Failed with error: {:?}", err);
            }
        }
    }

    async fn trigger_level_up_message(
        &self,
        ctx: &Context,
        member: Member,
        summoning_channel: ChannelId,
        new_level: i64,
    ) {
        if new_level == 0 {
            return;
        }

        let guild_id = member.guild_id.get() as i64;

        let level_up_configuration = match sqlx::query!(
            "SELECT * FROM xp_level_up_messages WHERE guild_id = $1",
            guild_id
        )
        .fetch_optional(&self.main_database)
        .await
        {
            Ok(level_up_configuration) => match level_up_configuration {
                Some(level_up_configuration) => level_up_configuration,
                None => {
                    return;
                }
            },
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

        let Some(mut content) = level_up_configuration.message else {
            return;
        };
        content = content.replace("{user.name}", &member.user.name);
        content = content.replace("{user.mention}", &format!("<@{}>", &member.user.id));
        content = content.replace("{user.level}", &new_level.to_string());
        let level_up_message = CreateMessage::new().content(content);
        if level_up_configuration.dm_message {
            if let Err(err) = member.user.dm(&ctx.http, level_up_message).await {
                error!("Failed to send level up message to user: {:?}", err);
            }
        } else if let Some(channel) = level_up_configuration.channel {
            if let Err(err) = ChannelId::new(channel as u64)
                .send_message(&ctx.http, level_up_message)
                .await
            {
                error!("Failed to send level up message to channel: {:?}", err);
            }
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
