-- Your SQL goes here
CREATE TABLE IF NOT EXISTS users (
    id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    permission VARCHAR(255) NOT NULL,
    PRIMARY KEY (id, guild_id, permission)
);

CREATE TABLE IF NOT EXISTS roles (
    id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    permission VARCHAR(255) NOT NULL,
    PRIMARY KEY (id, guild_id, permission)
);

CREATE TABLE IF NOT EXISTS moderation_configuration (
    guild_id BIGINT NOT NULL,
    mute_role BIGINT NULL,
    default_strike_duration VARCHAR(8) NULL DEFAULT '30d',
    footer TEXT NULL,
    PRIMARY KEY (guild_id)
);

CREATE TABLE IF NOT EXISTS actions (
    id VARCHAR(24) NOT NULL,
    action_type VARCHAR(6) NOT NULL,
    user_id BIGINT NOT NULL,
    moderator_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    reason VARCHAR(255) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    expiry TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS strike_escalations (
    guild_id BIGINT NOT NULL,
    strike_count INT NOT NULL,
    action_type VARCHAR(6) NOT NULL,
    action_duration VARCHAR(8) NULL,
    PRIMARY KEY (guild_id, strike_count)
);

CREATE TABLE IF NOT EXISTS logging_configuration (
    guild_id BIGINT NOT NULL,
    log_actions BOOLEAN NOT NULL DEFAULT FALSE,
    log_messages BOOLEAN NOT NULL DEFAULT FALSE,
    log_voice BOOLEAN NOT NULL DEFAULT FALSE,
    log_channel BIGINT NULL,
    log_action_channel BIGINT NULL,
    log_message_channel BIGINT NULL,
    log_voice_channel BIGINT NULL,
    PRIMARY KEY (guild_id)
);

CREATE TABLE IF NOT EXISTS role_recovery (
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    PRIMARY KEY (guild_id, user_id, role_id)
);

CREATE TABLE IF NOT EXISTS giveaways (
    id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    prize VARCHAR(255) NOT NULL,
    description VARCHAR(255) NULL,
    winners INT NOT NULL DEFAULT 1,
    duration TIMESTAMP NOT NULL,
    role_restriction BIGINT NULL,
    host BIGINT NOT NULL,
    image_url TEXT NULL,
    color INTEGER NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS giveaway_entry (
    id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    PRIMARY KEY (id, user_id)
);

CREATE TABLE IF NOT EXISTS boards (
    channel_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    emote_quota INT NOT NULL DEFAULT 5,
    ignore_self_reacts BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (channel_id, guild_id)
);

CREATE TABLE IF NOT EXISTS board_emotes (
    channel_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    emote TEXT NOT NULL,
    PRIMARY KEY (channel_id, guild_id, emote)
);

CREATE TABLE IF NOT EXISTS board_ignored_channels (
    channel_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    ignored_channel BIGINT NOT NULL,
    PRIMARY KEY (channel_id, guild_id, ignored_channel)
);

CREATE TABLE IF NOT EXISTS board_entries (
    channel_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    message_id BIGINT NOT NULL,
    PRIMARY KEY (channel_id, guild_id, message_id)
);

CREATE TABLE IF NOT EXISTS guild_role_recovery_config (
    guild_id BIGINT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (guild_id)
);

CREATE TABLE IF NOT EXISTS xp_configuration (
  guild_id BIGINT NOT NULL,
  min_xp_per_message INTEGER NULL DEFAULT 15,
  max_xp_per_message INTEGER NULL DEFAULT 40,
  set_xp_per_message INTEGER NULL DEFAULT NULL,
  message_cooldown INTEGER NOT NULL DEFAULT 60,
  max_level INTEGER NULL DEFAULT NULL,
  reset_level_on_leave BOOLEAN NOT NULL DEFAULT FALSE,
  stack_rewards BOOLEAN NOT NULL DEFAULT FALSE,
  stack_multipliers BOOLEAN NOT NULL DEFAULT TRUE,
  multiplier_cap FLOAT4 NULL DEFAULT NULL,
  PRIMARY KEY (guild_id)
);

CREATE TABLE IF NOT EXISTS xp_level_up_messages (
  guild_id BIGINT NOT NULL,
  enabled BOOLEAN NOT NULL DEFAULT FALSE,
  dm_message BOOLEAN NOT NULL DEFAULT FALSE,
  channel BIGINT NULL DEFAULT NULL,
  message TEXT NULL DEFAULT '{user.mention} has reached level **{user.level}**!',
  PRIMARY KEY (guild_id)
);

CREATE TABLE IF NOT EXISTS user_xp (
  guild_id BIGINT NOT NULL,
  user_id BIGINT NOT NULL,
  xp BIGINT NOT NULL DEFAULT 0,
  PRIMARY KEY (guild_id, user_id)
);

CREATE TABLE IF NOT EXISTS xp_channel_multipliers (
  guild_id BIGINT NOT NULL,
  channel BIGINT NOT NULL,
  multiplier FLOAT4 NOT NULL DEFAULT 1.0,
  PRIMARY KEY (guild_id, channel)
);

CREATE TABLE IF NOT EXISTS xp_role_multipliers (
  guild_id BIGINT NOT NULL,
  role BIGINT NOT NULL,
  multiplier FLOAT4 NOT NULL DEFAULT 1.0,
  PRIMARY KEY (guild_id, role)
);

CREATE TABLE IF NOT EXISTS xp_rewards (
  guild_id BIGINT NOT NULL,
  role BIGINT NOT NULL,
  level BIGINT NOT NULL,
  PRIMARY KEY (guild_id, role)
);

CREATE TABLE IF NOT EXISTS xp_role_blacklists (
  guild_id BIGINT NOT NULL,
  role BIGINT NOT NULL,
  PRIMARY KEY (guild_id, role)
);

CREATE TABLE IF NOT EXISTS xp_channel_blacklists (
  guild_id BIGINT NOT NULL,
  channel BIGINT NOT NULL,
  PRIMARY KEY (guild_id, channel)
);

CREATE TABLE IF NOT EXISTS global_kills (
    feature TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    killed_by BIGINT NULL DEFAULT NULL,
    PRIMARY KEY (feature)
);

CREATE TABLE IF NOT EXISTS user_kills (
    user_id BIGINT NOT NULL,
    killed_by BIGINT NOT NULL,
    PRIMARY KEY (user_id)
);

CREATE TABLE IF NOT EXISTS guild_kills (
    guild_id BIGINT NOT NULL,
    killed_by BIGINT NOT NULL,
    PRIMARY KEY (guild_id)
);

INSERT INTO global_kills (feature) VALUES ('commands') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.ban') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.config') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.duration') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.expire') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.giveaway') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.info') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.kick') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.mute') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.permissions') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.reason') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.remove') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.search') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.strike') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.unban') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.unmute') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.dm') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.privacy') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.purge') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.rank') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.leaderboard') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.level') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('commands.xp') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('logging') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('logging.action') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('logging.voice') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('logging.message') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('event.automod') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('event.boards') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('event.giveaways') ON CONFLICT DO NOTHING;
INSERT INTO global_kills (feature) VALUES ('event.expiry') ON CONFLICT DO NOTHING;