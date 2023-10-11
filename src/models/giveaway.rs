#[derive(Clone)]
pub struct Giveaway {
    pub id: i64,
    pub channel_id: i64,
    pub prize: String,
    pub description: Option<String>,
    pub winners: i32,
    pub duration: time::OffsetDateTime,
    pub role_restriction: Option<i64>,
}

impl From<DatabaseGiveaway> for Giveaway {
    fn from(value: DatabaseGiveaway) -> Self {
        Giveaway {
            id: value.id,
            channel_id: value.id,
            prize: value.prize,
            description: value.description,
            winners: value.winners,
            duration: value.duration.assume_utc(),
            role_restriction: value.role_restriction,
        }
    }
}

pub struct DatabaseGiveaway {
    pub id: i64,
    pub channel_id: i64,
    pub prize: String,
    pub description: Option<String>,
    pub winners: i32,
    pub duration: time::PrimitiveDateTime,
    pub role_restriction: Option<i64>,
}

impl From<Giveaway> for DatabaseGiveaway {
    fn from(value: Giveaway) -> Self {
        DatabaseGiveaway {
            id: value.id,
            channel_id: value.id,
            prize: value.prize,
            description: value.description,
            winners: value.winners,
            duration: time::PrimitiveDateTime::new(value.duration.date(), value.duration.time()),
            role_restriction: value.role_restriction,
        }
    }
}
