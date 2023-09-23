use std::time::Duration;

use serenity::{
    all::{ButtonStyle, CommandInteraction, ComponentInteractionDataKind},
    builder::{
        CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
        CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind,
        CreateSelectMenuOption,
    },
    futures::StreamExt,
    model::Permissions,
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

fn create_components(permissions: &[Permission]) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::SelectMenu(CreateSelectMenu::new(
            "permissions".to_string(),
            CreateSelectMenuKind::String {
                options: Permission::iter()
                    .map(|permission| {
                        let label = if permissions.contains(&permission) {
                            format!("Remove {}", permission.to_string())
                        } else {
                            format!("Add {}", permission.to_string())
                        };

                        CreateSelectMenuOption::new(label, permission.to_string())
                    })
                    .collect(),
            },
        )),
        CreateActionRow::Buttons(vec![CreateButton::new("done")
            .emoji('✅')
            .style(ButtonStyle::Success)]),
    ]
}

pub async fn user(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
) -> ResponseResult {
    let mut start = std::time::Instant::now();

    let options = Options {
        options: cmd.data.options(),
    };
    let Some(user) = options.get_user("user").into_owned() else {
        return Err(ResponseError::ExecutionError("No member found!", Some("The user option either was not provided, or this command was not ran in a guild. Both of these should not occur, if they do, please contact a developer.".to_string())));
    };

    let has_admin = if let Ok(permissions) = ctx
        .guild
        .member(&ctx.ctx, user.id)
        .await
        .unwrap()
        .permissions(&ctx.ctx)
    {
        permissions.contains(Permissions::ADMINISTRATOR)
    } else {
        false
    };

    let existing_permissions = if user.id == ctx.guild.owner_id || has_admin {
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
        || has_admin
    {
        vec![]
    } else {
        create_components(&existing_permissions)
    };

    let message = match ctx
        .reply_get_message(
            cmd,
            Response::new()
                .embed(
                    CreateEmbed::new()
                        .title(format!("{}'s permissions", user.name))
                        .description(
                            existing_permissions
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

    if !ctx.user_permissions.contains(&Permission::PermissionsEdit)
        || user.id == ctx.guild.owner_id
        || has_admin
    {
        return Ok(());
    }

    let mut interaction_stream = message
        .await_component_interactions(&ctx.ctx)
        .timeout(Duration::new(60 * 60 * 24, 0))
        .stream();
    let mut temp_permissions = existing_permissions.clone();
    while let Some(interaction) = interaction_stream.next().await {
        start = std::time::Instant::now();

        if interaction.user.id != cmd.user.id
            && !get_user(
                handler,
                ctx.guild.id.0.get() as i64,
                interaction.user.id.0.get() as i64,
            )
            .await
            .contains(&Permission::PermissionsEdit)
        {
            // TODO: Investigate method to condense this
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
            continue;
        }

        let permission_to_change = match &interaction.data.kind {
            ComponentInteractionDataKind::Button => {
                if interaction.data.custom_id == "done" {
                    for permission in &existing_permissions {
                        if !temp_permissions.contains(permission) {
                            remove_permission_from_user(
                                handler,
                                ctx.guild.id.0.get() as i64,
                                user.id.0.get() as i64,
                                permission,
                            )
                            .await;
                        }
                    }
                    for permission in &temp_permissions {
                        if !existing_permissions.contains(permission) {
                            add_permission_to_user(
                                handler,
                                ctx.guild.id.0.get() as i64,
                                user.id.0.get() as i64,
                                permission,
                            )
                            .await;
                        }
                    }

                    if let Err(err) = cmd.delete_response(&ctx.ctx.http).await {
                        error!(
                            "Failed to delete command interaction response with error: {:?}",
                            err
                        );
                    }

                    if let Err(err) = interaction
                        .create_response(&ctx.ctx, CreateInteractionResponse::Acknowledge)
                        .await
                    {
                        error!(
                            "Failed to acknowledge command interaction with error: {:?}",
                            err
                        );
                    }

                    return Ok(());
                }
                continue;
            }
            ComponentInteractionDataKind::StringSelect { values } => {
                Permission::from(values[0].clone())
            }
            _ => continue,
        };

        if temp_permissions.contains(&permission_to_change) {
            temp_permissions.remove(
                temp_permissions
                    .iter()
                    .position(|permission| permission == &permission_to_change)
                    .unwrap(),
            );
        } else {
            temp_permissions.push(permission_to_change.clone());
        }

        if let Err(err) = ctx
            .reply(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(format!("{}'s permissions", user.name))
                            .description(
                                temp_permissions
                                    .iter()
                                    .map(|permission| format!("`{}`\n", permission.to_string()))
                                    .collect::<String>(),
                            )
                            .footer(CreateEmbedFooter::new(format!(
                                "Total execution time: {:?}",
                                start.elapsed()
                            ))),
                    )
                    .components(create_components(&temp_permissions)),
            )
            .await
        {
            error!(
                "Failed to reply to command interaction with error: {:?}",
                err
            );
        }

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

    Ok(())
}
