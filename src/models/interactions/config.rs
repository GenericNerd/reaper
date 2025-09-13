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

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum XPStage {
    Enter,
    RandomOrSet,
    SelectedRandomOrSet { is_random: bool },
    MessageCooldown,
    ChangeMessageCooldown,
    MaxLevel,
    ChangeMaxLevel { is_limited: bool },
    StackRewards,
    ChangeStackRewards { is_enabled: bool },
    StackMultipliers,
    ChangeStackMultipliers { is_enabled: bool },
    MultiplierCap,
    ChangeMultiplierCap { is_limited: bool },
    ResetXpOnLeave,
    ChangeResetXPOnLeave { is_enabled: bool },
    LevelUpMessages,
    ChangeLevelUpMessages { is_enabled: bool },
    DmOnLevelUp,
    ChangeDmOnLevelUp { is_enabled: bool },
    LevelUpChannel,
    ChangeLevelUpChannel,
    LevelUpMessage,
    ChangeLevelUpMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ConfigInteraction {
    Moderation { stage: ModerationStage },
    Logging { stage: LoggingStage },
    XP { stage: XPStage },
    Complete,
}
