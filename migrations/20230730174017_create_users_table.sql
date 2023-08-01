-- Create Users Table
CREATE TABLE users(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL, -- plain text for now :/
    created_at timestamptz NOT NULL
);