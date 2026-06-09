-- Add migration script here
CREATE TABLE users (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    createdAt DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- CREATE INDEX idx_user_email ON users(email);
