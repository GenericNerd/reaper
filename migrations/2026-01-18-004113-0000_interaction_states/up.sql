-- Your SQL goes here
CREATE TABLE interaction_state (
  id UUID PRIMARY KEY,
  handler_id TEXT NOT NULL,
  state JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMPTZ NULL
);