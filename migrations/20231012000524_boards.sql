-- Add migration script here
DROP TABLE IF EXISTS boards;
CREATE TABLE boards (
    channel_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    emote_quota INT NOT NULL DEFAULT 5,
    ignore_self_reacts BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (channel_id, guild_id)
);
DROP TABLE IF EXISTS board_emotes;
CREATE TABLE board_emotes (
    channel_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    emote VARCHAR(255) NOT NULL,
    PRIMARY KEY (channel_id, guild_id, emote)
);
DROP TABLE IF EXISTS board_ignored_channels;
CREATE TABLE board_ignored_channels (
    channel_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    ignored_channel BIGINT NOT NULL,
    PRIMARY KEY (channel_id, guild_id, ignored_channel)
);
DROP TABLE IF EXISTS board_entries;
CREATE TABLE board_entries (
    channel_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    message_id BIGINT NOT NULL,
    PRIMARY KEY (channel_id, guild_id, message_id)
);