use serenity::builder::{CreateActionRow, CreateAllowedMentions, CreateEmbed};

pub struct Response {
    pub content: Option<String>,
    pub embeds: Option<Vec<CreateEmbed>>,
    pub allowed_mentions: Option<CreateAllowedMentions>,
    pub components: Option<Vec<CreateActionRow>>,
}

impl Response {
    pub fn new() -> Self {
        Response {
            content: None,
            embeds: None,
            allowed_mentions: None,
            components: None,
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

    pub fn embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds = Some(embeds);
        self
    }

    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    pub fn components(mut self, components: Vec<CreateActionRow>) -> Self {
        self.components = Some(components);
        self
    }
}
