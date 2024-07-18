#![allow(clippy::struct_field_names)]

#[derive(Clone)]
pub struct BoardConfiguration {
    pub channel_id: i64,
    pub emote_quota: i32,
    pub ignore_self_reacts: bool,
}