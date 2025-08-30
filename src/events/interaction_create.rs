use std::{
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use inflections::Inflect;
use metrics::{counter, histogram};
use serenity::all::{
    CommandInteraction, ComponentInteraction, Context as SerenityContext,
    CreateInteractionResponse, CreateInteractionResponseMessage, Interaction, PartialGuild,
    Permissions,
};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{
    events::EventRouter,
    models::{
        bot::Bot,
        context::{Context, ContextReply, PopulatedContext, UnpopulatedContext},
        guild::Guild,
        permissions::Permission,
        response::ResponseError,
        role::Role,
        user::User,
    },
};

impl EventRouter {
    #[tracing::instrument(skip(ctx, command), fields(command_name = command.data.name))]
    async fn on_command(&self, ctx: SerenityContext, command: CommandInteraction) {
        let timing = histogram!("bot.timing.on_command", "command" => command.data.name.clone());
        let start = std::time::Instant::now();
        let context = Context::Unpopulated(UnpopulatedContext { ctx: &ctx });
        counter!("bot.command_count").increment(1);

        // Check if all commands are disabled
        let commands_active =
            match sqlx::query!("SELECT active FROM global_kills WHERE feature = 'commands'")
                .fetch_one(Bot::global().postgres())
                .await
            {
                Ok(row) => row.active,
                Err(err) => {
                    error!("Failed to fetch global kills configuration: {err}");
                    let _res = context
                        .error_message(&command, ResponseError::Sqlx(err))
                        .await;
                    timing.record(start.elapsed());
                    return;
                }
            };

        if command.data.name != "global" && !commands_active {
            info!("Commands are disabled, not responding to command");
            let _res = context.error_message(&command, ResponseError::Execution(
                "Commands are currently disabled".to_string(),
                Some("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.".to_string()),
            )).await;
            timing.record(start.elapsed());
            return;
        }

        // Check if this specific command is disabled
        let command_active = match sqlx::query!(
            "SELECT active FROM global_kills WHERE feature = $1",
            format!("commands.{}", command.data.name)
        )
        .fetch_one(Bot::global().postgres())
        .await
        {
            Ok(row) => row.active,
            Err(err) => {
                error!("Failed to fetch global kills configuration: {err}");
                let _res = context
                    .error_message(&command, ResponseError::Sqlx(err))
                    .await;
                timing.record(start.elapsed());
                return;
            }
        };

        if command.data.name != "global" && !command_active {
            info!(
                "{} is disabled, not responding to command",
                command.data.name
            );
            let command_name = command.data.name.to_title_case();
            let _res = context
                .error_message(
                    &command,
                    ResponseError::Execution(
                        format!("{command_name} is currently disabled"),
                        Some("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.".to_string()),
                    ),
                )
                .await;
            timing.record(start.elapsed());
            return;
        }

        let user = User::from(command.user.id);

        // Check if this user is disabled
        if sqlx::query!(
            "SELECT user_id FROM user_kills WHERE user_id = $1",
            user.as_i64()
        )
        .fetch_optional(Bot::global().postgres())
        .await
        .unwrap_or(None)
        .is_some()
        {
            info!("User is disabled, not responding to command");
            let _res = context.error_message(&command, ResponseError::Execution(
                "You are currently disabled".to_string(),
                Some("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.".to_string()),
            )).await;
            timing.record(start.elapsed());
            return;
        }

        let raw_guild_id = command.guild_id;
        let Some(guild) = raw_guild_id.map(Guild::from) else {
            // Check if the command can even be run outside of a guild
            const ALLOWED_UNGUILDED_COMMANDS: [&str; 2] = ["privacy", "info"];
            if !ALLOWED_UNGUILDED_COMMANDS.contains(&command.data.name.as_str()) {
                debug!("Reaper command attempted outside of guild");
                let _res = context
                    .error_message(
                        &command,
                        ResponseError::Execution(
                            "Reaper cannot be used here".to_string(),
                            Some("This command can only be used in a server.".to_string()),
                        ),
                    )
                    .await;

                timing.record(start.elapsed());
                return;
            }

            let executing_command = match Bot::global().commands().get(&command.data.name.as_str())
            {
                Some(cmd) => cmd,
                None => {
                    let _res = context.error_message(&command, ResponseError::Execution("Command not found".to_string(), Some("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.".to_string()))).await;
                    timing.record(start.elapsed());
                    return;
                }
            };

            let _res = executing_command.router(&context, &command).await;
            timing.record(start.elapsed());
            return;
        };

        // Check if the guild is disabled
        if sqlx::query!(
            "SELECT guild_id FROM guild_kills WHERE guild_id = $1",
            guild.as_i64()
        )
        .fetch_optional(Bot::global().postgres())
        .await
        .unwrap_or(None)
        .is_some()
        {
            debug!("Guild is disabled, not responding to command");
            let _res = context
                .error_message(
                    &command,
                    ResponseError::Execution(
                        "This server is disabled".to_string(),
                        Some("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.".to_string()),
                    ),
                )
                .await;
            timing.record(start.elapsed());
            return;
        }

        let mut partial_guild = None;
        debug!(
            "Attempting cache of {}'s partial guild from cache",
            guild.as_u64()
        );
        let cached_guild = guild
            .as_serenity_id()
            .to_guild_cached(&ctx.cache)
            .map(|guild| PartialGuild::from(guild.clone()));

        if let Some(cached_partial_guild) = cached_guild {
            debug!(
                "Successfully cached {}'s partial guild from cache",
                guild.as_u64()
            );
            partial_guild = Some(cached_partial_guild);
        } else if let Ok(fetched_guild) = guild.as_serenity_id().to_partial_guild(&ctx).await {
            debug!(
                "Local cache failed, obtaining {}'s partial guild via API",
                guild.as_u64()
            );
            partial_guild = Some(fetched_guild);
        }

        let Some(partial_guild) = partial_guild else {
            error!("Failed to fetch guild");
            let _res = context
                .error_message(&command, ResponseError::Execution(
                    "Failed to fetch guild".to_string(),
                    Some("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.".to_string()),
                ))
                .await;
            timing.record(start.elapsed());
            return;
        };

        debug!("Obtaining information required to populate context");
        let mut highest_role = 0;
        let user_permissions: Vec<Permission> = if partial_guild.owner_id == user.as_serenity_id() {
            highest_role = u16::MAX;
            enum_iterator::all::<Permission>().collect::<Vec<_>>()
        } else {
            let mut user_permissions: Vec<Permission> = vec![];
            for user_permission in Permission::get_user(guild, user).await {
                if !user_permissions.contains(&user_permission) {
                    user_permissions.push(user_permission);
                }
            }
            for role in command.member.clone().unwrap().roles {
                if let Some(role) = partial_guild.roles.get(&role) {
                    if role.position > highest_role {
                        highest_role = role.position;
                    }

                    if role.permissions.contains(Permissions::ADMINISTRATOR) {
                        highest_role = u16::MAX - 1;
                        user_permissions = enum_iterator::all::<Permission>().collect::<Vec<_>>();
                        break;
                    }
                }

                let role = Role::from(role);

                for role_permission in Permission::get_role(guild, role).await {
                    if !user_permissions.contains(&role_permission) {
                        user_permissions.push(role_permission);
                    }
                }
            }
            let everyone_role = Role::from(guild.as_u64());
            let everyone_role = Permission::get_role(guild, everyone_role).await;
            for role_permission in everyone_role {
                if !user_permissions.contains(&role_permission) {
                    user_permissions.push(role_permission);
                }
            }
            user_permissions
        };

        let context = Context::Populated(PopulatedContext {
            ctx: &ctx,
            has_responded: Arc::new(AtomicBool::new(false)),
            user_permissions,
            highest_role,
            guild,
        });
        debug!("Generated context in {:?}", start.elapsed());

        let executing_command = match Bot::global().commands().get(&command.data.name.as_str()) {
            Some(cmd) => cmd,
            None => {
                let _res = context.error_message(&command, ResponseError::Execution("Command not found".to_string(), Some("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.".to_string()))).await;
                timing.record(start.elapsed());
                return;
            }
        };

        if executing_command.name() != "privacy" {
            if let Err(err) = command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Defer(CreateInteractionResponseMessage::default()),
                )
                .await
            {
                error!("Failed to defer command: {err}");
                let _res = context
                    .error_message(&command, ResponseError::Serenity(err))
                    .await;
                timing.record(start.elapsed());
                return;
            }
            if let Context::Populated(ctx) = &context {
                ctx.has_responded.store(true, Ordering::Relaxed);
            }
        }

        if let Some(required_permission) = executing_command.required_permission() {
            let user_permissions = match &context {
                Context::Populated(ctx) => &ctx.user_permissions,
                Context::Unpopulated(_) => return,
            };

            if !user_permissions.contains(&required_permission) {
                let _res = context.error_message(&command, ResponseError::Execution("You do not have permission to use this command".to_string(), Some("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.".to_string()))).await;
                timing.record(start.elapsed());
                return;
            }
        }

        let res = match executing_command.router(&context, &command).await {
            Ok(_) => Ok(()),
            Err(err) => {
                let _res = context.error_message(&command, err).await;
                Err(())
            }
        };
        counter!("bot.command_execution", "command" => executing_command.name(), "status" => if res.is_ok() { "success" } else { "failure" }).increment(1);

        debug!(
            "Command {} executed in {:?}",
            executing_command.name(),
            start.elapsed()
        );
        timing.record(start.elapsed());
    }

    #[tracing::instrument(skip(ctx, component), fields(interaction_id = component.data.custom_id))]
    async fn on_component(&self, ctx: SerenityContext, component: ComponentInteraction) {
        let timing =
            histogram!("bot.timing.on_component", "component" => component.data.custom_id.clone());
        let start = std::time::Instant::now();
        let context = Context::Unpopulated(UnpopulatedContext { ctx: &ctx });
        counter!("bot.component_count").increment(1);

        let components_active =
            match sqlx::query!("SELECT active FROM global_kills WHERE feature = 'commands'")
                .fetch_one(Bot::global().postgres())
                .await
            {
                Ok(row) => row.active,
                Err(err) => {
                    error!("Failed to fetch global kills configuration: {err}");
                    let _res = context
                        .error_message(&component, ResponseError::Sqlx(err))
                        .await;
                    timing.record(start.elapsed());
                    return;
                }
            };

        let interaction_id = Uuid::from_str(component.data.custom_id.as_str()).unwrap();

        let Some(interaction) = Bot::global().interaction_state().get(interaction_id).await else {
            return;
        };
        // TODO: Not use unwrap, it's ugly
        let interaction_name = interaction
            .data
            .get("interaction")
            .unwrap()
            .as_str()
            .unwrap();

        if interaction_name != "global" && !components_active {
            info!(
                "{} is disabled, not responding to component",
                interaction_name
            );
            let component_name = interaction_name.to_title_case();
            let _res = context
                .error_message(
                    &component,
                    ResponseError::Execution(
                        format!("{component_name} is currently disabled"),
                        Some("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.".to_string()),
                    ),
                )
                .await;
            timing.record(start.elapsed());
            return;
        }

        let user = User::from(component.user.id);

        // Check if this user is disabled
        if sqlx::query!(
            "SELECT user_id FROM user_kills WHERE user_id = $1",
            user.as_i64()
        )
        .fetch_optional(Bot::global().postgres())
        .await
        .unwrap_or(None)
        .is_some()
        {
            info!("User is disabled, not responding to component");
            let _res = context.error_message(&component, ResponseError::Execution(
                "You are currently disabled".to_string(),
                Some("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.".to_string()),
            )).await;
            timing.record(start.elapsed());
            return;
        }

        let Some(raw_guild_id) = component.guild_id else {
            debug!("Component interaction attempted outside of guild");
            let _res = context
                .error_message(
                    &component,
                    ResponseError::Execution(
                        "Reaper cannot be used here".to_string(),
                        Some("This command can only be used in a server.".to_string()),
                    ),
                )
                .await;
            timing.record(start.elapsed());
            return;
        };

        let guild = Guild::from(raw_guild_id);

        // Check if the guild is disabled
        if sqlx::query!(
            "SELECT guild_id FROM guild_kills WHERE guild_id = $1",
            guild.as_i64()
        )
        .fetch_optional(Bot::global().postgres())
        .await
        .unwrap_or(None)
        .is_some()
        {
            debug!("Guild is disabled, not responding to component");
            let _res = context
                .error_message(
                    &component,
                    ResponseError::Execution(
                        "This server is disabled".to_string(),
                        Some("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.".to_string()),
                    ),
                )
                .await;
            timing.record(start.elapsed());
            return;
        }

        let mut partial_guild = None;
        debug!(
            "Attempting cache of {}'s partial guild from cache",
            guild.as_u64()
        );
        let cached_guild = guild
            .as_serenity_id()
            .to_guild_cached(&ctx.cache)
            .map(|guild| PartialGuild::from(guild.clone()));

        if let Some(cached_partial_guild) = cached_guild {
            debug!(
                "Successfully cached {}'s partial guild from cache",
                guild.as_u64()
            );
            partial_guild = Some(cached_partial_guild);
        } else if let Ok(fetched_guild) = guild.as_serenity_id().to_partial_guild(&ctx).await {
            debug!(
                "Local cache failed, obtaining {}'s partial guild via API",
                guild.as_u64()
            );
            partial_guild = Some(fetched_guild);
        }

        let Some(partial_guild) = partial_guild else {
            error!("Failed to fetch guild");
            let _res = context
                .error_message(&component, ResponseError::Execution(
                    "Failed to fetch guild".to_string(),
                    Some("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.".to_string()),
                ))
                .await;
            timing.record(start.elapsed());
            return;
        };

        debug!("Obtaining information required to populate context");
        let mut highest_role = 0;
        let user_permissions: Vec<Permission> = if partial_guild.owner_id == user.as_serenity_id() {
            highest_role = u16::MAX;
            enum_iterator::all::<Permission>().collect::<Vec<_>>()
        } else {
            let mut user_permissions: Vec<Permission> = vec![];
            for user_permission in Permission::get_user(guild, user).await {
                if !user_permissions.contains(&user_permission) {
                    user_permissions.push(user_permission);
                }
            }
            for role in component.member.clone().unwrap().roles {
                if let Some(role) = partial_guild.roles.get(&role) {
                    if role.position > highest_role {
                        highest_role = role.position;
                    }

                    if role.permissions.contains(Permissions::ADMINISTRATOR) {
                        highest_role = u16::MAX - 1;
                        user_permissions = enum_iterator::all::<Permission>().collect::<Vec<_>>();
                        break;
                    }
                }

                let role = Role::from(role);

                for role_permission in Permission::get_role(guild, role).await {
                    if !user_permissions.contains(&role_permission) {
                        user_permissions.push(role_permission);
                    }
                }
            }
            let everyone_role = Role::from(guild.as_u64());
            let everyone_role = Permission::get_role(guild, everyone_role).await;
            for role_permission in everyone_role {
                if !user_permissions.contains(&role_permission) {
                    user_permissions.push(role_permission);
                }
            }
            user_permissions
        };

        let context = Context::Populated(PopulatedContext {
            ctx: &ctx,
            has_responded: Arc::new(AtomicBool::new(false)),
            user_permissions,
            highest_role,
            guild,
        });
        debug!("Generated context in {:?}", start.elapsed());

        info!("Executing component {}", interaction_name);
        let executing_component = match Bot::global().components().get(&interaction_name) {
            Some(component) => component,
            None => {
                let _res = context.error_message(&component, ResponseError::Execution("Component not found".to_string(), Some("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.".to_string()))).await;
                timing.record(start.elapsed());
                return;
            }
        };

        if let Err(err) = component
            .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
            .await
        {
            error!("Failed to acknowledge component: {err}");
            let _res = context
                .error_message(&component, ResponseError::Serenity(err))
                .await;
            timing.record(start.elapsed());
            return;
        }

        if let Some(required_permission) = executing_component.required_permission() {
            let user_permissions = match &context {
                Context::Populated(ctx) => &ctx.user_permissions,
                Context::Unpopulated(_) => return,
            };

            if !user_permissions.contains(&required_permission) {
                let _res = context.error_message(&component, ResponseError::Execution("You do not have permission to use this command".to_string(), Some("Please reach out to the [support server](https://discord.gg/jhD3Xc5cm6) for more information.".to_string()))).await;
                timing.record(start.elapsed());
                return;
            }
        }

        let res = match executing_component.router(&context, &component).await {
            Ok(_) => Ok(()),
            Err(err) => {
                let _res = context.error_message(&component, err).await;
                Err(())
            }
        };
        counter!("bot.component_execution", "component" => executing_component.name(), "status" => if res.is_ok() { "success" } else { "failure" }).increment(1);

        debug!(
            "Component {} executed in {:?}",
            executing_component.name(),
            start.elapsed()
        );
        timing.record(start.elapsed());
    }

    pub async fn interaction_create(&self, ctx: SerenityContext, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => self.on_command(ctx, command).await,
            Interaction::Component(component) => self.on_component(ctx, component).await,
            // Interaction::Modal(modal) => self.on_modal(ctx, modal).await
            _ => todo!(),
        }
    }
}
