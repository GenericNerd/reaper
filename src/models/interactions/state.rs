use std::collections::HashSet;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use diesel::{
    ExpressionMethods, OptionalExtension, QueryDsl, Queryable, RunQueryDsl, Selectable, dsl::now,
    prelude::Insertable, upsert::excluded,
};
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    models::{bot::Bot, response::ResponseResult},
    schema::interaction_state,
};

pub struct InteractionStateBuilder {
    handler_id: String,
    state: serde_json::Value,
    expires_at: Option<DateTime<Utc>>,
}

impl InteractionStateBuilder {
    pub fn new(
        handler_id: String,
        state: impl Serialize + DeserializeOwned,
        expires_at: Option<DateTime<Utc>>,
    ) -> ResponseResult<Self> {
        Ok(Self {
            handler_id,
            state: serde_json::to_value(state)?,
            expires_at,
        })
    }
}

#[derive(Debug, Clone, Insertable, Selectable, Queryable)]
#[diesel(table_name = interaction_state)]
pub struct InteractionState {
    pub id: Uuid,
    pub handler_id: String,
    pub state: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default)]
pub struct StateStore {
    hot: DashMap<Uuid, InteractionState>,
    dirty: RwLock<HashSet<Uuid>>,
}

impl StateStore {
    pub async fn store(&mut self, state: InteractionStateBuilder) -> ResponseResult<Uuid> {
        let id = Uuid::new_v4();
        let state = InteractionState {
            id,
            handler_id: state.handler_id,
            state: state.state,
            created_at: Utc::now(),
            expires_at: state.expires_at,
        };
        self.hot.insert(id, state);
        self.dirty.write().await.insert(id);
        Ok(id)
    }

    pub async fn get(&self, id: Uuid) -> Option<InteractionState> {
        if let Some(state) = self.hot.get(&id) {
            return Some(state.clone());
        }

        let state = {
            use crate::schema::interaction_state::dsl::*;
            interaction_state
                .find(id)
                .first::<InteractionState>(&mut Bot::instance().postgres())
                .optional()
        };

        if let Ok(Some(state)) = state {
            self.hot.insert(id, state.clone());
            return Some(state);
        }

        None
    }

    pub async fn persist(&self) -> ResponseResult<()> {
        let dirty_uuids = {
            let mut guard = self.dirty.write().await;
            std::mem::take(&mut *guard)
        };

        if dirty_uuids.is_empty() {
            return Ok(());
        }

        let states = dirty_uuids
            .iter()
            .filter_map(|uuid| self.hot.get(uuid).map(|s| s.clone()))
            .collect::<Vec<_>>();

        diesel::insert_into(interaction_state::table)
            .values(&states)
            .on_conflict(interaction_state::id)
            .do_update()
            .set((
                interaction_state::state.eq(excluded(interaction_state::state)),
                interaction_state::expires_at.eq(excluded(interaction_state::expires_at)),
            ))
            .execute(&mut Bot::instance().postgres())?;

        Ok(())
    }

    pub async fn cleanup(&self) -> ResponseResult<usize> {
        Ok(diesel::delete(interaction_state::table)
            .filter(interaction_state::expires_at.lt(now))
            .execute(&mut Bot::instance().postgres())?)
    }
}
