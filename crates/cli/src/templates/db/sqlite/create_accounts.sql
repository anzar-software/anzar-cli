-- Add migration script here
CREATE TABLE accounts (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    userId TEXT NOT NULL,
    password TEXT NOT NULL,
    locked BOOLEAN NOT NULL DEFAULT 0,
    createdAt DATETIME NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (userId) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_account_userId ON accounts(userId);
