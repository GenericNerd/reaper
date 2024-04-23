use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};
use strum::IntoEnumIterator;

use crate::{
    commands::{get_command_list, global::get_kill_commands},
    database::postgres::permissions::{get_role, get_user},
    models::{
        command::{CommandContext, CommandContextReply, FailedCommandContext},
        handler::Handler,
        permissions::Permission,
        response::{Response, ResponseError},
    },
};
use inflections::Inflect;
use serenity::{
    all::{CommandInteraction, PartialGuild},
    builder::{CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage},
    model::Permissions,
    prelude::Context as IncomingContext,
};
use tracing::{debug, error};

impl Handler {
    pub async fn on_command(&self, ctx: IncomingContext, command: CommandInteraction) {
        if !sqlx::query!("SELECT active FROM global_kills WHERE feature = 'commands'")
            .fetch_one(&self.main_database)
            .await
            .unwrap()
            .active
        {
            let fail_context = FailedCommandContext { ctx };
            if let Err(err) = fail_context
                .reply(
                    &command,
                    Response::new()
                        .embed(
                            CreateEmbed::new()
                                .title("Commands are currently disabled")
                                .color(0xff0000)
                                .description("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.")
                        ),
                )
                .await
            {
                error!("Failed to reply to command: {:?}", err);
            }
            return;
        }

        let start = Instant::now();

        let Some(guild_id) = command.guild_id else {
            let fail_context = FailedCommandContext { ctx };
            if let Err(err) = fail_context
                .reply(
                    &command,
                    Response::new().content("Reaper cannot be used outside of guilds"),
                )
                .await
            {
                error!("Failed to reply to command: {:?}", err);
            }
            return;
        };

        let mut temp_guild = guild_id
            .to_guild_cached(&ctx.cache)
            .map(|guild| PartialGuild::from(guild.clone()));
        if temp_guild.is_none() {
            temp_guild = if let Ok(guild) = guild_id.to_partial_guild(&ctx.http).await {
                Some(guild)
            } else {
                let fail_context = FailedCommandContext { ctx: ctx.clone() };
                if let Err(err) = fail_context
                    .reply(
                        &command,
                        Response::new().content("Reaper could not obtain the guild"),
                    )
                    .await
                {
                    error!("Failed to reply to command: {:?}", err);
                }
                return;
            }
        }
        let guild = temp_guild.unwrap();

        debug!("Took {:?} to get guild ID and guild", start.elapsed());

        let user_permissions = if guild.owner_id == command.user.id {
            Permission::iter().collect::<Vec<_>>()
        } else {
            let mut user_permissions: Vec<Permission> = vec![];
            for user_permission in
                get_user(self, guild_id.get() as i64, command.user.id.get() as i64).await
            {
                if !user_permissions.contains(&user_permission) {
                    user_permissions.push(user_permission);
                }
            }
            for role in command.member.clone().unwrap().roles {
                if let Some(role) = guild.roles.get(&role) {
                    if role.permissions.contains(Permissions::ADMINISTRATOR) {
                        user_permissions = Permission::iter().collect::<Vec<_>>();
                        break;
                    }
                }

                for role_permission in
                    get_role(self, guild_id.get() as i64, role.get() as i64).await
                {
                    if !user_permissions.contains(&role_permission) {
                        user_permissions.push(role_permission);
                    }
                }
            }
            user_permissions
        };

        let command_context = CommandContext {
            ctx,
            has_responsed: Arc::new(AtomicBool::new(true)),
            user_permissions,
            guild,
        };

        debug!("Context generated in {:?}", start.elapsed());

        if command.data.name != "global"
            && !sqlx::query!(
                "SELECT active FROM global_kills WHERE feature = $1",
                format!("commands.{}", command.data.name)
            )
            .fetch_one(&self.main_database)
            .await
            .unwrap()
            .active
        {
            if let Err(err) = command_context
                    .reply(
                        &command,
                        Response::new()
                            .embed(
                                CreateEmbed::new()
                                    .title(format!("{} is currently disabled", command.data.name.to_title_case()))
                                    .color(0xff0000)
                                    .description("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.")
                            )
                    )
                    .await
                {
                    error!("Failed to reply to command: {:?}", err);
                }
            return;
        }

        let mut existing_commands = get_command_list();
        existing_commands.extend(get_kill_commands());

        for existing_command in existing_commands {
            if existing_command.name() == command.data.name {
                if let Err(err) = command
                    .create_response(
                        &command_context.ctx,
                        CreateInteractionResponse::Defer(
                            CreateInteractionResponseMessage::default(),
                        ),
                    )
                    .await
                {
                    error!(
                        "Failed to acknowledge command, took {:?}: {err:?}",
                        start.elapsed(),
                    );
                    return;
                };

                if let Err(err) = existing_command
                    .router(self, &command_context, &command)
                    .await
                {
                    error!("Failed to handle command: {:?}. Sending error message", err);
                    match err {
                        ResponseError::Execution(title, description) => {
                            if let Err(err) = command_context
                                .reply(
                                    &command,
                                    Response::new()
                                        .embed(
                                            CreateEmbed::new()
                                                .title(title)
                                                .description(description.unwrap_or(String::new()))
                                                .color(0xff0000),
                                        )
                                        .components(vec![])
                                        .ephemeral(true),
                                )
                                .await
                            {
                                error!("Failed to send error message: {:?}", err);
                            }
                        }
                        ResponseError::Serenity(err) => {
                            if let Err(err) = command_context
                                .reply(
                                    &command,
                                    Response::new().embed(
                                        CreateEmbed::new()
                                            .title("A Discord error occured while executing the command")
                                            .description(format!("```{err:?}```"))
                                            .color(0xff0000),
                                    )
                                    .components(vec![])
                                    .ephemeral(true),
                                )
                                .await
                            {
                                error!("Failed to send error message: {err:?}");
                            }
                        }
                        ResponseError::Redis(_) => {}
                    }
                }
            }
        }

        debug!("Took {:?} to handle a command", start.elapsed());
    }
}
