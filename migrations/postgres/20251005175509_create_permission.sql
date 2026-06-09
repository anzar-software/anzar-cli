CREATE TABLE permissions (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    name TEXT NOT NULL UNIQUE,
    "createdAt" TIMESTAMPTZ DEFAULT NOW()
);
