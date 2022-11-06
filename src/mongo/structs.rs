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
    #[serde(rename = "moderation.kick")]
    ModerationKick,
}

impl PartialEq for Permissions {
    fn eq(&self, other: &Self) -> bool {
        return self == other;
    }
}

impl From<String> for Permissions {
    fn from(s: String) -> Self {
        match s.as_str() {
            "permissions.view" => Permissions::PermissionsView,
            "permissions.add" => Permissions::PermissionsAdd,
            "permissions.remove" => Permissions::PermissionsRemove,
            "permissions.list" => Permissions::PermissionsList,
            "moderation.kick" => Permissions::ModerationKick,
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
            Permissions::ModerationKick => "moderation.kick".to_string()
        }
    }
}

impl Permissions {
    pub fn variants() -> Vec<Permissions> {
        return vec![Permissions::PermissionsView, Permissions::PermissionsAdd, Permissions::PermissionsRemove, Permissions::PermissionsList, Permissions::ModerationKick];
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