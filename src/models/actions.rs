use std::sync::atomic::AtomicBool;

use tracing::error;

use crate::{common::duration::Duration, models::response::ResponseError};

use super::{handler::Handler, response::ResponseResult};

#[derive(Clone, PartialEq)]
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

#[derive(Clone)]
pub struct Action {
    pub id: Result<objectid::ObjectId, String>,
    pub action_type: ActionType,
    pub user_id: i64,
    pub moderator_id: i64,
    pub guild_id: i64,
    pub reason: String,
    pub active: bool,
    pub expiry: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
}

impl Action {
    pub fn new(
        action_type: ActionType,
        user_id: i64,
        moderator_id: i64,
        guild_id: i64,
        reason: String,
        expiry: Option<Duration>,
    ) -> Self {
        let active = action_type != ActionType::Kick;
        Action {
            id: Ok(objectid::ObjectId::new().unwrap()),
            action_type,
            user_id,
            moderator_id,
            guild_id,
            reason,
            expiry: expiry.map(|duration| duration.to_timestamp().unwrap()),
            active,
            created_at: time::OffsetDateTime::now_utc(),
        }
    }

    pub fn get_id(&self) -> String {
        match &self.id {
            Ok(oid) => oid.to_string(),
            Err(string) => string.to_string(),
        }
    }

    pub async fn insert(&self, handler: &Handler) -> ResponseResult {
        DatabaseAction::from(self.clone()).insert(handler).await
    }
}

impl From<DatabaseAction> for Action {
    fn from(value: DatabaseAction) -> Self {
        Action {
            id: objectid::ObjectId::with_string(&value.id).map_err(|_| value.id),
            action_type: ActionType::from(value.action_type),
            user_id: value.user_id,
            moderator_id: value.moderator_id,
            guild_id: value.moderator_id,
            reason: value.reason,
            active: value.active,
            expiry: value.expiry.map(time::PrimitiveDateTime::assume_utc),
            created_at: value.created_at.assume_utc(),
        }
    }
}

#[derive(Clone)]
pub struct DatabaseAction {
    pub id: String,
    pub action_type: String,
    pub user_id: i64,
    pub moderator_id: i64,
    pub guild_id: i64,
    pub reason: String,
    pub active: bool,
    pub expiry: Option<time::PrimitiveDateTime>,
    pub created_at: time::PrimitiveDateTime,
}

impl From<Action> for DatabaseAction {
    fn from(value: Action) -> Self {
        DatabaseAction {
            id: match value.id {
                Ok(oid) => oid.to_string(),
                Err(string) => string,
            },
            action_type: value.action_type.to_string(),
            user_id: value.user_id,
            moderator_id: value.moderator_id,
            guild_id: value.guild_id,
            reason: value.reason,
            active: value.active,
            expiry: value
                .expiry
                .map(|offset| time::PrimitiveDateTime::new(offset.date(), offset.time())),
            created_at: time::PrimitiveDateTime::new(
                value.created_at.date(),
                value.created_at.time(),
            ),
        }
    }
}

impl DatabaseAction {
    pub async fn insert(&self, handler: &Handler) -> ResponseResult {
        match sqlx::query!(
            "INSERT INTO actions (id, action_type, user_id, moderator_id, guild_id, reason, active, expiry, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            self.id,
            self.action_type,
            self.user_id,
            self.moderator_id,
            self.guild_id,
            self.reason,
            self.active,
            self.expiry,
            self.created_at
        ).execute(&handler.main_database).await {
            Ok(_) => Ok(()),
            Err(err) => {
                error!("Failed to insert action into database: {}", err);
                Err(ResponseError::Execution(
                    "Failed to insert action into database!",
                    Some("Please contac tthe bot owner for assistance".to_string())
                ))
            }
        }
    }
}

pub struct ActionDatabaseInsert {
    pub action: Action,
    pub dm_notified: AtomicBool,
}

pub struct ActionEscalation {
    pub guild_id: i64,
    pub strike_count: i64,
    pub action_type: ActionType,
    pub action_duration: Option<String>,
}

impl From<DatabaseActionEscalation> for ActionEscalation {
    fn from(value: DatabaseActionEscalation) -> Self {
        ActionEscalation {
            guild_id: value.guild_id,
            strike_count: value.strike_count,
            action_type: ActionType::from(value.action_type),
            action_duration: value.action_duration,
        }
    }
}

pub struct DatabaseActionEscalation {
    pub guild_id: i64,
    pub strike_count: i64,
    pub action_type: String,
    pub action_duration: Option<String>,
}
