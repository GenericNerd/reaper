use serenity::all::CommandInteraction;
use tracing::info;

use crate::{
    common::options::Options,
    models::{command::CommandContext, handler::Handler, response::ResponseResult},
};

pub async fn router(
    handler: &Handler,
    ctx: &CommandContext,
    cmd: &CommandInteraction,
) -> ResponseResult {
    let options = Options {
        options: cmd.data.options(),
    };

    info!("{:?}", options.get_string("feature_string"));

    Ok(())
}
