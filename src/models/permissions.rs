use std::fmt::{self, Display, Formatter};

use enum_iterator::Sequence;
use tracing::{debug, error};

use crate::models::{bot::Bot, guild::Guild, role::Role, user::User};

#[derive(Debug, Copy, Clone, PartialEq, Sequence)]
pub enum Permission {
    PermissionsView,
    PermissionsEdit,
    ConfigEdit,
    ModerationStrike,
    ModerationSearchSelf,
    ModerationSearchSelfExpired,
    ModerationSearchOthers,
    ModerationSearchOthersExpired,
    ModerationSearchUuid,
    ModerationMute,
    ModerationUnmute,
    ModerationKick,
    ModerationBan,
    ModerationUnban,
    ModerationExpire,
    ModerationRemove,
    ModerationDuration,
    ModerationReason,
    ModerationPurge,
    GiveawayCreate,
    GiveawayEnd,
    GiveawayReroll,
    GiveawayDelete,
    XPEdit,
}

impl Display for Permission {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Permission::PermissionsView => write!(f, "permissions.view"),
            Permission::PermissionsEdit => write!(f, "permissions.edit"),
            Permission::ConfigEdit => write!(f, "config.edit"),
            Permission::ModerationStrike => write!(f, "moderation.strike"),
            Permission::ModerationSearchSelf => write!(f, "moderation.search.self"),
            Permission::ModerationSearchSelfExpired => write!(f, "moderation.search.self.expired"),
            Permission::ModerationSearchOthers => write!(f, "moderation.search.others"),
            Permission::ModerationSearchOthersExpired => {
                write!(f, "moderation.search.others.expired")
            }
            Permission::ModerationSearchUuid => write!(f, "moderation.search.uuid"),
            Permission::ModerationMute => write!(f, "moderation.mute"),
            Permission::ModerationUnmute => write!(f, "moderation.unmute"),
            Permission::ModerationKick => write!(f, "moderation.kick"),
            Permission::ModerationBan => write!(f, "moderation.ban"),
            Permission::ModerationUnban => write!(f, "moderation.unban"),
            Permission::ModerationExpire => write!(f, "moderation.expire"),
            Permission::ModerationRemove => write!(f, "moderation.remove"),
            Permission::ModerationDuration => write!(f, "moderation.duration"),
            Permission::ModerationReason => write!(f, "moderation.reason"),
            Permission::ModerationPurge => write!(f, "moderation.purge"),
            Permission::GiveawayCreate => write!(f, "giveaway.create"),
            Permission::GiveawayEnd => write!(f, "giveaway.end"),
            Permission::GiveawayReroll => write!(f, "giveaway.reroll"),
            Permission::GiveawayDelete => write!(f, "giveaway.delete"),
            Permission::XPEdit => write!(f, "xp.edit"),
        }
    }
}

impl From<&str> for Permission {
    fn from(value: &str) -> Self {
        match value {
            "permissions.view" => Permission::PermissionsView,
            "permissions.edit" => Permission::PermissionsEdit,
            "config.edit" => Permission::ConfigEdit,
            "moderation.strike" => Permission::ModerationStrike,
            "moderation.search.self" => Permission::ModerationSearchSelf,
            "moderation.search.self.expired" => Permission::ModerationSearchSelfExpired,
            "moderation.search.others" => Permission::ModerationSearchOthers,
            "moderation.search.others.expired" => Permission::ModerationSearchOthersExpired,
            "moderation.search.uuid" => Permission::ModerationSearchUuid,
            "moderation.mute" => Permission::ModerationMute,
            "moderation.unmute" => Permission::ModerationUnmute,
            "moderation.kick" => Permission::ModerationKick,
            "moderation.ban" => Permission::ModerationBan,
            "moderation.unban" => Permission::ModerationUnban,
            "moderation.expire" => Permission::ModerationExpire,
            "moderation.remove" => Permission::ModerationRemove,
            "moderation.duration" => Permission::ModerationDuration,
            "moderation.reason" => Permission::ModerationReason,
            "moderation.purge" => Permission::ModerationPurge,
            "giveaway.create" => Permission::GiveawayCreate,
            "giveaway.end" => Permission::GiveawayEnd,
            "giveaway.reroll" => Permission::GiveawayReroll,
            "giveaway.delete" => Permission::GiveawayDelete,
            "xp.edit" => Permission::XPEdit,
            _ => panic!("Invalid permission"),
        }
    }
}

impl Permission {
    #[tracing::instrument(skip(guild, user), fields(guild_id = guild.as_u64(), user_id = user.as_u64()))]
    pub async fn get_user(guild: Guild, user: User) -> Vec<Permission> {
        debug!("Querying main database for user permissions in guild");
        match sqlx::query!(
            "SELECT permission FROM users WHERE guild_id = $1 AND id = $2",
            guild.as_i64(),
            user.as_i64()
        )
        .fetch_all(Bot::global().postgres())
        .await
        {
            Ok(rows) => {
                let mut permissions = vec![];
                for row in rows {
                    permissions.push(Permission::from(row.permission.as_str()));
                }
                permissions
            }
            Err(err) => {
                error!(
                    "Attempted to query main database for user permissions, failed with error: {err}",
                );
                return vec![];
            }
        }
    }

    #[tracing::instrument(skip(guild, role), fields(guild_id = guild.as_u64(), role_id = role.as_u64()))]
    pub async fn get_role(guild: Guild, role: Role) -> Vec<Permission> {
        debug!("Querying main database for role permissions in guild");
        match sqlx::query!(
            "SELECT permission FROM roles WHERE guild_id = $1 AND id = $2",
            guild.as_i64(),
            role.as_i64()
        )
        .fetch_all(Bot::global().postgres())
        .await
        {
            Ok(rows) => {
                let mut permissions = vec![];
                for row in rows {
                    permissions.push(Permission::from(row.permission.as_str()));
                }
                permissions
            }
            Err(err) => {
                error!(
                    "Attempted to query main database for role permissions, failed with error: {err}",
                );
                return vec![];
            }
        }
    }
}
