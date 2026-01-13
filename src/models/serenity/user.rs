use serenity::all::UserId;

pub struct User {
    serenity_user: UserId,
    raw: u64,
}

impl From<UserId> for User {
    fn from(serenity_user: UserId) -> Self {
        User {
            serenity_user,
            raw: serenity_user.get(),
        }
    }
}

impl From<u64> for User {
    fn from(raw: u64) -> Self {
        User {
            serenity_user: UserId::new(raw),
            raw,
        }
    }
}

impl User {
    pub fn as_u64(&self) -> u64 {
        self.raw
    }

    pub fn as_serenity_id(&self) -> UserId {
        self.serenity_user
    }
}
