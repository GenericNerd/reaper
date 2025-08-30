#[derive(Debug)]
pub struct EventRouter;

mod automod_action_execution;
mod guild_create;
mod guild_leave;
mod interaction_create;
mod member_join;
mod member_leave;
mod on_ready;
mod router;
