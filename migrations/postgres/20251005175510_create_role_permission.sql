CREATE TABLE role_permissions (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,

    "permissionId" TEXT NOT NULL,
    "roleId" TEXT NOT NULL,
    "issuedAt" TIMESTAMPTZ,

    UNIQUE ("permissionId", "roleId"),

    CONSTRAINT fk_permission FOREIGN KEY ("permissionId") REFERENCES permissions(id) ON DELETE CASCADE,
    CONSTRAINT fk_role FOREIGN KEY ("roleId") REFERENCES roles(id) ON DELETE CASCADE
);
