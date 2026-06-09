-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "pgcrypto"; -- if Postgres < 13
CREATE TYPE role AS ENUM ('User', 'Admin');

CREATE TABLE users (
    id          TEXT        PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    username    TEXT        NOT NULL UNIQUE,
    email       TEXT        NOT NULL UNIQUE,
    role        role        NOT NULL DEFAULT 'User',
    verified    BOOLEAN     NOT NULL DEFAULT FALSE,
    "createdAt" TIMESTAMPTZ  DEFAULT NOW()
);
