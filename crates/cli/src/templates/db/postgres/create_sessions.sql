CREATE TABLE sessions (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    "userId" TEXT NOT NULL,
    "issuedAt" TIMESTAMPTZ,
    "expiresAt" TIMESTAMPTZ,
    "usedAt" TIMESTAMPTZ,
    token TEXT NOT NULL,

    CONSTRAINT fk_user FOREIGN KEY ("userId") REFERENCES users(id) ON DELETE CASCADE
);

