-- Add migration script here
DROP TABLE IF EXISTS role_recovery;
CREATE TABLE role_recovery (
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    PRIMARY KEY (guild_id, user_id, role_id)
);