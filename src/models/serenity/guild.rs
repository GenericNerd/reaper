use serenity::all::GuildId;

pub struct Guild {
    serenity_guild: GuildId,
    raw: u64,
}

impl From<GuildId> for Guild {
    fn from(serenity_guild: GuildId) -> Self {
        Guild {
            serenity_guild,
            raw: serenity_guild.get(),
        }
    }
}

impl From<u64> for Guild {
    fn from(raw: u64) -> Self {
        Guild {
            serenity_guild: GuildId::new(raw),
            raw,
        }
    }
}

impl Guild {
    pub fn as_u64(&self) -> u64 {
        self.raw
    }

    pub fn as_i64(&self) -> i64 {
        self.raw as i64
    }

    pub fn as_serenity_id(&self) -> GuildId {
        self.serenity_guild
    }
}
