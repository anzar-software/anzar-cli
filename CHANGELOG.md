# Changelog

All notable changes to this project will be documented here.

## [Unreleased]

### Added
- Support for RateLimits using three type: default, ip and strict.
- Read secret values from .env file instead of saving them in anzar.yml.
- Introduce tests for sesison authentication method.
- RateLimit can be enabled or disabled.
- Add rate limits testings

### Fixed
- Search for token in Database using atomic operation withconsume method.
- /email/verify has been changed form GET to POST because it introduce a side effect.


## [0.5.5] - 2026-06-23
