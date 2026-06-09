CREATE TABLE user_roles (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,

    "userId" TEXT NOT NULL,
    "roleId" TEXT NOT NULL,
    "issuedAt" TIMESTAMPTZ,

    UNIQUE ("roleId", "userId"),

    CONSTRAINT fk_user FOREIGN KEY ("userId") REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_role FOREIGN KEY ("roleId") REFERENCES roles(id) ON DELETE CASCADE
);
