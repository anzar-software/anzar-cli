-- Add migration script here
CREATE TABLE password_reset_tokens (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    userId TEXT NOT NULL,
    issuedAt DATETIME NOT NULL DEFAULT (datetime('now')),
    expiresAt DATETIME,
    usedAt DATETIME,
    token TEXT NOT NULL,

    FOREIGN KEY (userId) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_password_reset_token_token ON password_reset_tokens(token);
CREATE INDEX idx_password_reset_token_userId ON password_reset_tokens(userId);
