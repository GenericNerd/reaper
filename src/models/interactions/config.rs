use serde::{Deserialize, Serialize};
use strum::Display;

use crate::models::actions::ActionEscalation;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ModerationStage {
    Enter,
    Footer,
    ChangeFooter,
    MuteRole,
    SelectedMuteRole,
    DefaultStrikeDuration,
    ChangeDefaultStrikeDuration,
    Escalations {
        escalations: Option<Vec<ActionEscalation>>,
    },
    AddEscalation {
        escalations: Vec<ActionEscalation>,
    },
    RemoveEscalation {
        escalations: Vec<ActionEscalation>,
    },
    SubmitEscalations {
        escalations: Vec<ActionEscalation>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LogCategory {
    Actions,
    Messages,
    Voice,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LoggingStage {
    Enter,
    Categories {
        actions: Option<bool>,
        messages: Option<bool>,
        voice: Option<bool>,
    },
    SubmitCategories {
        actions: bool,
        messages: bool,
        voice: bool,
    },
    OneOrMultiple,
    SingleLogChannel,
    SubmitSingleLogChannel,
    MultipleLogChannels {
        category: LogCategory,
    },
    SubmitMultipleLogChannels {
        category: LogCategory,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Reward {
    pub role: i64,
    pub level: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RoleMultiplier {
    pub role: i64,
    pub multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChannelMultiplier {
    pub channel: i64,
    pub multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum XPStage {
    Enter,
    RandomOrSet,
    SelectedRandomOrSet {
        is_random: bool,
    },
    MessageCooldown,
    ChangeMessageCooldown,
    MaxLevel,
    ChangeMaxLevel {
        is_limited: bool,
    },
    StackRewards,
    ChangeStackRewards {
        is_enabled: bool,
    },
    StackMultipliers,
    ChangeStackMultipliers {
        is_enabled: bool,
    },
    MultiplierCap,
    ChangeMultiplierCap {
        is_limited: bool,
    },
    ResetXpOnLeave,
    ChangeResetXpOnLeave {
        is_enabled: bool,
    },
    LevelUpMessages,
    ChangeLevelUpMessages {
        is_enabled: bool,
    },
    DmOnLevelUp,
    ChangeDmOnLevelUp {
        is_enabled: bool,
    },
    LevelUpChannel,
    ChangeLevelUpChannel,
    LevelUpMessage,
    ChangeLevelUpMessage,
    RewardsEnter,
    Rewards {
        page: usize,
        rewards: Option<Vec<Reward>>,
    },
    AddReward {
        page: usize,
        rewards: Vec<Reward>,
    },
    RemoveReward {
        page: usize,
        rewards: Vec<Reward>,
    },
    SaveRewards {
        rewards: Vec<Reward>,
    },
    RoleMultiplierEnter,
    RoleMultipliers {
        page: usize,
        multipliers: Option<Vec<RoleMultiplier>>,
    },
    AddRoleMultiplier {
        page: usize,
        multipliers: Vec<RoleMultiplier>,
    },
    RemoveRoleMultiplier {
        page: usize,
        multipliers: Vec<RoleMultiplier>,
    },
    SaveRoleMultipliers {
        multipliers: Vec<RoleMultiplier>,
    },
    ChannelMultiplierEnter,
    ChannelMultipliers {
        page: usize,
        multipliers: Option<Vec<ChannelMultiplier>>,
    },
    AddChannelMultiplier {
        page: usize,
        multipliers: Vec<ChannelMultiplier>,
    },
    RemoveChannelMultiplier {
        page: usize,
        multipliers: Vec<ChannelMultiplier>,
    },
    SaveChannelMultipliers {
        multipliers: Vec<ChannelMultiplier>,
    },
    RoleBlacklistEnter,
    RoleBlacklists {
        page: usize,
        blacklists: Option<Vec<i64>>,
    },
    AddRoleBlacklist {
        page: usize,
        blacklists: Vec<i64>,
    },
    RemoveRoleBlacklist {
        page: usize,
        blacklists: Vec<i64>,
    },
    SaveRoleBlacklist {
        blacklists: Vec<i64>,
    },
    ChannelBlacklistEnter,
    ChannelBlacklists {
        page: usize,
        blacklists: Option<Vec<i64>>,
    },
    AddChannelBlacklist {
        page: usize,
        blacklists: Vec<i64>,
    },
    RemoveChannelBlacklist {
        page: usize,
        blacklists: Vec<i64>,
    },
    SaveChannelBlacklist {
        blacklists: Vec<i64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BoardsStage {
    Enter,
    SelectBoardChannel,
    EditSettingsOrEmotes {
        channel_id: Option<i64>,
    },
    Quota {
        channel_id: i64,
    },
    ChangeQuota {
        channel_id: i64,
    },
    IgnoreSelfReacts {
        channel_id: i64,
    },
    ChangeIgnoreSelfReacts {
        channel_id: i64,
        is_ignoring: bool,
    },
    Emotes {
        channel_id: i64,
        emotes: Option<Vec<String>>,
        page: usize,
    },
    AddEmote {
        channel_id: i64,
        emotes: Vec<String>,
        page: usize,
    },
    RemoveEmote {
        channel_id: i64,
        emotes: Vec<String>,
        page: usize,
    },
    SaveEmotes {
        channel_id: i64,
        emotes: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ConfigInteraction {
    Moderation { stage: ModerationStage },
    Logging { stage: LoggingStage },
    Xp { stage: XPStage },
    Boards { stage: BoardsStage },
    Complete,
}
