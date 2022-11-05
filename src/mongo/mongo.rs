use std::env;
use mongodb::{Client, options::ClientOptions, bson::doc, error::Error, Collection};
use crate::mongo::structs;
use tracing::error;

pub struct Database {
    pub client: Client
}

impl Database {
    pub async fn get_user(&self, guild_id: i64, user_id: i64) -> Result<Option<structs::User>, Error> {
        let users: Collection<structs::User> = self.client.database("reaper").collection("users");
        match users.find_one(doc!{"guildID": guild_id, "id": user_id}, None).await {
            Ok(user) => Ok(user),
            Err(err) => {
                error!("An error occurred while retrieving a user from the database. The error was: {}", err);
                Err(err)
            }
        }
    }
}

pub async fn connect() -> Result<Database, Error> {
    let uri = match env::var("MONGO_URI") {
        Ok(uri) => uri,
        Err(err) => {
            panic!("A uri could not be retrieved from the environment. The error was: {}", err);
        }
    };

    let mut client_options = match ClientOptions::parse(&uri).await {
        Ok(client_options) => client_options,
        Err(err) => {
            panic!("Client options could not be parsed. The error was: {}", err);
        }
    };
    client_options.app_name = Some("Reaper".to_string());

    let client = match Client::with_options(client_options) {
        Ok(client) => client,
        Err(err) => {
            panic!("A client could not be created. The error was: {}", err);
        }
    };
    
    Ok(Database {
        client
    })
}