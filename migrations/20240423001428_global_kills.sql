-- Add migration script here
CREATE TABLE global_kills (
    feature TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    killed_by BIGINT NULL DEFAULT NULL,
    PRIMARY KEY (feature)
);

CREATE TABLE user_kills (
    user_id BIGINT NOT NULL,
    killed_by BIGINT NOT NULL,
    PRIMARY KEY (user_id)
);

CREATE TABLE guild_kills (
    guild_id BIGINT NOT NULL,
    killed_by BIGINT NOT NULL,
    PRIMARY KEY (guild_id)
);

INSERT INTO global_kills (feature) VALUES ('commands');
INSERT INTO global_kills (feature) VALUES ('commands.ban');
INSERT INTO global_kills (feature) VALUES ('commands.config');
INSERT INTO global_kills (feature) VALUES ('commands.duration');
INSERT INTO global_kills (feature) VALUES ('commands.expire');
INSERT INTO global_kills (feature) VALUES ('commands.giveaway');
INSERT INTO global_kills (feature) VALUES ('commands.info');
INSERT INTO global_kills (feature) VALUES ('commands.kick');
INSERT INTO global_kills (feature) VALUES ('commands.mute');
INSERT INTO global_kills (feature) VALUES ('commands.permissions');
INSERT INTO global_kills (feature) VALUES ('commands.reason');
INSERT INTO global_kills (feature) VALUES ('commands.remove');
INSERT INTO global_kills (feature) VALUES ('commands.search');
INSERT INTO global_kills (feature) VALUES ('commands.strike');
INSERT INTO global_kills (feature) VALUES ('commands.unban');
INSERT INTO global_kills (feature) VALUES ('commands.unmute');
INSERT INTO global_kills (feature) VALUES ('commands.dm');
INSERT INTO global_kills (feature) VALUES ('logging');
INSERT INTO global_kills (feature) VALUES ('logging.action');
INSERT INTO global_kills (feature) VALUES ('logging.voice');
INSERT INTO global_kills (feature) VALUES ('logging.message');
INSERT INTO global_kills (feature) VALUES ('event.automod');
INSERT INTO global_kills (feature) VALUES ('event.boards');
INSERT INTO global_kills (feature) VALUES ('event.giveaways');
INSERT INTO global_kills (feature) VALUES ('event.expiry');
