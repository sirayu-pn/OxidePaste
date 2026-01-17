-- Add user_id foreign key to pastes table
ALTER TABLE pastes ADD COLUMN user_id INTEGER REFERENCES users(id);

-- Index for faster user paste lookups
CREATE INDEX IF NOT EXISTS idx_pastes_user_id ON pastes(user_id);
