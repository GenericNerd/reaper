use serenity::prelude::SerenityError;

pub struct CommandError {
    pub message: String,
    pub command_error: Option<SerenityError>
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}