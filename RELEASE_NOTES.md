# Release Notes - v0.3.0

## 🐛 Critical Bug Fix: Authentication Method

**Date:** September 30, 2025  
**Type:** Bug Fix (Breaking Change)

### What's Fixed
- **Fixed Jira authentication method**: Changed from Bearer token to Basic authentication
- **Root Cause**: Jira Personal Access Tokens require Basic auth (email:token), not Bearer auth
- **Impact**: This resolves authentication failures when connecting to Jira instances

### Breaking Changes
- Authentication header format changed from `Bearer {token}` to `Basic {base64(email:token)}`
- This may affect any custom integrations that were expecting Bearer authentication

### Technical Details
- Updated `auth_header()` method in `src/config/jira.rs`
- Fixed deprecated `base64::encode()` usage
- Enhanced error handling and documentation

### Migration Guide
No action required if using the MCP server normally. The fix is automatic and backward compatible for standard usage.

### Files Changed
- `src/config/jira.rs` - Authentication method fix
- `docs/troubleshooting.md` - Updated documentation
- `CONFIGURATION.md` - Added authentication notes
- `CHANGELOG.md` - New changelog file

### Testing
- ✅ Authentication header format verified
- ✅ Binary builds and runs successfully
- ✅ Environment variable loading works correctly
- ✅ No compilation warnings

### Binary Location
- **Release Binary**: `target/release/rust-jira-mcp`
- **Version**: 0.3.0
- **Build Date**: September 30, 2025
