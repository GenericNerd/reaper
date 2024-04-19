use std::fmt::{self, Display, Formatter};

#[derive(strum::EnumIter, Copy, Clone, PartialEq)]
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
    GiveawayCreate,
    GiveawayEnd,
    GiveawayReroll,
    GiveawayDelete,
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
            Permission::ModerationSearchOthersExpired => write!(f, "moderation.search.others.expired"),
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
            Permission::GiveawayCreate => write!(f, "giveaway.create"),
            Permission::GiveawayEnd => write!(f, "giveaway.end"),
            Permission::GiveawayReroll => write!(f, "giveaway.reroll"),
            Permission::GiveawayDelete => write!(f, "giveaway.delete"),
        }
    }
}

impl From<String> for Permission {
    fn from(value: String) -> Self {
        match value.as_str() {
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
            "giveaway.create" => Permission::GiveawayCreate,
            "giveaway.end" => Permission::GiveawayEnd,
            "giveaway.reroll" => Permission::GiveawayReroll,
            "giveaway.delete" => Permission::GiveawayDelete,
            _ => panic!("Invalid permission"),
        }
    }
}
