-- Add migration script here
DROP TABLE IF EXISTS guild_role_recovery_config;
CREATE TABLE guild_role_recovery_config (
    guild_id BIGINT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (guild_id)
);