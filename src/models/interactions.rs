// TODO: Write logging

use std::sync::Arc;

use dashmap::DashMap;
use sqlx::PgPool;

use crate::models::user::User;

#[derive(Debug, Clone)]
pub struct Interaction {
    pub id: uuid::Uuid,
    pub interaction: String,
    pub action: String,
    pub user_id: User,
    pub data: serde_json::Value,
    pub expires_at: Option<time::OffsetDateTime>,
}

pub struct InteractionBuilder {
    pub interaction: String,
    pub action: String,
    pub user_id: User,
    pub data: serde_json::Value,
    pub expires_at: Option<time::OffsetDateTime>,
}

impl InteractionBuilder {
    pub fn new(
        interaction: String,
        action: String,
        user_id: User,
        data: serde_json::Value,
        expires_at: Option<time::OffsetDateTime>,
    ) -> Self {
        Self {
            interaction,
            action,
            user_id,
            data,
            expires_at,
        }
    }

    pub fn build(&self) -> Interaction {
        Interaction {
            id: uuid::Uuid::new_v4(),
            interaction: self.interaction.clone(),
            action: self.action.clone(),
            user_id: self.user_id,
            data: self.data.clone(),
            expires_at: self.expires_at,
        }
    }

    pub fn one_hour_expiry() -> time::OffsetDateTime {
        time::OffsetDateTime::now_utc() + time::Duration::hours(1)
    }
}

pub struct InteractionState {
    cache: DashMap<uuid::Uuid, Interaction>,
    database: Arc<PgPool>,
}

fn offset_to_primative_datetime(offset: time::OffsetDateTime) -> time::PrimitiveDateTime {
    time::PrimitiveDateTime::new(offset.date(), offset.time())
}

impl InteractionState {
    pub fn new(database: Arc<PgPool>) -> Self {
        Self {
            cache: DashMap::new(),
            database,
        }
    }

    pub async fn register(
        &self,
        interactions: Vec<Interaction>,
    ) -> Result<Vec<uuid::Uuid>, sqlx::Error> {
        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO interaction_states (id, interaction, action, user_id, data, expires_at) ",
        );

        query_builder.push_values(interactions.iter(), |mut b, interaction| {
            b.push_bind(interaction.id)
                .push_bind(interaction.interaction.clone())
                .push_bind(interaction.action.clone())
                .push_bind(interaction.user_id.as_i64())
                .push_bind(interaction.data.clone());
            if let Some(expires_at) = interaction.expires_at {
                b.push_bind(offset_to_primative_datetime(expires_at));
            } else {
                b.push_bind(None::<time::PrimitiveDateTime>);
            }
        });

        query_builder
            .build()
            .execute(self.database.as_ref())
            .await?;

        for interaction in &interactions {
            self.cache.insert(interaction.id, interaction.clone());
        }

        Ok(interactions
            .iter()
            .map(|interaction| interaction.id)
            .collect())
    }

    pub async fn get(&self, id: uuid::Uuid) -> Option<Interaction> {
        #[derive(sqlx::FromRow)]
        struct InteractionRow {
            id: uuid::Uuid,
            interaction: String,
            action: String,
            user_id: i64,
            data: serde_json::Value,
            expires_at: Option<time::PrimitiveDateTime>,
        }

        if let Some(interaction) = self.cache.get(&id) {
            return Some(interaction.clone());
        }

        let row = sqlx::query_as!(
            InteractionRow,
            "SELECT id, interaction, action, user_id, data, expires_at FROM interaction_states WHERE id = $1",
            id
        )
        .fetch_optional(self.database.as_ref())
        .await
        .ok()??;

        let interaction = Interaction {
            id: row.id,
            interaction: row.interaction,
            action: row.action,
            user_id: User::from(row.user_id as u64),
            data: row.data,
            expires_at: row
                .expires_at
                .map(|dt| dt.assume_offset(time::UtcOffset::UTC)),
        };

        self.cache.insert(interaction.id, interaction.clone());

        Some(interaction)
    }

    pub async fn cleanup(&self) {
        let now = time::OffsetDateTime::now_utc();
        self.cache.retain(|_, interaction| {
            interaction.expires_at.is_none() || interaction.expires_at.unwrap() > now
        });

        sqlx::query!(
            "DELETE FROM interaction_states WHERE expires_at < $1",
            offset_to_primative_datetime(now)
        )
        .execute(self.database.as_ref())
        .await
        .ok();
    }
}
