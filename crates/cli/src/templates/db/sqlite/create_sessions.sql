-- Add migration script here
CREATE TABLE sessions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    userId TEXT NOT NULL,
    issuedAt DATETIME NOT NULL DEFAULT (datetime('now')),
    expiresAt DATETIME,
    usedAt DATETIME,
    token TEXT NOT NULL,

    FOREIGN KEY (userId) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_session_token ON sessions(token);
CREATE INDEX idx_session_userId ON sessions(userId);
