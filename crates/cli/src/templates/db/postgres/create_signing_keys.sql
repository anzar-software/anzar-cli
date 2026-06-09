-- create a restricted role for your app
-- CREATE ROLE auth_app LOGIN PASSWORD 'yourpassword';

-- create the table
CREATE TABLE signing_keys (
  id                    TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
  encrypted_private_key TEXT         NOT NULL,
  public_key            TEXT         NOT NULL,
  algorithm             VARCHAR(10)  NOT NULL DEFAULT 'RS256',
  kid                   VARCHAR      NOT NULL UNIQUE,
  kty                   VARCHAR      NOT NULL UNIQUE,
  created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
  rotated_at            TIMESTAMPTZ,
  expiresAt             TIMESTAMPTZ,
  status                TEXT         NOT NULL
);

-- lock it down
-- GRANT SELECT, INSERT ON signing_keys TO auth_app;
-- REVOKE UPDATE, DELETE ON signing_keys FROM auth_app;
