use crate::models::command::Context;
use std::sync::atomic::AtomicBool;
use strum::IntoEnumIterator;

use serenity::{
    all::{CommandInteraction, PartialGuild},
    prelude::Context as IncomingContext,
};
use tracing::{debug, error};

use crate::{
    commands::get_command_list,
    database::postgres::permissions::{get_role, get_user},
    models::{
        command::{CommandContext, FailedCommandContext},
        handler::Handler,
        permissions::Permission,
        response::Response,
    },
};

impl Handler {
    pub async fn on_command(&self, ctx: IncomingContext, command: CommandInteraction) {
        let start = std::time::Instant::now();

        let guild_id = match command.guild_id {
            Some(guild_id) => guild_id,
            None => {
                let fail_context = FailedCommandContext { ctx };
                if let Err(err) = fail_context
                    .reply(
                        &command,
                        Response::new()
                            .content("Reaper cannot be used outside of guilds".to_string()),
                    )
                    .await
                {
                    error!("Failed to reply to command: {:?}", err);
                }
                return;
            }
        };

        let mut temp_guild = match guild_id.to_guild_cached(&ctx.cache) {
            Some(guild) => Some(PartialGuild::from(guild.clone())),
            None => None,
        };
        if let None = temp_guild {
            temp_guild = match guild_id.to_partial_guild(&ctx.http).await {
                Ok(guild) => Some(guild),
                Err(_) => {
                    let fail_context = FailedCommandContext { ctx: ctx.clone() };
                    if let Err(err) = fail_context
                        .reply(
                            &command,
                            Response::new()
                                .content("Reaper could not obtain the guild".to_string()),
                        )
                        .await
                    {
                        error!("Failed to reply to command: {:?}", err);
                    }
                    return;
                }
            }
        }
        let guild = temp_guild.unwrap();

        debug!("Took {:?} to get guild ID and guild", start.elapsed());

        let user_permissions = if guild.owner_id == command.user.id {
            Permission::iter().collect::<Vec<_>>()
        } else {
            let mut user_permissions: Vec<Permission> = vec![];
            for user_permission in get_user(
                self,
                guild_id.0.get() as i64,
                command.user.id.0.get() as i64,
            )
            .await
            {
                if !user_permissions.contains(&user_permission) {
                    user_permissions.push(user_permission);
                }
            }
            for role in command.member.clone().unwrap().roles {
                for role_permission in
                    get_role(self, guild_id.0.get() as i64, role.0.get() as i64).await
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
            has_responsed: AtomicBool::new(false),
            user_permissions: user_permissions,
            guild,
        };

        debug!("Context generated in {:?}", start.elapsed());

        for existing_command in get_command_list() {
            if existing_command.name() == command.data.name {
                if let Err(err) = existing_command
                    .router(self, &command_context, &command)
                    .await
                {
                    error!("Failed to handle command: {:?}", err);
                }
            }
        }

        debug!("Took {:?} to handle a command", start.elapsed());
    }
}
