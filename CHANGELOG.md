# Changelog

All notable changes to this project will be documented here.

## [Unreleased]

### Added
### Fixed


## [0.8.5] - 2026-05-24
### Added
- Support for RateLimits using three type: default, ip and strict.
- RateLimit can be enabled or disabled.
- Read secret values from .env file instead of saving them in anzar.yml.
- Introduce tests for sesison authentication method.
- Add rate limits testings

### Fixed
- Search for token in Database using atomic operation withconsume method.
- /email/verify has been changed form GET to POST because it introduce a side effect.
