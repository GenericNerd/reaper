-- Add migration script here
CREATE TABLE interaction_states (
  id UUID PRIMARY KEY,
  interaction_route TEXT NOT NULL,
  user_id BIGINT NOT NULL,
  data JSONB NOT NULL,
  expires_at TIMESTAMP
);