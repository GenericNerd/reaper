#[derive(Clone)]
pub struct BoardConfiguration {
    pub guild_id: i64,
    pub channel_id: i64,
    pub emote_quota: i32,
    pub ignore_self_reacts: bool,
}

#[derive(Clone)]
pub struct BoardEmote {
    pub guild_id: i64,
    pub channel_id: i64,
    pub emote: String,
}

#[derive(Clone)]
pub struct BoardIgnoredChannel {
    pub guild_id: i64,
    pub channel_id: i64,
    pub ignored_channel: i64,
}

#[derive(Clone)]
pub struct BoardEntry {
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_id: i64,
}
