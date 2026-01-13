use serenity::all::ChannelId;

pub struct Channel {
    serenity_channel: ChannelId,
    raw: u64,
}

impl From<ChannelId> for Channel {
    fn from(serenity_channel: ChannelId) -> Self {
        Channel {
            serenity_channel,
            raw: serenity_channel.get(),
        }
    }
}

impl From<u64> for Channel {
    fn from(raw: u64) -> Self {
        Channel {
            serenity_channel: ChannelId::new(raw),
            raw,
        }
    }
}

impl Channel {
    pub fn as_u64(&self) -> u64 {
        self.raw
    }

    pub fn as_serenity_id(&self) -> ChannelId {
        self.serenity_channel
    }
}
