use std::env;
use mongodb::{Client, options::ClientOptions, bson::doc, Collection};
use crate::mongo::structs;
use tracing::error;

pub struct Database {
    pub client: Client
}

impl Database {
    pub async fn add_user(&self, guild_id: i64, user_id: i64) -> Result<structs::User, structs::MongoError> {
        let collection = self.client.database("reaper").collection("users");
        let user = structs::User {
            guild_id,
            id: user_id,
            permissions: vec![]
        };

        let permisions: Vec<String> = Vec::new();
        match collection.insert_one(doc!{"guildID": user.guild_id, "id": user.id, "permissions": permisions}, None).await {
            Ok(_) => {
                return Ok(user);
            },
            Err(err) => {
                return Err(structs::MongoError {
                    message: "An error occurred while adding the user to the database".to_string(),
                    mongo_error: Some(err)
                });
            }
        }
    }
    pub async fn get_user(&self, guild_id: i64, user_id: i64) -> Result<structs::User, structs::MongoError> {
        let users: Collection<structs::User> = self.client.database("reaper").collection("users");
        match users.find_one(doc!{"guildID": guild_id, "id": user_id}, None).await {
            Ok(user) => {
                match user {
                    Some(user) => {
                        return Ok(user);
                    },
                    None => {
                        return self.add_user(guild_id, user_id).await;
                    }
                }
            },
            Err(err) => {
                error!("An error occurred while retrieving a user from the database. The error was: {}", err);
                Err(structs::MongoError {
                    message: "An error occurred while retrieving a user from the database".to_string(),
                    mongo_error: Some(err)
                })
            }
        }
    }

    pub async fn update_user_permissions(&self, guild_id: i64, user_id: i64, permissions: Vec<structs::Permissions>) -> Result<(), structs::MongoError> {
        let users: Collection<structs::User> = self.client.database("reaper").collection("users");
        let mut permission_strings: Vec<String> = Vec::new();
        for permission in permissions.iter() {
            permission_strings.push(permission.to_string());
        }
        match users.update_one(doc!{"guildID": guild_id, "id": user_id}, doc!{"$set": {"permissions": permission_strings}}, None).await {
            Ok(_) => Ok(()),
            Err(err) => {
                error!("An error occurred while updating a user's permissions in the database. The error was: {}", err);
                Err(structs::MongoError {
                    message: "An error occurred while updating a user's permissions in the database".to_string(),
                    mongo_error: Some(err)
                })
            }
        }
    }
}

pub async fn connect() -> Result<Database, structs::MongoError> {
    let uri = match env::var("MONGO_URI") {
        Ok(uri) => uri,
        Err(err) => {
            error!("An error occurred while retrieving the MONGO_URI environment variable. The error was: {}", err);
            return Err(structs::MongoError {
                message: "An error occurred while retrieving the MONGO_URI environment variable".to_string(),
                mongo_error: None
            });
        }
    };

    let mut client_options = match ClientOptions::parse(&uri).await {
        Ok(client_options) => client_options,
        Err(err) => {
            error!("An error occurred while parsing the MONGO_URI environment variable. The error was: {}", err);
            return Err(structs::MongoError {
                message: "An error occurred while parsing the MONGO_URI environment variable".to_string(),
                mongo_error: Some(err)
            });
        }
    };
    client_options.app_name = Some("Reaper".to_string());

    let client = match Client::with_options(client_options) {
        Ok(client) => client,
        Err(err) => {
            error!("An error occurred while creating a MongoDB client. The error was: {}", err);
            return Err(structs::MongoError {
                message: "An error occurred while creating a MongoDB client".to_string(),
                mongo_error: Some(err)
            });
        }
    };
    
    Ok(Database {
        client
    })
}