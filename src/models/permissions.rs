#[derive(strum::EnumIter, Clone)]
pub enum Permission {
    PermissionsView,
    PermissionsEdit,
    LoggingEdit,
    ModerationEdit,
    BoardsEdit,
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

impl ToString for Permission {
    fn to_string(&self) -> String {
        match self {
            Permission::PermissionsView => "permissions.view".to_string(),
            Permission::PermissionsEdit => "permissions.edit".to_string(),
            Permission::LoggingEdit => "logging.edit".to_string(),
            Permission::ModerationEdit => "moderation.edit".to_string(),
            Permission::BoardsEdit => "boards.edit".to_string(),
            Permission::ModerationStrike => "moderation.strike".to_string(),
            Permission::ModerationSearchSelf => "moderation.search.self".to_string(),
            Permission::ModerationSearchSelfExpired => "moderation.search.self.expired".to_string(),
            Permission::ModerationSearchOthers => "moderation.search.others".to_string(),
            Permission::ModerationSearchOthersExpired => {
                "moderation.search.others.expired".to_string()
            }
            Permission::ModerationSearchUuid => "moderation.search.uuid".to_string(),
            Permission::ModerationMute => "moderation.mute".to_string(),
            Permission::ModerationUnmute => "moderation.unmute".to_string(),
            Permission::ModerationKick => "moderation.kick".to_string(),
            Permission::ModerationBan => "moderation.ban".to_string(),
            Permission::ModerationUnban => "moderation.unban".to_string(),
            Permission::ModerationExpire => "moderation.expire".to_string(),
            Permission::ModerationRemove => "moderation.remove".to_string(),
            Permission::ModerationDuration => "moderation.duration".to_string(),
            Permission::ModerationReason => "moderation.reason".to_string(),
            Permission::GiveawayCreate => "giveaway.create".to_string(),
            Permission::GiveawayEnd => "giveaway.end".to_string(),
            Permission::GiveawayReroll => "giveaway.reroll".to_string(),
            Permission::GiveawayDelete => "giveaway.delete".to_string(),
        }
    }
}

impl From<String> for Permission {
    fn from(value: String) -> Self {
        match value.as_str() {
            "permissions.view" => Permission::PermissionsView,
            "permissions.edit" => Permission::PermissionsEdit,
            "logging.edit" => Permission::LoggingEdit,
            "moderation.edit" => Permission::ModerationEdit,
            "boards.edit" => Permission::BoardsEdit,
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

impl PartialEq for Permission {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}
