use std::sync::atomic::AtomicBool;

#[derive(Clone, sqlx::Type)]
#[sqlx(type_name = "type")]
#[sqlx(rename_all = "lowercase")]
pub enum ActionType {
    Strike,
    Mute,
    Kick,
    Ban,
}

impl ToString for ActionType {
    fn to_string(&self) -> String {
        match self {
            ActionType::Strike => "strike".to_string(),
            ActionType::Mute => "mute".to_string(),
            ActionType::Kick => "kick".to_string(),
            ActionType::Ban => "ban".to_string(),
        }
    }
}

#[derive(Clone, sqlx::FromRow)]
pub struct Action {
    pub id: String,
    #[sqlx(rename = "type")]
    pub action_type: ActionType,
    pub user_id: i64,
    pub moderator_id: i64,
    pub guild_id: i64,
    pub reason: String,
    pub active: bool,
    pub expiry: Option<time::OffsetDateTime>,
}

pub struct ActionInsert {
    pub action: Action,
    pub dm_notified: AtomicBool,
}

pub struct ActionEscalation {
    pub guild_id: i64,
    pub strike_count: i64,
    pub action_type: ActionType,
    pub action_duration: Option<String>,
}
