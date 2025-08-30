use serenity::all::Member as SerenityMember;

use crate::models::{guild::Guild, user::User};

#[derive(Clone)]
pub struct Member {
    raw: SerenityMember,
    guild: Guild,
    user: User,
}

impl From<SerenityMember> for Member {
    fn from(value: SerenityMember) -> Self {
        Self {
            guild: Guild::from(value.guild_id),
            user: User::from(value.user.id),
            raw: value,
        }
    }
}

impl Member {
    pub fn as_serenity(&self) -> &SerenityMember {
        &self.raw
    }

    pub fn guild(&self) -> Guild {
        self.guild
    }

    pub fn user(&self) -> User {
        self.user
    }
}
