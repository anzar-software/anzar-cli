CREATE TABLE roles (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    name TEXT NOT NULL UNIQUE,
    "createdAt" TIMESTAMPTZ DEFAULT NOW()
);
