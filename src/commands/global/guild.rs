use serenity::all::CommandInteraction;

use crate::models::{command::CommandContext, handler::Handler, response::ResponseResult};

pub async fn router(
    _handler: &Handler,
    _ctx: &CommandContext,
    _cmd: &CommandInteraction,
) -> ResponseResult {
    Ok(())
}
