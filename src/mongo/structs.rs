use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Permissions {
    #[serde(rename = "moderation.kick")]
    ModerationKick
}

impl PartialEq for Permissions {
    fn eq(&self, other: &Self) -> bool {
        return self == other;
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