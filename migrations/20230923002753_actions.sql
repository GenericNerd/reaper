-- Add migration script here
DROP TYPE IF EXISTS action_type CASCADE;
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
    default_strike_duration VARCHAR(8) NULL DEFAULT '30d',
    PRIMARY KEY (guild_id)
);
DROP TABLE IF EXISTS actions;
CREATE TABLE actions (
    id VARCHAR(24) NOT NULL,
    type action_type NOT NULL,
    user_id BIGINT NOT NULL,
    moderator_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    reason VARCHAR(255) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    expiry TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id)
);
DROP TABLE IF EXISTS strike_escalations;
CREATE TABLE strike_escalations (
    guild_id BIGINT NOT NULL,
    strike_count INT NOT NULL,
    action_type action_type NOT NULL,
    action_duration VARCHAR(8) NULL
);