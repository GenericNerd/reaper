use std::sync::atomic::AtomicBool;

use serenity::all::Context as SerenityContext;

use crate::models::{
    permission::Permission,
    response::{ReaperError, ResponseError, ResponseResult},
    serenity::{guild::Guild, user::User},
};

pub struct UnpopulatedContext {
    pub serenity_context: SerenityContext,
}

pub struct PopulatedContext {
    pub serenity_context: SerenityContext,
    pub has_responded: AtomicBool,
    pub user: User,
    pub user_permissions: Vec<Permission>,
    pub highest_role: u16,
    pub guild: Option<Guild>,
}

pub enum Context {
    Unpopulated(UnpopulatedContext),
    Populated(PopulatedContext),
}

impl Context {
    pub fn get_populated_context(&self) -> ResponseResult<&PopulatedContext> {
        match self {
            Context::Populated(ctx) => Ok(ctx),
            Context::Unpopulated(_) => Err(ResponseError::Reaper(ReaperError::UnpopulatedContext)),
        }
    }
}
