use serenity::model::id::ChannelId as SerenityChannelId;

#[derive(Debug, Clone, Copy)]
pub struct Channel {
    raw_id: u64,
    serenity_id: SerenityChannelId,
}

impl From<u64> for Channel {
    fn from(raw_id: u64) -> Self {
        Self {
            raw_id,
            serenity_id: SerenityChannelId::new(raw_id),
        }
    }
}

impl From<SerenityChannelId> for Channel {
    fn from(serenity_id: SerenityChannelId) -> Self {
        Self {
            raw_id: serenity_id.get(),
            serenity_id,
        }
    }
}

impl Channel {
    pub fn as_u64(&self) -> u64 {
        self.raw_id
    }

    pub fn as_i64(&self) -> i64 {
        self.raw_id as i64
    }

    pub fn as_serenity_id(&self) -> SerenityChannelId {
        self.serenity_id
    }
}
