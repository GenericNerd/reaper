use serenity::all::RoleId;

pub struct Role {
    serenity_role: RoleId,
    raw: u64,
}

impl From<RoleId> for Role {
    fn from(serenity_role: RoleId) -> Self {
        Role {
            serenity_role,
            raw: serenity_role.get(),
        }
    }
}

impl From<u64> for Role {
    fn from(raw: u64) -> Self {
        Role {
            serenity_role: RoleId::new(raw),
            raw,
        }
    }
}

impl Role {
    pub fn as_u64(&self) -> u64 {
        self.raw
    }

    pub fn as_serenity_id(&self) -> RoleId {
        self.serenity_role
    }
}
