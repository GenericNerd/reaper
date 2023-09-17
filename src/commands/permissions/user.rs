use crate::models::command::Context;
use serenity::{
    all::CommandInteraction,
    builder::{
        CreateActionRow, CreateEmbed, CreateEmbedFooter, CreateSelectMenu, CreateSelectMenuKind,
        CreateSelectMenuOption,
    },
};
use strum::IntoEnumIterator;

use crate::{
    common::options::Options,
    database::postgres::permissions::get_user,
    models::{
        command::{CommandContext, CommandResult},
        handler::Handler,
        permissions::Permission,
        response::Response,
    },
};

pub async fn user(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
) -> CommandResult {
    let start = std::time::Instant::now();

    let options = Options {
        options: cmd.data.options(),
    };
    let user = match options.get_user("user").into_owned() {
        Some(user) => user,
        None => {
            return ctx.reply(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title("No member found!")
                            .description("The user option either was not provided, or this command was not ran in a guild. Both of these should not occur, if they do, please contact a developer.")
                            .color(0xf00)
                    )
                ).await;
        }
    };

    let permission_set = if user.id == ctx.guild.owner_id {
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
            "permissions",
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

    return ctx
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
                .components(components),
        )
        .await;
}
