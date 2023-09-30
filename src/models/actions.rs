use std::sync::atomic::AtomicBool;

#[derive(Clone)]
pub enum ActionType {
    Strike,
    Mute,
    Kick,
    Ban,
}

impl sqlx::Decode<'_, sqlx::Postgres> for ActionType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        match value.as_str().unwrap() {
            "strike" => Ok(ActionType::Strike),
            "mute" => Ok(ActionType::Mute),
            "kick" => Ok(ActionType::Kick),
            "ban" => Ok(ActionType::Ban),
            _ => Err("Invalid action type".into()),
        }
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ActionType {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        match self {
            ActionType::Strike => buf.extend("strike".as_bytes()),
            ActionType::Mute => buf.extend("mute".as_bytes()),
            ActionType::Kick => buf.extend("kick".as_bytes()),
            ActionType::Ban => buf.extend("ban".as_bytes()),
        }

        sqlx::encode::IsNull::No
    }
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

#[derive(Clone)]
pub struct Action {
    pub id: String,
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
