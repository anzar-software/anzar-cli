# Changelog

All notable changes to this project will be documented here.

## [Unreleased]
### TODO
- Email sending
- Admin API
- Admin UI
- Social logins
- Social UI
- dart SDk
- maybe OIDC provider layer !!!
### Added
### Fixed


## [] - 2026-.-.
### Added
- add support for EC/EDDCA algorithms in jwks endpoint.
### Fixed
- Decode jwt using matching signing key not new one after rotation.
- resolve multiple deep layer RBAC inheritence.
- remove expects from code.

## [0.9.21] - 2026-06-12
### Added
- migrate to a multi-crate project (monorepo)
- add retired_at, expires_at to SigningKey model
- add keys command with subcommands (list, rotate, prune, revoke)
### Fixed
- update SigninKey attribute active -> status: active, retired, revoked

## [0.8.6] - 2026-05-28
### Added
- Key rotation by introducing versions
- Asymmetric JWT algorithms
- Only allow Asymmetric algorithms, deprecate HS256
- add `inherits: [user]` for RBAC system to inherits permissions
### Fixed
- push auth.password.security.max_failed_attempts to security.auth.max_failed_attempts under Secutiry Configuraion
- Remove the algorithm choice from the user-facing config entirely

## [0.8.5] - 2026-05-24
### Added
- Support for RateLimits using three type: default, ip and strict.
- RateLimit can be enabled or disabled.
- Read secret values from .env file instead of saving them in anzar.yml.
- Introduce tests for sesison authentication method.
- Add rate limits testings
### Fixed
- Search for token in Database using atomic operation with consume method.
- /email/verify has been changed form GET to POST because it introduce a side effect.
