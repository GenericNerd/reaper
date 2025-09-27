use serenity::model::id::ChannelId as SerenityChannelId;

#[derive(Debug, Clone, Copy)]
pub struct Channel {
    raw_id: u64,
}

impl From<u64> for Channel {
    fn from(raw_id: u64) -> Self {
        Self { raw_id }
    }
}

impl From<i64> for Channel {
    fn from(raw_id: i64) -> Self {
        Self {
            raw_id: raw_id as u64,
        }
    }
}

impl From<SerenityChannelId> for Channel {
    fn from(serenity_id: SerenityChannelId) -> Self {
        Self {
            raw_id: serenity_id.get(),
        }
    }
}

impl Channel {
    pub fn as_i64(self) -> i64 {
        self.raw_id as i64
    }
}
