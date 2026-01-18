use std::fmt::{Display, Formatter};

use diesel::{
    ExpressionMethods, RunQueryDsl,
    query_dsl::methods::{FilterDsl, SelectDsl},
};
use tracing::{debug, error};

use crate::models::{
    bot::Bot,
    serenity::{guild::Guild, role::Role, user::User},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

impl From<String> for Permission {
    fn from(value: String) -> Self {
        Permission::from(value.as_str())
    }
}

impl From<&String> for Permission {
    fn from(value: &String) -> Self {
        Permission::from(value.as_str())
    }
}

impl Permission {
    pub const ALL: [Self; 24] = [
        Self::PermissionsView,
        Self::PermissionsEdit,
        Self::ConfigEdit,
        Self::ModerationStrike,
        Self::ModerationSearchSelf,
        Self::ModerationSearchSelfExpired,
        Self::ModerationSearchOthers,
        Self::ModerationSearchOthersExpired,
        Self::ModerationSearchUuid,
        Self::ModerationMute,
        Self::ModerationUnmute,
        Self::ModerationKick,
        Self::ModerationBan,
        Self::ModerationUnban,
        Self::ModerationExpire,
        Self::ModerationRemove,
        Self::ModerationDuration,
        Self::ModerationReason,
        Self::ModerationPurge,
        Self::GiveawayCreate,
        Self::GiveawayEnd,
        Self::GiveawayReroll,
        Self::GiveawayDelete,
        Self::XPEdit,
    ];

    #[tracing::instrument(skip(guild, user), fields(guild_id = guild.as_u64(), user_id = user.as_u64()))]
    pub async fn get_user(guild: &Guild, user: &User) -> Vec<Permission> {
        use crate::schema::users::dsl::*;
        debug!("Querying main database for user permissions in guild");

        users
            .filter(guild_id.eq(guild.as_i64()))
            .filter(id.eq(user.as_i64()))
            .select(permission)
            .load::<String>(&mut Bot::instance().postgres())
            .unwrap_or_else(|error| {
                error!(
                    error = %error, "Attempted to query main database for user permissions",
                );
                Vec::new()
            })
            .iter()
            .map(|perm| Permission::from(perm))
            .collect::<Vec<Permission>>()
    }

    #[tracing::instrument(skip(guild, role), fields(guild_id = guild.as_u64(), role_id = role.as_u64()))]
    pub async fn get_role(guild: &Guild, role: &Role) -> Vec<Permission> {
        use crate::schema::roles::dsl::*;
        debug!("Querying main database for role permissions in guild");

        roles
            .filter(guild_id.eq(guild.as_i64()))
            .filter(id.eq(role.as_i64()))
            .select(permission)
            .load::<String>(&mut Bot::instance().postgres())
            .unwrap_or_else(|error| {
                error!(
                    error = %error, "Attempted to query main database for role permissions",
                );
                Vec::new()
            })
            .iter()
            .map(|perm| Permission::from(perm))
            .collect::<Vec<Permission>>()
    }
}
