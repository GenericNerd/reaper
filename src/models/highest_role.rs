use serenity::all::{Permissions, User};

use super::command::CommandContext;

pub async fn get_highest_role(ctx: &CommandContext, user: &User) -> u16 {
    if ctx.guild.owner_id == user.id {
        return u16::max_value();
    }

    let Ok(member) = ctx.guild.member(&ctx.ctx.http, user.id).await else {
        return 0;
    };

    let mut highest_role = 0;
    for role in member.roles {
        if let Some(role) = ctx.guild.roles.get(&role) {
            if role.permissions.contains(Permissions::ADMINISTRATOR) {
                return u16::max_value() - 1;
            }

            if role.position > highest_role {
                highest_role = role.position;
            }
        }
    }

    highest_role
}
