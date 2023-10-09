use serenity::all::CommandInteraction;

use crate::models::{command::CommandContext, handler::Handler, response::ResponseResult};

pub async fn reroll(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
) -> ResponseResult {
    Ok(())
}
