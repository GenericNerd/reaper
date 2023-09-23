-- Add migration script here
CREATE TYPE action_type as ENUM (
    'strike',
    'mute',
    'kick',
    'ban'
);
DROP TABLE IF EXISTS moderation_configuration;
CREATE TABLE moderation_configuration (
    guild_id BIGINT NOT NULL,
    mute_role BIGINT NULL,
    default_strike_duration VARCHAR(8) NULL,
    PRIMARY KEY (guild_id)
);
DROP TABLE IF EXISTS actions;
CREATE TABLE actions (
    id VARCHAR(12) NOT NULL,
    type action_type NOT NULL,
    user_id BIGINT NOT NULL,
    moderator_id BIGINT NULL,
    guild_id BIGINT NOT NULL,
    reason VARCHAR(255) NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    expiry TIMESTAMP NULL,
    PRIMARY KEY (id)
);
DROP TABLE IF EXISTS strike_escalations;
CREATE TABLE strike_escalations (
    guild_id BIGINT NOT NULL,
    strike_count INT NOT NULL,
    action_type action_type NOT NULL
);