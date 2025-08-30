-- Add migration script here
DROP TABLE IF EXISTS logging_configuration;
CREATE TABLE logging_configuration (
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