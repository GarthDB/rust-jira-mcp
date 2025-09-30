# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-09-30

### Fixed
- **BREAKING CHANGE**: Fixed authentication method from Bearer to Basic auth for Jira Personal Access Tokens
  - Jira PATs require Basic authentication (email:token) not Bearer authentication
  - Updated `auth_header()` method to use proper Basic auth format
  - Fixed deprecated `base64::encode()` usage to use `base64::Engine::encode()`

### Changed
- Authentication header now uses `Basic base64(email:token)` format instead of `Bearer token`
- Updated documentation to clarify correct authentication method

### Added
- Enhanced troubleshooting documentation for authentication issues
- Clear notes about Jira PAT authentication requirements

## [0.2.0] - Previous Release

### Added
- Initial release with comprehensive Jira API integration
- MCP server implementation
- Performance optimizations and caching
- Extensive configuration management
