-- Add migration script here
ALTER TABLE giveaways ADD COLUMN host BIGINT NOT NULL;
ALTER TABLE giveaways ADD COLUMN image_url TEXT NULL;
ALTER TABLE giveaways ADD COLUMN color INTEGER NULL;