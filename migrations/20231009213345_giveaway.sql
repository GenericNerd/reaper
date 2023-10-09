-- Add migration script here
DROP TABLE IF EXISTS giveaways;
CREATE TABLE giveaways (
    id BIGINT NOT NULL,
    winners INT(4) NOT NULL DEFAULT 1,
    duration TIMESTAMP NOT NULL,
    PRIMARY KEY (id)
);
DROP TABLE IF EXISTS giveaway_entry;
CREATE TABLE giveaway_entry (
    id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    PRIMARY KEY (id, user_id)
);