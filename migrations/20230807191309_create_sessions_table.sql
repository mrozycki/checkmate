-- Create sessions table
CREATE TABLE IF NOT EXISTS sessions (
    token bytea PRIMARY KEY,
    user_id uuid REFERENCES users (id) ON DELETE CASCADE,
    valid_until timestamptz NOT NULL
);