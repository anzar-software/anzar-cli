-- Add migration script here
CREATE TABLE users (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    "createdAt" TIMESTAMPTZ DEFAULT NOW()
);

-- CREATE INDEX idx_user_email ON users(email);
