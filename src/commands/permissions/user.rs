use serenity::{
    all::{CommandInteraction, ComponentInteractionDataKind},
    builder::{
        CreateActionRow, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
        CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind,
        CreateSelectMenuOption,
    },
    futures::StreamExt,
};
use strum::IntoEnumIterator;
use tracing::error;

use crate::{
    common::options::Options,
    database::postgres::permissions::{
        add_permission_to_user, get_user, remove_permission_from_user,
    },
    models::{
        command::{CommandContext, CommandContextReply},
        handler::Handler,
        permissions::Permission,
        response::{Response, ResponseError, ResponseResult},
    },
};

pub async fn user(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
) -> ResponseResult {
    let mut start = std::time::Instant::now();

    let options = Options {
        options: cmd.data.options(),
    };
    let user = match options.get_user("user").into_owned() {
        Some(user) => user,
        None => {
            return Err(ResponseError::ExecutionError("No member found!", Some("The user option either was not provided, or this command was not ran in a guild. Both of these should not occur, if they do, please contact a developer.".to_string())));
        }
    };

    let mut permission_set = if user.id == ctx.guild.owner_id {
        Permission::iter().collect::<Vec<_>>()
    } else {
        get_user(
            handler,
            cmd.guild_id.unwrap().get() as i64,
            user.id.get() as i64,
        )
        .await
    };

    let components = if !ctx.user_permissions.contains(&Permission::PermissionsEdit)
        || user.id == ctx.guild.owner_id
    {
        vec![]
    } else {
        vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
            format!("permissions->user {}.{}", ctx.guild.id.get(), user.id.get()),
            CreateSelectMenuKind::String {
                options: Permission::iter()
                    .map(|permission| {
                        let label = if !permission_set.contains(&permission) {
                            format!("Add {}", permission.to_string())
                        } else {
                            format!("Remove {}", permission.to_string())
                        };

                        CreateSelectMenuOption::new(label, permission.to_string())
                    })
                    .collect(),
            },
        ))]
    };

    let message = match ctx
        .reply_get_message(
            cmd,
            Response::new()
                .embed(
                    CreateEmbed::new()
                        .title(format!("{}'s permissions", user.name))
                        .description(
                            permission_set
                                .iter()
                                .map(|permission| format!("`{}`\n", permission.to_string()))
                                .collect::<String>(),
                        )
                        .footer(CreateEmbedFooter::new(format!(
                            "Total execution time: {:?}",
                            start.elapsed()
                        ))),
                )
                .components(components),
        )
        .await
    {
        Ok(message) => message,
        Err(err) => return Err(err),
    };

    let mut interaction_stream = message.await_component_interactions(&ctx.ctx).stream();
    while let Some(interaction) = interaction_stream.next().await {
        start = std::time::Instant::now();

        let permission_to_change = Permission::from(match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => values[0].clone(),
            _ => continue,
        });

        if interaction.user.id != cmd.user.id {
            if !get_user(
                handler,
                ctx.guild.id.0.get() as i64,
                interaction.user.id.0.get() as i64,
            )
            .await
            .contains(&Permission::PermissionsEdit)
            {
                if let Err(err) = interaction
                    .create_response(
                        &ctx.ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .embed(
                                    CreateEmbed::new()
                                        .title("You do not have permission to do this!")
                                        .description(format!("You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.", Permission::PermissionsEdit.to_string()))
                                        .color(0xf00),
                                )
                                .ephemeral(true),
                        ),
                    )
                    .await
                {
                    error!(
                        "Failed to reply to command interaction with error: {:?}",
                        err
                    );
                }
            }
            continue;
        }

        if permission_set.contains(&permission_to_change) {
            permission_set.remove(
                permission_set
                    .iter()
                    .position(|permission| permission == permission)
                    .unwrap(),
            );
            remove_permission_from_user(
                handler,
                ctx.guild.id.0.get() as i64,
                user.id.0.get() as i64,
                &permission_to_change,
            )
            .await;
        } else {
            permission_set.push(permission_to_change.clone());
            add_permission_to_user(
                handler,
                ctx.guild.id.0.get() as i64,
                user.id.0.get() as i64,
                &permission_to_change,
            )
            .await;
        }

        match ctx
            .reply(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(format!("{}'s permissions", user.name))
                            .description(
                                permission_set
                                    .iter()
                                    .map(|permission| format!("`{}`\n", permission.to_string()))
                                    .collect::<String>(),
                            )
                            .footer(CreateEmbedFooter::new(format!(
                                "Total execution time: {:?}",
                                start.elapsed()
                            ))),
                    )
                    .components(vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
                        format!("permissions->user {}.{}", ctx.guild.id.get(), user.id.get()),
                        CreateSelectMenuKind::String {
                            options: Permission::iter()
                                .map(|permission| {
                                    let label = if !permission_set.contains(&permission) {
                                        format!("Add {}", permission.to_string())
                                    } else {
                                        format!("Remove {}", permission.to_string())
                                    };

                                    CreateSelectMenuOption::new(label, permission.to_string())
                                })
                                .collect(),
                        },
                    ))]),
            )
            .await
        {
            Ok(_) => {
                if let Err(err) = interaction
                    .create_response(&ctx.ctx, CreateInteractionResponse::Acknowledge)
                    .await
                {
                    error!(
                        "Failed to acknowledge command interaction with error: {:?}",
                        err
                    );
                }
            }
            Err(err) => {
                error!(
                    "Failed to reply to command interaction with error: {:?}",
                    err
                );
                return Err(err);
            }
        }
    }

    Ok(())
}
