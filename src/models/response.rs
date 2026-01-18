use std::fmt::Display;

pub enum ReaperError {
    UnpopulatedContext,
    FailedToObtainShardLatency,
    CommandNotFound,
    ComponentNotFound,
    ModalNotFound,
}

impl Display for ReaperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReaperError::UnpopulatedContext => write!(f, "Unpopulated context"),
            ReaperError::FailedToObtainShardLatency => write!(f, "Failed to obtain shard latency"),
            ReaperError::CommandNotFound => write!(f, "Command not found"),
            ReaperError::ComponentNotFound => write!(f, "Component not found"),
            ReaperError::ModalNotFound => write!(f, "Modal not found"),
        }
    }
}

pub enum ResponseError {
    Reaper(ReaperError),
    Serenity(serenity::Error),
    Diesel(diesel::result::Error),
    Json(serde_json::Error),
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseError::Reaper(err) => write!(f, "{err}"),
            ResponseError::Serenity(err) => write!(f, "{err}"),
            ResponseError::Diesel(err) => write!(f, "{err}"),
            ResponseError::Json(err) => write!(f, "{err}"),
        }
    }
}

impl From<ReaperError> for ResponseError {
    fn from(err: ReaperError) -> Self {
        ResponseError::Reaper(err)
    }
}

impl From<serenity::Error> for ResponseError {
    fn from(err: serenity::Error) -> Self {
        ResponseError::Serenity(err)
    }
}

impl From<diesel::result::Error> for ResponseError {
    fn from(err: diesel::result::Error) -> Self {
        ResponseError::Diesel(err)
    }
}

impl From<serde_json::Error> for ResponseError {
    fn from(err: serde_json::Error) -> Self {
        ResponseError::Json(err)
    }
}

pub type ResponseResult<T> = Result<T, ResponseError>;
