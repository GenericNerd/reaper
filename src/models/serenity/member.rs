use serenity::all::Member as SerenityMember;

use crate::models::serenity::{guild::Guild, role::Role, user::User};

pub struct Member {
    pub user: User,
    pub guild: Guild,
    pub roles: Vec<Role>,
}

impl From<SerenityMember> for Member {
    fn from(serenity_member: SerenityMember) -> Self {
        Member {
            user: User::from(serenity_member.user.id),
            guild: Guild::from(serenity_member.guild_id),
            roles: serenity_member
                .roles
                .iter()
                .map(|role| Role::from(*role))
                .collect(),
        }
    }
}

impl From<Box<SerenityMember>> for Member {
    fn from(serenity_member: Box<SerenityMember>) -> Self {
        Member {
            user: User::from(serenity_member.user.id),
            guild: Guild::from(serenity_member.guild_id),
            roles: serenity_member
                .roles
                .iter()
                .map(|role| Role::from(*role))
                .collect(),
        }
    }
}
