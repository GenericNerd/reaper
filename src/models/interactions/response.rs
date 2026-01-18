use serenity::{
    all::CreateAttachment,
    builder::{CreateActionRow, CreateAllowedMentions, CreateEmbed},
};

pub struct Response {
    pub content: Option<String>,
    pub embeds: Option<Vec<CreateEmbed>>,
    pub allowed_mentions: Option<CreateAllowedMentions>,
    pub components: Option<Vec<CreateActionRow>>,
    pub attachments: Option<CreateAttachment>,
    pub ephemeral: bool,
}

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
