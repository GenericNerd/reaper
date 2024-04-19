use tracing::{debug, error};

use crate::models::{handler::Handler, permissions::Permission};

struct PermissionRecord {
    permission: String,
}

pub async fn get_user(handler: &Handler, guild_id: i64, user_id: i64) -> Vec<Permission> {
    debug!("Querying main database for user {user_id} permissions in guild {guild_id}");
    let permissions = match sqlx::query_as!(
        PermissionRecord,
        "SELECT permission FROM users WHERE guild_id = $1 AND id = $2",
        guild_id,
        user_id
    )
    .fetch_all(&handler.main_database)
    .await
    {
        Ok(permissions) => permissions,
        Err(err) => {
            error!(
                "Attempted to query main database for user {user_id} permissions in guild {guild_id}, failed with error: {err}",
            );
            return vec![];
        }
    };

    permissions
        .into_iter()
        .map(|p| Permission::from(p.permission.as_str()))
        .collect()
}

pub async fn add_permission_to_user(
    handler: &Handler,
    guild_id: i64,
    user_id: i64,
    permission: &Permission,
) {
    debug!(
        "Querying main database to add permission {} to user {user_id} in guild {guild_id}",
        permission.to_string()
    );
    match sqlx::query!(
        "INSERT INTO users (guild_id, id, permission) VALUES ($1, $2, $3)",
        guild_id,
        user_id,
        permission.to_string()
    )
    .execute(&handler.main_database)
    .await
    {
        Ok(_) => (),
        Err(err) => {
            error!(
                "Attempted to query main database to add permission {} to user {user_id} in guild {guild_id}, failed with error: {err}",
                permission.to_string()
            );
        }
    }
}

pub async fn remove_permission_from_user(
    handler: &Handler,
    guild_id: i64,
    user_id: i64,
    permission: &Permission,
) {
    debug!(
        "Querying main database to remove permission {} from user {user_id} in guild {guild_id}",
        permission.to_string()
    );
    match sqlx::query!(
        "DELETE FROM users WHERE guild_id = $1 AND id = $2 AND permission = $3",
        guild_id,
        user_id,
        permission.to_string()
    )
    .execute(&handler.main_database)
    .await
    {
        Ok(_) => (),
        Err(err) => {
            error!(
                "Attempted to query main database to remove permission {} from user {user_id} in guild {guild_id}, failed with error: {err}",
                permission.to_string()
            );
        }
    }
}

pub async fn get_role(handler: &Handler, guild_id: i64, role_id: i64) -> Vec<Permission> {
    debug!("Querying main database for role {role_id} permissions in guild {guild_id}");
    let permissions = match sqlx::query_as!(
        PermissionRecord,
        "SELECT permission FROM roles WHERE guild_id = $1 AND id = $2",
        guild_id,
        role_id
    )
    .fetch_all(&handler.main_database)
    .await
    {
        Ok(permissions) => permissions,
        Err(err) => {
            error!(
                "Attempted to query main database for role {role_id} permissions in guild {guild_id}, failed with error: {err}",
            );
            return vec![];
        }
    };

    permissions
        .into_iter()
        .map(|p| Permission::from(p.permission.as_str()))
        .collect()
}

pub async fn add_permission_to_role(
    handler: &Handler,
    guild_id: i64,
    role_id: i64,
    permission: &Permission,
) {
    debug!(
        "Querying main database to add permission {} to role {role_id} in guild {guild_id}",
        permission.to_string()
    );
    match sqlx::query!(
        "INSERT INTO roles (guild_id, id, permission) VALUES ($1, $2, $3)",
        guild_id,
        role_id,
        permission.to_string()
    )
    .execute(&handler.main_database)
    .await
    {
        Ok(_) => (),
        Err(err) => {
            error!(
                "Attempted to query main database to add permission {} to role {role_id} in guild {guild_id}, failed with error: {err}",
                permission.to_string()
            );
        }
    }
}

pub async fn remove_permission_from_role(
    handler: &Handler,
    guild_id: i64,
    role_id: i64,
    permission: &Permission,
) {
    debug!(
        "Querying main database to remove permission {} from role {role_id} in guild {guild_id}",
        permission.to_string()
    );
    match sqlx::query!(
        "DELETE FROM roles WHERE guild_id = $1 AND id = $2 AND permission = $3",
        guild_id,
        role_id,
        permission.to_string()
    )
    .execute(&handler.main_database)
    .await
    {
        Ok(_) => (),
        Err(err) => {
            error!(
                "Attempted to query main database to remove permission {} from role {role_id} in guild {guild_id}, failed with error: {err}",
                permission.to_string()
            );
        }
    }
}
