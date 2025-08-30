-- Add migration script here
CREATE TABLE xp_configuration (
  guild_id BIGINT NOT NULL,
  min_xp_per_message INTEGER NULL DEFAULT 15,
  max_xp_per_message INTEGER NULL DEFAULT 40,
  set_xp_per_message INTEGER NULL DEFAULT NULL,
  message_cooldown INTEGER NOT NULL DEFAULT 60,
  max_level INTEGER NULL DEFAULT NULL,
  reset_level_on_leave BOOLEAN NOT NULL DEFAULT FALSE,
  stack_rewards BOOLEAN NOT NULL DEFAULT FALSE,
  stack_multipliers BOOLEAN NOT NULL DEFAULT TRUE,
  multiplier_cap FLOAT4 NULL DEFAULT NULL,
  PRIMARY KEY (guild_id)
);

CREATE TABLE xp_level_up_messages (
  guild_id BIGINT NOT NULL,
  enabled BOOLEAN NOT NULL DEFAULT FALSE,
  dm_message BOOLEAN NOT NULL DEFAULT FALSE,
  channel BIGINT NULL DEFAULT NULL,
  message TEXT NULL DEFAULT '{user.mention} has reached level **{user.level}**!',
  PRIMARY KEY (guild_id)
);

CREATE TABLE user_xp (
  guild_id BIGINT NOT NULL,
  user_id BIGINT NOT NULL,
  xp BIGINT NOT NULL DEFAULT 0,
  PRIMARY KEY (guild_id, user_id)
);

CREATE TABLE xp_channel_multipliers (
  guild_id BIGINT NOT NULL,
  channel BIGINT NOT NULL,
  multiplier FLOAT4 NOT NULL DEFAULT 1.0,
  PRIMARY KEY (guild_id, channel)
);

CREATE TABLE xp_role_multipliers (
  guild_id BIGINT NOT NULL,
  role BIGINT NOT NULL,
  multiplier FLOAT4 NOT NULL DEFAULT 1.0,
  PRIMARY KEY (guild_id, role)
);

CREATE TABLE xp_rewards (
  guild_id BIGINT NOT NULL,
  role BIGINT NOT NULL,
  level BIGINT NOT NULL,
  PRIMARY KEY (guild_id, role)
);

INSERT INTO global_kills (feature) VALUES ('commands.rank');
INSERT INTO global_kills (feature) VALUES ('commands.leaderboard');
INSERT INTO global_kills (feature) VALUES ('commands.level');
INSERT INTO global_kills (feature) VALUES ('commands.xp');