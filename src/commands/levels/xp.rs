use serenity::all::{
    ChannelId, CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption,
    CreateEmbed, CreateEmbedFooter, CreateMessage, RoleId,
};
use tracing::error;

use crate::{
    common::{
        logging::{get_log_channel, LogType},
        options::Options,
    },
    models::{
        command::{Command, CommandContext, CommandContextReply},
        config::LoggingConfig,
        handler::Handler,
        permissions::Permission,
        response::{Response, ResponseError, ResponseResult},
    },
};

pub struct XPCommand;

async fn add_xp(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
) -> ResponseResult {
    let options = Options {
        options: cmd.data.options(),
    };

    let Some(user) = options.get_user("user").into_owned() else {
        return Err(ResponseError::Execution(
            "No user provided",
            Some("Please provide a user to add XP to".to_string()),
        ));
    };

    let user_xp = match sqlx::query!(
        "SELECT xp FROM user_xp WHERE guild_id = $1 AND user_id = $2",
        ctx.guild.id.get() as i64,
        user.id.get() as i64
    )
    .fetch_optional(&handler.main_database)
    .await
    {
        Ok(user_xp) => {
            if let Some(user_xp) = user_xp {
                user_xp.xp
            } else {
                if let Err(err) = sqlx::query!(
                    "INSERT INTO user_xp (guild_id, user_id, xp) VALUES ($1, $2, 0)",
                    ctx.guild.id.get() as i64,
                    user.id.get() as i64
                )
                .execute(&handler.main_database)
                .await
                {
                    error!("Failed to insert user XP. Failed with error: {:?}", err);
                    return Err(ResponseError::Execution(
                        "Failed to insert user XP",
                        Some("Failed to insert user XP".to_string()),
                    ));
                }
                0
            }
        }
        Err(err) => {
            error!("Failed to fetch user XP. Failed with error: {:?}", err);
            return Err(ResponseError::Execution(
                "Failed to fetch user XP",
                Some("Failed to fetch user XP".to_string()),
            ));
        }
    };

    let Some(xp) = options.get_integer("xp") else {
        return Err(ResponseError::Execution(
            "No XP provided",
            Some("Please provide a XP amount to add".to_string()),
        ));
    };

    let old_level = ((-25.0 + f64::sqrt((625 + (200 * user_xp)) as f64)) / 100.0).floor() as i64;
    let new_level =
        ((-25.0 + f64::sqrt((625 + (200 * (user_xp + xp))) as f64)) / 100.0).floor() as i64;

    sqlx::query!(
        "UPDATE user_xp SET xp = xp + $1 WHERE guild_id = $2 AND user_id = $3",
        xp,
        ctx.guild.id.get() as i64,
        user.id.get() as i64
    )
    .execute(&handler.main_database)
    .await?;

    if new_level != old_level {
        let Ok(member) = ctx.ctx.http.get_member(ctx.guild.id, user.id).await else {
            return Err(ResponseError::Execution(
                "Failed to fetch member",
                Some("Failed to fetch member".to_string()),
            ));
        };

        handler
            .user_level_up(&ctx.ctx, member, cmd.channel_id, new_level)
            .await;
    }

    let logging_config = match sqlx::query_as!(
        LoggingConfig,
        "SELECT log_actions, log_messages, log_voice, log_channel, log_action_channel, log_message_channel, log_voice_channel FROM logging_configuration WHERE guild_id = $1",
        ctx.guild.id.get() as i64
    )
    .fetch_one(&handler.main_database)
    .await
    {
        Ok(logging_config) => logging_config,
        Err(err) => {
            error!(
                "Failed to fetch logging configuration. Failed with error: {:?}",
                err
            );
            return Err(ResponseError::Execution(
                "Failed to fetch logging configuration",
                Some("Failed to fetch logging configuration".to_string()),
            ));
        }
    };
    if let Some(channel) = get_log_channel(handler, &logging_config, &LogType::Action).await {
        if let Err(err) = ChannelId::new(channel as u64)
            .send_message(
                &ctx.ctx,
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .title("XP Added")
                        .description(format!(
                            "<@{}> gave <@{}> {xp} XP",
                            cmd.user.id.get(),
                            user.id.get()
                        ))
                        .footer(CreateEmbedFooter::new(format!(
                            "User {} gave user {} {xp} XP",
                            cmd.user.id.get(),
                            user.id.get()
                        )))
                        .color(0x00ff00),
                ),
            )
            .await
        {
            error!(
                "Failed to send XP added log message. Failed with error: {:?}",
                err
            );
        };
    }

    ctx.reply(
        cmd,
        Response::new().embed(
            CreateEmbed::new()
                .title("XP Added")
                .description(format!("<@{}> has been given {xp}XP", user.id.get()))
                .color(0x00ff00),
        ),
    )
    .await
}

