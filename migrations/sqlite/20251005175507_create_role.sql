CREATE TABLE roles (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL UNIQUE,
    createdAt DATETIME NOT NULL DEFAULT (datetime('now'))
);
