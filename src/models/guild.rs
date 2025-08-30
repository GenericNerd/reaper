use serenity::model::id::GuildId as SerenityGuildId;

#[derive(Debug, Clone, Copy)]
pub struct Guild {
    raw_id: u64,
    serenity_id: SerenityGuildId,
}

impl From<u64> for Guild {
    fn from(raw_id: u64) -> Self {
        Self {
            raw_id,
            serenity_id: SerenityGuildId::new(raw_id),
        }
    }
}

impl From<SerenityGuildId> for Guild {
    fn from(serenity_id: SerenityGuildId) -> Self {
        Self {
            raw_id: serenity_id.get(),
            serenity_id,
        }
    }
}

impl Guild {
    pub fn as_u64(&self) -> u64 {
        self.raw_id
    }

    pub fn as_i64(&self) -> i64 {
        self.raw_id as i64
    }

    pub fn as_serenity_id(&self) -> SerenityGuildId {
        self.serenity_id
    }
}
