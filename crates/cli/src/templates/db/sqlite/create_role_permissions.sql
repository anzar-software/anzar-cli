CREATE TABLE role_permissions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),

    permissionId TEXT NOT NULL,
    roleId TEXT NOT NULL,
    issuedAt DATETIME NOT NULL DEFAULT (datetime('now')),

    UNIQUE ("permissionId", "roleId"),

    FOREIGN KEY (permissionId) REFERENCES permissions(id) ON DELETE CASCADE,
    FOREIGN KEY (roleId) REFERENCES roles(id) ON DELETE CASCADE
);
