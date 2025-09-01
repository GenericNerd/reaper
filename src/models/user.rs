use serde::{Deserialize, Serialize};
use serenity::model::id::UserId as SerenityUserId;

#[derive(Clone, Copy, Debug)]
pub struct User {
    raw: u64,
    serenity_id: SerenityUserId,
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.raw.serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for User {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let raw = u64::deserialize(deserializer)?;
        Ok(User::from(raw))
    }
}

impl From<u64> for User {
    fn from(raw: u64) -> Self {
        Self {
            raw,
            serenity_id: SerenityUserId::new(raw),
        }
    }
}

impl From<SerenityUserId> for User {
    fn from(serenity_id: SerenityUserId) -> Self {
        Self {
            raw: serenity_id.get(),
            serenity_id,
        }
    }
}

impl User {
    pub fn as_u64(&self) -> u64 {
        self.raw
    }

    pub fn as_i64(&self) -> i64 {
        self.raw as i64
    }

    pub fn as_serenity_id(&self) -> SerenityUserId {
        self.serenity_id
    }
}
