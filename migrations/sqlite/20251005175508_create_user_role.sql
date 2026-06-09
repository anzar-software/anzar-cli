CREATE TABLE user_roles (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),

    userId TEXT NOT NULL,
    roleId TEXT NOT NULL,
    issuedAt DATETIME NOT NULL DEFAULT (datetime('now')),

    UNIQUE ("roleId", "userId"),

    FOREIGN KEY (userId) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (roleId) REFERENCES roles(id) ON DELETE CASCADE
);
