use serde::{Deserialize, Serialize};
use strum::Display;

use crate::models::actions::ActionEscalation;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ModerationStage {
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
pub enum LoggingStage {
    Enter,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ConfigInteraction {
    Moderation { stage: ModerationStage },
    Logging { stage: LoggingStage },
}
