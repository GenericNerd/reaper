-- Add migration script here
DROP TABLE IF EXISTS users;
CREATE TABLE users (
    id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    permission VARCHAR(255) NOT NULL
);
DROP TABLE IF EXISTS roles;
CREATE TABLE roles (
    id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    permission VARCHAR(255) NOT NULL
);