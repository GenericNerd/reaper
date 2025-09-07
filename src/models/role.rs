use serenity::model::id::RoleId as SerenityRoleId;

#[derive(Clone, Copy)]
pub struct Role {
    raw: u64,
    serenity_id: SerenityRoleId,
}

impl From<u64> for Role {
    fn from(raw: u64) -> Self {
        Self {
            raw,
            serenity_id: SerenityRoleId::new(raw),
        }
    }
}

impl From<i64> for Role {
    fn from(raw: i64) -> Self {
        Self::from(raw as u64)
    }
}

impl From<SerenityRoleId> for Role {
    fn from(serenity_id: SerenityRoleId) -> Self {
        Self {
            raw: serenity_id.get(),
            serenity_id,
        }
    }
}

impl Role {
    pub fn as_i64(&self) -> i64 {
        self.raw as i64
    }

    pub fn as_u64(&self) -> u64 {
        self.raw
    }

    pub fn as_serenity(&self) -> SerenityRoleId {
        self.serenity_id
    }
}
