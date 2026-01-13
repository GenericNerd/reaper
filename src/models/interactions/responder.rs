use async_trait::async_trait;
use serenity::all::{CommandInteraction, ComponentInteraction, ModalInteraction};

#[async_trait]
pub trait Responder: Send + Sync {}

impl Responder for CommandInteraction {}
impl Responder for ComponentInteraction {}
impl Responder for ModalInteraction {}
