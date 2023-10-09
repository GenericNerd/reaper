use serenity::all::CommandInteraction;

use crate::models::{command::CommandContext, handler::Handler, response::ResponseResult};

pub async fn end(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
) -> ResponseResult {
    Ok(())
}
