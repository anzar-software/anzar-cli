CREATE TABLE sessions (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    "userId" TEXT NOT NULL,
    "issuedAt" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "expiresAt" TIMESTAMPTZ,
    "usedAt" TIMESTAMPTZ,
    token TEXT NOT NULL,

    roles TEXT[],
    permissions TEXT[],

    CONSTRAINT fk_user FOREIGN KEY ("userId") REFERENCES users(id) ON DELETE CASCADE
);
