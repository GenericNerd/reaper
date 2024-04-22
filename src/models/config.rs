#![allow(clippy::struct_field_names)]
pub struct ModerationConfig {
    pub mute_role: Option<i64>,
    pub default_strike_duration: Option<String>,
}

pub struct LoggingConfig {
    pub log_actions: bool,
    pub log_messages: bool,
    pub log_voice: bool,
    pub log_channel: Option<i64>,
    pub log_action_channel: Option<i64>,
    pub log_message_channel: Option<i64>,
    pub log_voice_channel: Option<i64>,
}
