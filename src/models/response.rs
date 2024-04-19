use serenity::builder::{CreateActionRow, CreateAllowedMentions, CreateEmbed};

pub struct Response {
    pub content: Option<String>,
    pub embeds: Option<Vec<CreateEmbed>>,
    pub allowed_mentions: Option<CreateAllowedMentions>,
    pub components: Option<Vec<CreateActionRow>>,
    pub ephemeral: bool,
}

#[derive(Debug)]
pub enum ResponseError {
    Serenity(serenity::Error),
    Execution(&'static str, Option<String>),
    Redis(redis::RedisError),
}

pub type ResponseResult = Result<(), ResponseError>;

impl Response {
    pub const fn new() -> Self {
        Response {
            content: None,
            embeds: None,
            allowed_mentions: None,
            components: None,
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

    pub const fn components(mut self, components: Vec<CreateActionRow>) -> Self {
        self.components = Some(components);
        self
    }

    pub const fn ephemeral(mut self, ephemeral: bool) -> Self {
        self.ephemeral = ephemeral;
        self
    }
}
