-- Add migration script here
DROP TABLE IF EXISTS giveaways;
CREATE TABLE giveaways (
    id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,
    prize VARCHAR(255) NOT NULL,
    description VARCHAR(255) NULL,
    winners INT NOT NULL DEFAULT 1,
    duration TIMESTAMP NOT NULL,
    role_restriction BIGINT NULL,
    PRIMARY KEY (id)
);
DROP TABLE IF EXISTS giveaway_entry;
CREATE TABLE giveaway_entry (
    id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    PRIMARY KEY (id, user_id)
);