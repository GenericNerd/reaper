use std::{
    fmt::{self, Display, Formatter},
    sync::atomic::AtomicBool,
};

use tracing::error;

use crate::{common::duration::Duration, models::response::ResponseError};

use super::{handler::Handler, response::ResponseResult};

#[derive(Copy, Clone, PartialEq)]
pub enum ActionType {
    Strike,
    Mute,
    Kick,
    Ban,
}

impl Display for ActionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ActionType::Strike => write!(f, "strike"),
            ActionType::Mute => write!(f, "mute"),
            ActionType::Kick => write!(f, "kick"),
            ActionType::Ban => write!(f, "ban"),
        }
    }
}

impl From<&str> for ActionType {
    fn from(value: &str) -> Self {
        match value {
            "strike" => ActionType::Strike,
            "mute" => ActionType::Mute,
            "kick" => ActionType::Kick,
            "ban" => ActionType::Ban,
            _ => panic!("Invalid action type"),
        }
    }
}

impl From<String> for ActionType {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

#[derive(Clone)]
pub struct Action {
    pub id: Result<objectid::ObjectId, String>,
    pub typ: ActionType,
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
        typ: ActionType,
        user_id: i64,
        moderator_id: i64,
        guild_id: i64,
        reason: String,
        expiry: Option<Duration>,
    ) -> Self {
        let active = typ != ActionType::Kick;
        Action {
            id: Ok(objectid::ObjectId::new().unwrap()),
            typ,
            user_id,
            moderator_id,
            guild_id,
            reason,
            expiry: match expiry {
                Some(duration) => duration.to_timestamp(),
                None => None,
            },
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
            typ: ActionType::from(value.action_type.as_str()),
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
            action_type: value.typ.to_string(),
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

#[derive(Clone)]
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
            action_type: ActionType::from(value.action_type.as_str()),
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
