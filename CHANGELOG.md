# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.2] - 2025-10-01

### Fixed
- **URL Construction Bug**: Fixed critical bug where API base URL was missing the `/2` path component
  - MCP server was calling `https://jira.corp.adobe.com/rest/api/myself` instead of `https://jira.corp.adobe.com/rest/api/2/myself`
  - This caused requests to be redirected to Okta SSO instead of reaching the Jira API
  - Fixed URL joining logic to properly preserve the `/2` in the API path
  - Resolves authentication failures and Okta redirect issues

### Added
- **Debug Tools**: Added comprehensive debugging tools to identify HTTP request issues
  - `debug_auth` binary for testing authentication directly
  - `debug_curl.sh` script for comparing curl vs MCP requests
  - Enhanced HTTP request/response logging with detailed headers
  - User-Agent header added to all requests

### Technical Details
- Updated `build_url()` method in `src/jira/client.rs` to ensure proper URL joining
- Fixed both regular Jira API and Zephyr API URL construction
- Added trailing slash handling to prevent URL path truncation
- Re-enabled redirects after fixing the root cause

## [0.4.1] - 2025-10-01

### Fixed
- **Parameter Naming Compatibility**: Fixed parameter naming mismatch between camelCase and snake_case formats
  - MCP tools now accept both `maxResults`/`max_results` and `startAt`/`start_at` parameter formats
  - Maintains backward compatibility with existing snake_case clients
  - Resolves parsing failures when clients send camelCase parameters (e.g., Ferris MCP client)
  - Updated tool schemas to document both parameter naming conventions

### Added
- **Comprehensive Parameter Testing**: Added extensive test coverage for both parameter naming conventions
  - Tests for camelCase parameters (e.g., `maxResults`, `startAt`)
  - Tests for snake_case parameters (e.g., `max_results`, `start_at`)
  - Tests for mixed parameter usage and priority handling
  - Integration tests for both parameter formats

### Changed
- **Tool Parameter Handling**: Updated `SearchIssuesTool` and `GetZephyrTestCasesTool` to support both parameter formats
- **Tool Schemas**: Enhanced MCP tool schemas to document both parameter naming conventions
- **Parameter Priority**: When both formats are provided, snake_case takes precedence for backward compatibility

## [0.4.0] - 2025-09-30

### Added
- **Smart Authentication Detection**: Automatically detects between Adobe Jira (Bearer) and Standard Jira (Basic) authentication
  - Long tokens (>20 chars) without colons use Bearer authentication
  - Short tokens or tokens with colons use Basic authentication
- **Comprehensive Documentation**: Updated all documentation with detailed authentication setup instructions
- **Enhanced Testing**: Added comprehensive test coverage for both authentication methods
- **Configuration Examples**: Added specific examples for both Adobe and Standard Jira instances

### Fixed
- **BREAKING CHANGE**: Fixed authentication method from Bearer to Basic auth for Jira Personal Access Tokens
  - Jira PATs require Basic authentication (email:token) not Bearer authentication
  - Updated `auth_header()` method to use proper Basic auth format
  - Fixed deprecated `base64::encode()` usage to use `base64::Engine::encode()`

### Changed
- Authentication header now dynamically detects between `Bearer token` and `Basic base64(email:token)` formats
- Updated documentation to clarify correct authentication method for both Jira types
- Enhanced troubleshooting documentation for authentication issues
- Clear notes about Jira PAT authentication requirements

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
