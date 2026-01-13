use std::fmt::Display;

pub enum ReaperError {
    CommandNotFound,
    ComponentNotFound,
    ModalNotFound,
}

impl Display for ReaperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReaperError::CommandNotFound => write!(f, "Command not found"),
            ReaperError::ComponentNotFound => write!(f, "Component not found"),
            ReaperError::ModalNotFound => write!(f, "Modal not found"),
        }
    }
}

pub enum ResponseError {
    Reaper(ReaperError),
    Serenity(serenity::Error),
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseError::Reaper(err) => write!(f, "{err}"),
            ResponseError::Serenity(err) => write!(f, "{err}"),
        }
    }
}

pub type ResponseResult<T> = Result<T, ResponseError>;