async fn reset_xp(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
) -> ResponseResult {
    let options = Options {
        options: cmd.data.options(),
    };

    let Some(user) = options.get_user("user").into_owned() else {
        return Err(ResponseError::Execution(
            "No user provided",
            Some("Please provide a user to reset XP for".to_string()),
        ));
    };

    let Ok(member) = ctx.ctx.http.get_member(ctx.guild.id, user.id).await else {
        return Err(ResponseError::Execution(
            "Failed to fetch member",
            Some("Failed to fetch member".to_string()),
        ));
    };
    let member_roles = &member.roles;

    let level_rewards = match sqlx::query!(
        "SELECT role FROM xp_rewards WHERE guild_id = $1",
        ctx.guild.id.get() as i64
    )
    .fetch_all(&handler.main_database)
    .await
    {
        Ok(level_rewards) => level_rewards
            .iter()
            .map(|level_reward| RoleId::new(level_reward.role as u64))
            .collect::<Vec<_>>(),
        Err(err) => {
            error!("Failed to fetch XP rewards. Failed with error: {:?}", err);
            return Err(ResponseError::Execution(
                "Failed to fetch XP rewards",
                Some("Failed to fetch XP rewards".to_string()),
            ));
        }
    };

    let roles_to_remove = level_rewards
        .iter()
        .filter(|role| member_roles.contains(role))
        .map(|role| *role)
        .collect::<Vec<_>>();

    if let Err(err) = member
        .remove_roles(&ctx.ctx.http, roles_to_remove.as_slice())
        .await
    {
        error!(
            "Failed to remove roles from user. Failed with error: {:?}",
            err
        );
    };

    sqlx::query!(
        "UPDATE user_xp SET xp = 0 WHERE guild_id = $1 AND user_id = $2",
        ctx.guild.id.get() as i64,
        user.id.get() as i64
    )
    .execute(&handler.main_database)
    .await?;

    let logging_config = match sqlx::query_as!(
        LoggingConfig,
        "SELECT log_actions, log_messages, log_voice, log_channel, log_action_channel, log_message_channel, log_voice_channel FROM logging_configuration WHERE guild_id = $1",
        ctx.guild.id.get() as i64
    )
    .fetch_one(&handler.main_database)
    .await
    {
        Ok(logging_config) => logging_config,
        Err(err) => {
            error!(
                "Failed to fetch logging configuration. Failed with error: {:?}",
                err
            );
            return Err(ResponseError::Execution(
                "Failed to fetch logging configuration",
                Some("Failed to fetch logging configuration".to_string()),
            ));
        }
    };
    if let Some(channel) = get_log_channel(handler, &logging_config, &LogType::Action).await {
        if let Err(err) = ChannelId::new(channel as u64)
            .send_message(
                &ctx.ctx,
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .title("XP Reset")
                        .description(format!(
                            "<@{}> reset <@{}>'s XP",
                            cmd.user.id.get(),
                            user.id.get()
                        ))
                        .footer(CreateEmbedFooter::new(format!(
                            "User {} reset XP for user {}",
                            cmd.user.id.get(),
                            user.id.get()
                        )))
                        .color(0x00ff00),
                ),
            )
            .await
        {
            error!(
                "Failed to send XP reset log message. Failed with error: {:?}",
                err
            );
        };
    }

    ctx.reply(
        cmd,
        Response::new().embed(
            CreateEmbed::new()
                .title("XP Reset")
                .description(format!("<@{}> has been reset to 0XP", user.id.get()))
                .color(0x00ff00),
        ),
    )
    .await
}

