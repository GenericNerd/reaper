use serenity::{
    all::CreateAttachment,
    builder::{CreateActionRow, CreateAllowedMentions, CreateEmbed},
};

use crate::models::permissions::Permission;

pub struct Response {
    pub content: Option<String>,
    pub embeds: Option<Vec<CreateEmbed>>,
    pub allowed_mentions: Option<CreateAllowedMentions>,
    pub components: Option<Vec<CreateActionRow>>,
    pub attachments: Option<CreateAttachment>,
    pub ephemeral: bool,
}

pub trait ReaperError {
    fn title(&self) -> String;
    fn description(&self) -> Option<String>;
}

#[derive(Debug)]
pub enum InternalError {
    GuildOnlyInteraction,
    FailedToObtainContext,
    FailedToFetchGuild,
    FailedToObtainShardLatency,
    InteractionNotFound,
    InteractionsDisabled,
    InteractionDisabled { interaction_name: String },
    UserDisabled,
    GuildDisabled,
    InvalidInteractionType,
    InvalidConfigurationStep,
}

impl ReaperError for InternalError {
    fn title(&self) -> String {
        "Something went wrong!".to_string()
    }

    fn description(&self) -> Option<String> {
        let description = match self {
            InternalError::GuildOnlyInteraction => {
                "This interaction can only be used in a guild".to_string()
            }
            InternalError::FailedToObtainContext => {
                "We failed to obtain the context required to continue this command".to_string()
            }
            InternalError::FailedToFetchGuild => {
                "We couldn't get details about this server".to_string()
            }
            InternalError::FailedToObtainShardLatency => {
                "We couldn't get the latency of the bot".to_string()
            }
            InternalError::InteractionNotFound => "We couldn't find this interaction".to_string(),
            InternalError::InteractionsDisabled => {
                "All interactions are disabled. Please try again later".to_string()
            }
            InternalError::InteractionDisabled { interaction_name } => {
                format!("The interaction `{interaction_name}` is disabled")
            }
            InternalError::UserDisabled => {
                "You are currently disabled. Please try again later".to_string()
            }
            InternalError::GuildDisabled => {
                "This server is disabled. Please try again later".to_string()
            }
            InternalError::InvalidInteractionType => {
                "The interaction type wasn't what we were expecting".to_string()
            }
            InternalError::InvalidConfigurationStep => {
                "We have entered an invalid step during configuration".to_string()
            }
        };

        let mut official = false;
        if let Ok(is_official) = std::env::var("OFFICIAL") {
            official = is_official == "true";
        }

        if official {
            return Some(format!(
                "{description}\n\n*Please reach out to the [Reaper support server](https://discord.gg/jhD3Xc5cm6) if the issue persists.*"
            ));
        }
        Some(description)
    }
}

#[derive(Debug)]
pub enum InputError {
    NoEscalationSelected,
    NoRoleSelected,
    NoChannelSelected,
    NoActionTypeSelected,
    NoEmoteSelected,
    InvalidEscalation,
    InvalidRole { message: String },
    InvalidDuration,
    InvalidNumber,
    InvalidStrikeCount,
    InvalidMinXP,
    InvalidMaxLevel,
    InvalidMultiplierCap,
    InvalidEmote,
    EmoteNotInServer,
    Timeout { duration: String },
    InsufficientPermission { required_permission: Permission },
}

impl ReaperError for InputError {
    fn title(&self) -> String {
        "Something wasn't quite right!".to_string()
    }

    fn description(&self) -> Option<String> {
        Some(match self {
            InputError::NoEscalationSelected => {
                "We don't see any escalation selected. Please try again".to_string()
            }
            InputError::NoRoleSelected => {
                "We don't see any role selected. Please try again".to_string()
            }
            InputError::NoChannelSelected => {
                "We don't see any channel selected. Please try again".to_string()
            }
            InputError::NoActionTypeSelected => {
                "We don't see a selected action type. Please try again".to_string()
            }
            InputError::NoEmoteSelected => {
                "We don't see any emote selected. Please try again".to_string()
            }
            InputError::InvalidEscalation => {
                "The escalation you selected was invalid! Please try again".to_string()
            }
            InputError::InvalidRole { message } => {
                format!("The role you selected was invalid!\n`{message}`")
            }
            InputError::InvalidDuration => {
                "The duration you inputted was invalid! Please try again".to_string()
            }
            InputError::InvalidNumber => {
                "The number you inputted was invalid! Please try again".to_string()
            }
            InputError::InvalidMinXP => {
                "The minimum XP was higher than the maximum XP. Please try again.".to_string()
            }
            InputError::InvalidMaxLevel => {
                "The maximum level you inputted was invalid! Please try again".to_string()
            }
            InputError::InvalidMultiplierCap => {
                "The multiplier cap you inputted was invalid! Please try again".to_string()
            }
            InputError::InvalidEmote => {
                "The emote you selected was invalid! Please try again".to_string()
            }
            InputError::EmoteNotInServer => {
                "The emote you selected is not in this server! Please try again".to_string()
            }
            InputError::Timeout { duration } => {
                format!(
                    "We didn't receive anything from you for {duration}. Feel free to try again"
                )
            }
            InputError::InvalidStrikeCount => {
                "The strike count you inputted was invalid! Please try again".to_string()
            }
            InputError::InsufficientPermission {
                required_permission,
            } => {
                format!(
                    "To use this, you need the {required_permission}. Please contact your server administrators if you believe this to be a mistake"
                )
            }
        })
    }
}

#[derive(Debug)]
pub enum ExecutionError {
    Internal(InternalError),
    Input(InputError),
}

impl ReaperError for ExecutionError {
    fn title(&self) -> String {
        match self {
            ExecutionError::Internal(err) => err.title(),
            ExecutionError::Input(err) => err.title(),
        }
    }

    fn description(&self) -> Option<String> {
        match self {
            ExecutionError::Internal(err) => err.description(),
            ExecutionError::Input(err) => err.description(),
        }
    }
}

#[derive(Debug)]
pub enum ResponseError {
    Serenity(Box<serenity::Error>),
    Sqlx(Box<sqlx::Error>),
    Execution(ExecutionError),
    Redis(Box<redis::RedisError>),
}

impl From<serenity::Error> for ResponseError {
    fn from(value: serenity::Error) -> Self {
        Self::Serenity(Box::new(value))
    }
}

impl From<sqlx::Error> for ResponseError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(Box::new(value))
    }
}

impl From<redis::RedisError> for ResponseError {
    fn from(value: redis::RedisError) -> Self {
        Self::Redis(Box::new(value))
    }
}

pub type ResponseResult = Result<(), ResponseError>;

impl Response {
    pub const fn new() -> Self {
        Response {
            content: None,
            embeds: None,
            allowed_mentions: None,
            components: None,
            attachments: None,
            ephemeral: false,
        }
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn embed(mut self, embed: CreateEmbed) -> Self {
        self.embeds = Some(vec![embed]);
        self
    }

    pub fn components(mut self, components: Vec<CreateActionRow>) -> Self {
        self.components = Some(components);
        self
    }

    pub fn attachments(mut self, attachments: CreateAttachment) -> Self {
        self.attachments = Some(attachments);
        self
    }

    pub const fn ephemeral(mut self, ephemeral: bool) -> Self {
        self.ephemeral = ephemeral;
        self
    }
}
