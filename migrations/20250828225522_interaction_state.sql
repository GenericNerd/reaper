-- Add migration script here
CREATE TABLE interaction_states (
  id UUID PRIMARY KEY,
  action TEXT NOT NULL,
  user_id BIGINT NOT NULL,
  data JSONB NOT NULL,
  expires_at TIMESTAMP NOT NULL DEFAULT (CURRENT_TIMESTAMP + INTERVAL '1 hour')
);