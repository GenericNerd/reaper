-- Add migration script here
ALTER TABLE giveaways ADD COLUMN guild_id BIGINT NOT NULL;
ALTER TABLE giveaway_entry ADD COLUMN guild_id BIGINT NOT NULL;