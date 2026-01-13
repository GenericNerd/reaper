use std::sync::atomic::AtomicBool;

use serenity::all::Context as SerenityContext;

use crate::models::{permission::Permission, serenity::user::User};

pub struct UnpopulatedContext {
    pub serenity_context: SerenityContext,
}

pub struct PopulatedContext {
    pub serenity_context: SerenityContext,
    pub has_responded: AtomicBool,
    pub user: User,
    pub user_permissions: Vec<Permission>,
}

pub enum Context {
    Unpopulated(UnpopulatedContext),
    Populated(PopulatedContext),
}
