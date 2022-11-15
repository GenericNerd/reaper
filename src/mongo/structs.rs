use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};
use mongodb::error::Error;
use tracing::warn;

pub struct MongoError {
    pub message: String,
    pub mongo_error: Option<Error>
}

impl std::fmt::Display for MongoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Permissions {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "permissions.view")]
    PermissionsView,
    #[serde(rename = "permissions.add")]
    PermissionsAdd,
    #[serde(rename = "permissions.remove")]
    PermissionsRemove,
    #[serde(rename = "permissions.list")]
    PermissionsList,
    #[serde(rename = "moderation.strike")]
    ModerationStrike,
    #[serde(rename = "moderation.mute")]
    ModerationMute,
    #[serde(rename = "moderation.kick")]
    ModerationKick,
    #[serde(rename = "moderation.ban")]
    ModerationBan,
    #[serde(rename = "moderation.search.self")]
    ModerationSearchSelf,
    #[serde(rename = "moderation.search.self.expired")]
    ModerationSearchSelfExpired,
    #[serde(rename = "moderation.search.others")]
    ModerationSearchOthers,
    #[serde(rename = "moderation.search.others.expired")]
    ModerationSearchOthersExpired,
}

impl PartialEq for Permissions {
    fn eq(&self, other: &Self) -> bool {
        return self.to_string() == other.to_string();
    }
}

impl From<String> for Permissions {
    fn from(s: String) -> Self {
        match s.as_str() {
            "permissions.view" => Permissions::PermissionsView,
            "permissions.add" => Permissions::PermissionsAdd,
            "permissions.remove" => Permissions::PermissionsRemove,
            "permissions.list" => Permissions::PermissionsList,
            "moderation.strike" => Permissions::ModerationStrike,
            "moderation.mute" => Permissions::ModerationMute,
            "moderation.kick" => Permissions::ModerationKick,
            "moderation.ban" => Permissions::ModerationBan,
            "moderation.search.self" => Permissions::ModerationSearchSelf,
            "moderation.search.self.expired" => Permissions::ModerationSearchSelfExpired,
            "moderation.search.others" => Permissions::ModerationSearchOthers,
            "moderation.search.others.expired" => Permissions::ModerationSearchOthersExpired,
            _ => {
                warn!("Unknown permission: {}", s);
                Permissions::Unknown
            }
        }
    }
}

impl ToString for Permissions {
    fn to_string(&self) -> String {
        match self {
            Permissions::Unknown => "unknown".to_string(),
            Permissions::PermissionsView => "permissions.view".to_string(),
            Permissions::PermissionsAdd => "permissions.add".to_string(),
            Permissions::PermissionsRemove => "permissions.remove".to_string(),
            Permissions::PermissionsList => "permissions.list".to_string(),
            Permissions::ModerationStrike => "moderation.strike".to_string(),
            Permissions::ModerationMute => "moderation.mute".to_string(),
            Permissions::ModerationKick => "moderation.kick".to_string(),
            Permissions::ModerationBan => "moderation.ban".to_string(),
            Permissions::ModerationSearchSelf => "moderation.search.self".to_string(),
            Permissions::ModerationSearchSelfExpired => "moderation.search.self.expired".to_string(),
            Permissions::ModerationSearchOthers => "moderation.search.others".to_string(),
            Permissions::ModerationSearchOthersExpired => "moderation.search.others.expired".to_string(),
        }
    }
}

impl Permissions {
    pub fn variants() -> Vec<Permissions> {
        return vec![
            Permissions::PermissionsView,
            Permissions::PermissionsAdd,
            Permissions::PermissionsRemove,
            Permissions::PermissionsList,
            Permissions::ModerationStrike,
            Permissions::ModerationMute,
            Permissions::ModerationKick,
            Permissions::ModerationBan,
            Permissions::ModerationSearchSelf,
            Permissions::ModerationSearchSelfExpired,
            Permissions::ModerationSearchOthers,
            Permissions::ModerationSearchOthersExpired,
        ];
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "guildID")]
    pub guild_id: i64,
    pub id: i64,
    pub permissions: Vec<Permissions>
}

impl User {
    pub fn has_permission(&self, permission: Permissions) -> bool {
        self.permissions.contains(&permission)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    #[serde(rename = "guildID")]
    pub guild_id: i64,
    pub id: i64,
    pub permissions: Vec<Permissions>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingConfig {
    #[serde(rename = "channelID")]
    pub channel_id: i64
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationConfig {
    pub strike_escalations: HashMap<u64, StrikeEscalation>,
    pub mute_role: i64
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuildConfig {
    pub logging: Option<LoggingConfig>,
    pub moderation: Option<ModerationConfig>,
    pub notify_missing_permissions: bool
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Guild {
    pub id: i64,
    pub config: GuildConfig
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ActionType {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "strike")]
    Strike,
    #[serde(rename = "mute")]
    Mute,
    #[serde(rename = "kick")]
    Kick,
    #[serde(rename = "ban")]
    Ban
}

impl PartialEq for ActionType {
    fn eq(&self, other: &Self) -> bool {
        return self.to_string() == other.to_string();
    }
}

impl From<String> for ActionType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "strike" => ActionType::Strike,
            "mute" => ActionType::Mute,
            "kick" => ActionType::Kick,
            "ban" => ActionType::Ban,
            _ => {
                warn!("Unknown action type: {}", s);
                ActionType::Unknown
            }
        }
    }
}

impl ToString for ActionType {
    fn to_string(&self) -> String {
        match self {
            ActionType::Unknown => "unknown".to_string(),
            ActionType::Strike => "strike".to_string(),
            ActionType::Mute => "mute".to_string(),
            ActionType::Kick => "kick".to_string(),
            ActionType::Ban => "ban".to_string()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrikeEscalation {
    pub action_type: ActionType,
    pub duration: Option<String>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    #[serde(rename = "_id")]
    pub uuid: mongodb::bson::oid::ObjectId,
    pub action_type: ActionType,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "guildID")]
    pub guild_id: i64,
    #[serde(rename = "moderatorID")]
    pub moderator_id: i64,
    pub created_at: i64,
    pub reason: String,
    pub active: bool,
    pub expiry: Option<i64>
}