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
    SerenityError(serenity::Error),
    ExecutionError(&'static str, Option<String>),
}

pub type ResponseResult = Result<(), ResponseError>;

impl Response {
    pub fn new() -> Self {
        Response {
            content: None,
            embeds: None,
            allowed_mentions: None,
            components: None,
            ephemeral: false,
        }
    }

    pub fn content(mut self, content: String) -> Self {
        self.content = Some(content);
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

    pub fn ephemeral(mut self, ephemeral: bool) -> Self {
        self.ephemeral = ephemeral;
        self
    }
}
