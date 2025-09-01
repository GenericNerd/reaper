use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Copy, Clone, PartialEq, Display, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Strike,
    Mute,
    Kick,
    Ban,
}

impl From<String> for ActionType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "strike" => ActionType::Strike,
            "mute" => ActionType::Mute,
            "kick" => ActionType::Kick,
            "ban" => ActionType::Ban,
            _ => panic!("Invalid action type"),
        }
    }
}

impl From<&str> for ActionType {
    fn from(value: &str) -> Self {
        ActionType::from(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionEscalation {
    pub strike_count: i32,
    pub action_type: ActionType,
    pub action_duration: Option<String>,
}
