use std::{
    fmt::Write,
    time::{Duration, Instant},
};

use serenity::{
    all::{ButtonStyle, CommandInteraction, ComponentInteractionDataKind},
    builder::{
        CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
        CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
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
        command::{CommandContext, CommandContextReply, InteractionContext},
        handler::Handler,
        highest_role::get_highest_role,
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
                            format!("Remove {permission}")
                        } else {
                            format!("Add {permission}")
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
    let mut start = Instant::now();

    let options = Options {
        options: cmd.data.options(),
    };
    let Some(user) = options.get_user("user").into_owned() else {
        return Err(ResponseError::Execution("No member found!", Some("The user option either was not provided, or this command was not ran in a guild. Both of these should not occur, if they do, please contact a developer.".to_string())));
    };

    let target_user_highest_role = get_highest_role(ctx, &user).await;
    if ctx.highest_role <= target_user_highest_role {
        return Err(ResponseError::Execution(
            "You cannot change the permissions of this user!",
            Some(
                "You cannot change the permissions of a user with a role equal to or higher than yours."
                    .to_string(),
            ),
        ));
    }

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
                        .description(existing_permissions.iter().fold(
                            String::new(),
                            |mut acc, permission| {
                                writeln!(&mut acc, "`{permission}`").unwrap();
                                acc
                            },
                        ))
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
        let interaction_context =
            InteractionContext::new(handler, ctx.ctx.clone(), &interaction).await;
        start = Instant::now();

        if interaction_context.interaction.user.id != cmd.user.id
            && !interaction_context
                .user_permissions
                .contains(&Permission::PermissionsEdit)
        {
            // TODO: Investigate method to condense this
            if let Err(err) = interaction_context.error_message(
                ResponseError::Execution(
                    "You do not have permission to do this",
                    Some(format!(
                        "You are missing the `{}` permission. If you believe this is a mistake, please contact your server administrators.",
                        Permission::PermissionsEdit)
                    )
                )).await {
                error!(
                    "Failed to reply to command interaction with error: {:?}",
                    err
                );
            }
            continue;
        }

        let permission_to_change = match &interaction_context.interaction.data.kind {
            ComponentInteractionDataKind::Button => {
                if interaction_context.interaction.data.custom_id == "done" {
                    for permission in &existing_permissions {
                        if !temp_permissions.contains(permission) {
                            remove_permission_from_user(
                                handler,
                                ctx.guild.id.get() as i64,
                                user.id.get() as i64,
                                permission,
                            )
                            .await;
                        }
                    }
                    for permission in &temp_permissions {
                        if !existing_permissions.contains(permission) {
                            add_permission_to_user(
                                handler,
                                ctx.guild.id.get() as i64,
                                user.id.get() as i64,
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

                    if let Err(err) = interaction_context
                        .interaction
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
                Permission::from(values[0].as_str())
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
            temp_permissions.push(permission_to_change);
        }

        if let Err(err) = ctx
            .reply(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(format!("{}'s permissions", user.name))
                            .description(temp_permissions.iter().fold(
                                String::new(),
                                |mut acc, permission| {
                                    writeln!(&mut acc, "`{permission}`").unwrap();
                                    acc
                                },
                            ))
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

        if let Err(err) = interaction_context
            .interaction
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
