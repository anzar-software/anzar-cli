
CREATE TABLE signing_keys (
  id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
  encrypted_private_key TEXT         NOT NULL,
  public_key            TEXT         NOT NULL,
  algorithm             VARCHAR(10)  NOT NULL DEFAULT 'RS256',
  kid                   VARCHAR      NOT NULL UNIQUE,
  kty                   VARCHAR      NOT NULL UNIQUE,
  created_at            DATETIME     NOT NULL DEFAULT (datetime('now')),
  rotated_at            DATETIME,
  expiresAt             DATETIME,
  status                TEXT         NOT NULL
);
