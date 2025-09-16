-- Add migration script here
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL,
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Uniqueness (store email lowercased in app to enforce case-insensitive)
CREATE UNIQUE INDEX users_email_key ON users (email);
CREATE UNIQUE INDEX users_username_key ON users (username);
