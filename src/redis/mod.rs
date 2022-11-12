use redis::{Client as RedisClient, aio::Connection};
use std::{error::Error, env};
pub struct Redis {
    pub connection: Connection
}

pub async fn connect() -> Result<Redis, Box<dyn Error>> {
    let env_var = env::var("REDIS_URI")?;
    let client =  RedisClient::open(env_var)?;
    let conn = client.get_async_connection().await?;

    Ok(Redis { connection: conn })
}