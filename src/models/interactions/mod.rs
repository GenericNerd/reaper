use std::sync::Arc;

use dashmap::DashMap;
use metrics::{counter, histogram};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::Display;
use tracing::{debug, warn};

use crate::models::user::User;

pub mod config;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum InteractionKind {
    Config {
        category: config::ConfigInteraction,
        single_category: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: uuid::Uuid,
    pub route: String,
    pub kind: InteractionKind,
    pub user_id: User,
    pub expires_at: Option<time::OffsetDateTime>,
}

pub struct InteractionBuilder {
    pub kind: InteractionKind,
    pub user_id: User,
    pub expires_at: Option<time::OffsetDateTime>,
}

impl InteractionBuilder {
    pub fn new(
        kind: InteractionKind,
        user_id: User,
        expires_at: Option<time::OffsetDateTime>,
    ) -> Self {
        Self {
            kind,
            user_id,
            expires_at,
        }
    }

    pub fn build(&self) -> Interaction {
        Interaction {
            id: uuid::Uuid::new_v4(),
            route: self.kind.to_string(),
            kind: self.kind.clone(),
            user_id: self.user_id,
            expires_at: self.expires_at,
        }
    }

    pub fn one_hour_expiry() -> time::OffsetDateTime {
        time::OffsetDateTime::now_utc() + time::Duration::hours(1)
    }
}

#[derive(Debug)]
pub struct InteractionState {
    cache: DashMap<uuid::Uuid, Interaction>,
    database: Arc<PgPool>,
}

fn offset_to_primitive_datetime(offset: time::OffsetDateTime) -> time::PrimitiveDateTime {
    time::PrimitiveDateTime::new(offset.date(), offset.time())
}

impl InteractionState {
    pub fn new(database: Arc<PgPool>) -> Self {
        Self {
            cache: DashMap::new(),
            database,
        }
    }

    #[tracing::instrument]
    pub async fn register(
        &self,
        interactions: Vec<Interaction>,
    ) -> Result<Vec<uuid::Uuid>, sqlx::Error> {
        debug!("Registering {} interactions", interactions.len());
        counter!("bot.interaction.register_count").increment(interactions.len() as u64);

        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO interaction_states (id, interaction_route, user_id, data, expires_at) ",
        );

        let interaction_db_objs = interactions
            .iter()
            .map(|interaction| {
                let Ok(data) = serde_json::to_value(&interaction.kind) else {
                    return Err(sqlx::Error::from(std::io::Error::other(
                        "Failed to serialize interaction kind",
                    )));
                };

                Ok((
                    interaction.id,
                    interaction.kind.to_string(),
                    interaction.user_id.as_i64(),
                    data,
                    interaction.expires_at.map(offset_to_primitive_datetime),
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        query_builder.push_values(interaction_db_objs, |mut b, interaction| {
            b.push_bind(interaction.0)
                .push_bind(interaction.1)
                .push_bind(interaction.2)
                .push_bind(interaction.3)
                .push_bind(interaction.4);
        });

        query_builder
            .build()
            .execute(self.database.as_ref())
            .await?;

        debug!("Activating interactions in local cache");
        for interaction in &interactions {
            self.cache.insert(interaction.id, interaction.clone());
        }

        Ok(interactions
            .iter()
            .map(|interaction| interaction.id)
            .collect())
    }

    #[tracing::instrument]
    pub async fn get(&self, id: uuid::Uuid) -> Option<Interaction> {
        #[derive(sqlx::FromRow)]
        struct InteractionRow {
            id: uuid::Uuid,
            interaction_route: String,
            user_id: i64,
            data: serde_json::Value,
            expires_at: Option<time::PrimitiveDateTime>,
        }

        let timing = histogram!("bot.interaction.get_duration");
        let start = std::time::Instant::now();

        if let Some(interaction) = self.cache.get(&id) {
            debug!("Interaction was obtained using local cache");
            timing.record(start.elapsed());
            return Some(interaction.clone());
        }

        let row = sqlx::query_as!(
            InteractionRow,
            "SELECT id, interaction_route, user_id, data, expires_at FROM interaction_states WHERE id = $1",
            id
        )
        .fetch_optional(self.database.as_ref())
        .await
        .ok()??;

        let Ok(kind) = serde_json::from_value(row.data) else {
            counter!("bot.interaction.malformed", "id" => id.to_string()).increment(1);
            warn!("Interaction {id} was malformed");
            return None;
        };

        let interaction = Interaction {
            id: row.id,
            route: row.interaction_route,
            kind,
            user_id: User::from(row.user_id as u64),
            expires_at: row
                .expires_at
                .map(|dt| dt.assume_offset(time::UtcOffset::UTC)),
        };

        self.cache.insert(interaction.id, interaction.clone());

        timing.record(start.elapsed());
        Some(interaction)
    }

    pub async fn cleanup(&self) {
        let now = time::OffsetDateTime::now_utc();
        self.cache.retain(|_, interaction| {
            interaction.expires_at.is_none() || interaction.expires_at.unwrap() > now
        });

        sqlx::query!(
            "DELETE FROM interaction_states WHERE expires_at < $1",
            offset_to_primitive_datetime(now)
        )
        .execute(self.database.as_ref())
        .await
        .ok();
    }
}
