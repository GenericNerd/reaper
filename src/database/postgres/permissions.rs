use crate::models::{handler::Handler, permissions::Permission};

struct PermissionRecord {
    permission: String,
}

pub async fn get_user_permissions(
    handler: &Handler,
    guild_id: i64,
    user_id: i64,
) -> Vec<Permission> {
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
        Err(_) => return vec![],
    };
    permissions
        .into_iter()
        .map(|p| Permission::from(p.permission))
        .collect()
}

pub async fn get_role_permissions(
    handler: &Handler,
    guild_id: i64,
    role_id: i64,
) -> Vec<Permission> {
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
        Err(_) => return vec![],
    };
    permissions
        .into_iter()
        .map(|p| Permission::from(p.permission))
        .collect()
}
