// @generated automatically by Diesel CLI.

diesel::table! {
    actions (id) {
        #[max_length = 24]
        id -> Varchar,
        #[max_length = 6]
        action_type -> Varchar,
        user_id -> Int8,
        moderator_id -> Int8,
        guild_id -> Int8,
        #[max_length = 255]
        reason -> Varchar,
        active -> Bool,
        expiry -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    board_emotes (channel_id, guild_id, emote) {
        channel_id -> Int8,
        guild_id -> Int8,
        emote -> Text,
    }
}

diesel::table! {
    board_entries (channel_id, guild_id, message_id) {
        channel_id -> Int8,
        guild_id -> Int8,
        message_id -> Int8,
    }
}

diesel::table! {
    board_ignored_channels (channel_id, guild_id, ignored_channel) {
        channel_id -> Int8,
        guild_id -> Int8,
        ignored_channel -> Int8,
    }
}

diesel::table! {
    boards (channel_id, guild_id) {
        channel_id -> Int8,
        guild_id -> Int8,
        emote_quota -> Int4,
        ignore_self_reacts -> Bool,
    }
}

diesel::table! {
    giveaway_entry (id, user_id) {
        id -> Int8,
        user_id -> Int8,
        guild_id -> Int8,
    }
}

diesel::table! {
    giveaways (id) {
        id -> Int8,
        channel_id -> Int8,
        guild_id -> Int8,
        #[max_length = 255]
        prize -> Varchar,
        #[max_length = 255]
        description -> Nullable<Varchar>,
        winners -> Int4,
        duration -> Timestamp,
        role_restriction -> Nullable<Int8>,
        host -> Int8,
        image_url -> Nullable<Text>,
        color -> Nullable<Int4>,
    }
}

diesel::table! {
    global_kills (feature) {
        feature -> Text,
        active -> Bool,
        killed_by -> Nullable<Int8>,
    }
}

diesel::table! {
    guild_kills (guild_id) {
        guild_id -> Int8,
        killed_by -> Int8,
    }
}

diesel::table! {
    guild_role_recovery_config (guild_id) {
        guild_id -> Int8,
        enabled -> Bool,
    }
}

diesel::table! {
    logging_configuration (guild_id) {
        guild_id -> Int8,
        log_actions -> Bool,
        log_messages -> Bool,
        log_voice -> Bool,
        log_channel -> Nullable<Int8>,
        log_action_channel -> Nullable<Int8>,
        log_message_channel -> Nullable<Int8>,
        log_voice_channel -> Nullable<Int8>,
    }
}

diesel::table! {
    moderation_configuration (guild_id) {
        guild_id -> Int8,
        mute_role -> Nullable<Int8>,
        #[max_length = 8]
        default_strike_duration -> Nullable<Varchar>,
        footer -> Nullable<Text>,
    }
}

diesel::table! {
    role_recovery (guild_id, user_id, role_id) {
        guild_id -> Int8,
        user_id -> Int8,
        role_id -> Int8,
    }
}

diesel::table! {
    roles (id, guild_id, permission) {
        id -> Int8,
        guild_id -> Int8,
        #[max_length = 255]
        permission -> Varchar,
    }
}

diesel::table! {
    strike_escalations (guild_id, strike_count) {
        guild_id -> Int8,
        strike_count -> Int4,
        #[max_length = 6]
        action_type -> Varchar,
        #[max_length = 8]
        action_duration -> Nullable<Varchar>,
    }
}

diesel::table! {
    user_kills (user_id) {
        user_id -> Int8,
        killed_by -> Int8,
    }
}

diesel::table! {
    user_xp (guild_id, user_id) {
        guild_id -> Int8,
        user_id -> Int8,
        xp -> Int8,
    }
}

diesel::table! {
    users (id, guild_id, permission) {
        id -> Int8,
        guild_id -> Int8,
        #[max_length = 255]
        permission -> Varchar,
    }
}

diesel::table! {
    xp_channel_blacklists (guild_id, channel) {
        guild_id -> Int8,
        channel -> Int8,
    }
}

diesel::table! {
    xp_channel_multipliers (guild_id, channel) {
        guild_id -> Int8,
        channel -> Int8,
        multiplier -> Float4,
    }
}

diesel::table! {
    xp_configuration (guild_id) {
        guild_id -> Int8,
        min_xp_per_message -> Nullable<Int4>,
        max_xp_per_message -> Nullable<Int4>,
        set_xp_per_message -> Nullable<Int4>,
        message_cooldown -> Int4,
        max_level -> Nullable<Int4>,
        reset_level_on_leave -> Bool,
        stack_rewards -> Bool,
        stack_multipliers -> Bool,
        multiplier_cap -> Nullable<Float4>,
    }
}

diesel::table! {
    xp_level_up_messages (guild_id) {
        guild_id -> Int8,
        enabled -> Bool,
        dm_message -> Bool,
        channel -> Nullable<Int8>,
        message -> Nullable<Text>,
    }
}

diesel::table! {
    xp_rewards (guild_id, role) {
        guild_id -> Int8,
        role -> Int8,
        level -> Int8,
    }
}

diesel::table! {
    xp_role_blacklists (guild_id, role) {
        guild_id -> Int8,
        role -> Int8,
    }
}

diesel::table! {
    xp_role_multipliers (guild_id, role) {
        guild_id -> Int8,
        role -> Int8,
        multiplier -> Float4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    actions,
    board_emotes,
    board_entries,
    board_ignored_channels,
    boards,
    giveaway_entry,
    giveaways,
    global_kills,
    guild_kills,
    guild_role_recovery_config,
    logging_configuration,
    moderation_configuration,
    role_recovery,
    roles,
    strike_escalations,
    user_kills,
    user_xp,
    users,
    xp_channel_blacklists,
    xp_channel_multipliers,
    xp_configuration,
    xp_level_up_messages,
    xp_rewards,
    xp_role_blacklists,
    xp_role_multipliers,
);
