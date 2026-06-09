CREATE TABLE accounts (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    "userId" TEXT NOT NULL,
    password TEXT NOT NULL,
    locked BOOLEAN NOT NULL DEFAULT FALSE,
    "createdAt" TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT fk_user FOREIGN KEY ("userId") REFERENCES users(id) ON DELETE CASCADE
);

