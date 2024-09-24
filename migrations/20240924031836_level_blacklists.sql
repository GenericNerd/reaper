-- Add migration script here
CREATE TABLE xp_role_blacklists (
  guild_id BIGINT NOT NULL,
  role BIGINT NOT NULL,
  PRIMARY KEY (guild_id, role)
);

CREATE TABLE xp_channel_blacklists (
  guild_id BIGINT NOT NULL,
  channel BIGINT NOT NULL,
  PRIMARY KEY (guild_id, channel)
);