async fn set_xp(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
) -> ResponseResult {
    let options = Options {
        options: cmd.data.options(),
    };

    let Some(user) = options.get_user("user").into_owned() else {
        return Err(ResponseError::Execution(
            "No user provided",
            Some("Please provide a user to add XP to".to_string()),
        ));
    };

    let user_xp = match sqlx::query!(
        "SELECT xp FROM user_xp WHERE guild_id = $1 AND user_id = $2",
        ctx.guild.id.get() as i64,
        user.id.get() as i64
    )
    .fetch_optional(&handler.main_database)
    .await
    {
        Ok(user_xp) => {
            if let Some(user_xp) = user_xp {
                user_xp.xp
            } else {
                if let Err(err) = sqlx::query!(
                    "INSERT INTO user_xp (guild_id, user_id, xp) VALUES ($1, $2, 0)",
                    ctx.guild.id.get() as i64,
                    user.id.get() as i64
                )
                .execute(&handler.main_database)
                .await
                {
                    error!("Failed to insert user XP. Failed with error: {:?}", err);
                    return Err(ResponseError::Execution(
                        "Failed to insert user XP",
                        Some("Failed to insert user XP".to_string()),
                    ));
                }
                0
            }
        }
        Err(err) => {
            error!("Failed to fetch user XP. Failed with error: {:?}", err);
            return Err(ResponseError::Execution(
                "Failed to fetch user XP",
                Some("Failed to fetch user XP".to_string()),
            ));
        }
    };

    let Some(xp) = options.get_integer("xp") else {
        return Err(ResponseError::Execution(
            "No XP provided",
            Some("Please provide a XP amount to add".to_string()),
        ));
    };

    let old_level = ((-25.0 + f64::sqrt((625 + (200 * user_xp)) as f64)) / 100.0).floor() as i64;
    let new_level = ((-25.0 + f64::sqrt((625 + (200 * xp)) as f64)) / 100.0).floor() as i64;

    sqlx::query!(
        "UPDATE user_xp SET xp = $1 WHERE guild_id = $2 AND user_id = $3",
        xp,
        ctx.guild.id.get() as i64,
        user.id.get() as i64
    )
    .execute(&handler.main_database)
    .await?;

    if new_level != old_level {
        let Ok(member) = ctx.ctx.http.get_member(ctx.guild.id, user.id).await else {
            return Err(ResponseError::Execution(
                "Failed to fetch member",
                Some("Failed to fetch member".to_string()),
            ));
        };

        handler
            .user_level_up(&ctx.ctx, member, cmd.channel_id, new_level)
            .await;
    }

    let logging_config = match sqlx::query_as!(
        LoggingConfig,
        "SELECT log_actions, log_messages, log_voice, log_channel, log_action_channel, log_message_channel, log_voice_channel FROM logging_configuration WHERE guild_id = $1",
        ctx.guild.id.get() as i64
    )
    .fetch_one(&handler.main_database)
    .await
    {
        Ok(logging_config) => logging_config,
        Err(err) => {
            error!(
                "Failed to fetch logging configuration. Failed with error: {:?}",
                err
            );
            return Err(ResponseError::Execution(
                "Failed to fetch logging configuration",
                Some("Failed to fetch logging configuration".to_string()),
            ));
        }
    };
    if let Some(channel) = get_log_channel(handler, &logging_config, &LogType::Action).await {
        if let Err(err) = ChannelId::new(channel as u64)
            .send_message(
                &ctx.ctx,
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .title("XP Set")
                        .description(format!(
                            "<@{}> set <@{}> XP to {xp}",
                            cmd.user.id.get(),
                            user.id.get()
                        ))
                        .footer(CreateEmbedFooter::new(format!(
                            "User {} set user {} XP to {xp}",
                            cmd.user.id.get(),
                            user.id.get()
                        )))
                        .color(0x00ff00),
                ),
            )
            .await
        {
            error!(
                "Failed to send XP set log message. Failed with error: {:?}",
                err
            );
        };
    }

    ctx.reply(
        cmd,
        Response::new().embed(
            CreateEmbed::new()
                .title("XP Set")
                .description(format!("<@{}>'s XP has been set to {xp}XP", user.id.get()))
                .color(0x00ff00),
        ),
    )
    .await
}

#[async_trait::async_trait]
impl Command for XPCommand {
    fn name(&self) -> &'static str {
        "xp"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("xp")
            .dm_permission(false)
            .description("Set, reset, or add to someone's XP")
            .add_option(
                CreateCommandOption::new(CommandOptionType::SubCommand, "set", "Set someone's XP")
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::User,
                            "user",
                            "The user who's XP to set",
                        )
                        .required(true),
                    )
                    .add_sub_option(
                        CreateCommandOption::new(CommandOptionType::Integer, "xp", "The XP to set")
                            .required(true),
                    ),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "reset",
                    "Reset someone's XP",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::User,
                        "user",
                        "The user who's XP to reset",
                    )
                    .required(true),
                ),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "add",
                    "Add to someone's XP",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::User,
                        "user",
                        "The user who's XP to add to",
                    )
                    .required(true),
                )
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::Integer, "xp", "The XP to add")
                        .required(true),
                ),
            )
    }

    async fn router(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> ResponseResult {
        if !ctx.user_permissions.contains(&Permission::XPEdit) {
            return Err(ResponseError::Execution(
                "You do not have permission to do this!",
                Some(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::XPEdit)),
            ));
        }

        for option in &cmd.data.options {
            match option.name.as_str() {
                "add" => return add_xp(handler, ctx, cmd).await,
                "reset" => return reset_xp(handler, ctx, cmd).await,
                "set" => return set_xp(handler, ctx, cmd).await,
                _ => continue,
            }
        }

        Err(ResponseError::Execution(
            "Invalid command",
            Some("You must specify a subcommand to use this command!".to_string()),
        ))
    }
}
