-- Add migration script here
CREATE TABLE refresh_tokens (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    userId TEXT NOT NULL,
    issuedAt DATETIME NOT NULL DEFAULT (datetime('now')),
    expiresAt DATETIME,
    usedAt DATETIME,
    jti TEXT NOT NULL,

    FOREIGN KEY (userId) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_refresh_token_jti ON refresh_tokens(jti);
CREATE INDEX idx_refresh_token_userId ON refresh_tokens(userId);
