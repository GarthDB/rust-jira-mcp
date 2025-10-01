# GitHub Release v0.3.0

## 🐛 Critical Bug Fix: Jira Authentication Method

**Release Title:** `v0.3.0 - Fix Jira Authentication Method`

**Tag:** `v0.3.0`

**Release Type:** Bug Fix (Breaking Change)

---

## 📋 Release Notes

### What's Fixed
- **Fixed Jira authentication method**: Changed from Bearer token to Basic authentication
- **Root Cause**: Jira Personal Access Tokens require Basic auth (email:token), not Bearer auth
- **Impact**: This resolves authentication failures when connecting to Jira instances

### Breaking Changes
- Authentication header format changed from `Bearer {token}` to `Basic {base64(email:token)}`
- This may affect any custom integrations that were expecting Bearer authentication

### Technical Details
- Updated `auth_header()` method in `src/config/jira.rs`
- Fixed deprecated `base64::encode()` usage to use `base64::Engine::encode()`
- Enhanced error handling and documentation
- Updated all tests to expect Basic authentication format

### Files Changed
- `src/config/jira.rs` - Authentication method fix
- `tests/config_jira_test.rs` - Updated test expectations
- `tests/jira_client_integration_test.rs` - Updated test expectations
- `docs/troubleshooting.md` - Updated documentation
- `CONFIGURATION.md` - Added authentication notes
- `CHANGELOG.md` - New changelog file
- `RELEASE_NOTES.md` - Detailed release notes

### Migration Guide
No action required if using the MCP server normally. The fix is automatic and backward compatible for standard usage.

### Testing
- ✅ Authentication header format verified
- ✅ Binary builds and runs successfully
- ✅ Environment variable loading works correctly
- ✅ All tests pass (19/19 config tests, 27/27 integration tests)
- ✅ No compilation warnings

### Binary Information
- **Size**: 6.1MB
- **Version**: 0.3.0
- **Build Date**: September 30, 2025
- **Location**: `target/release/rust-jira-mcp`

---

## 🚀 How to Create the Release

1. Go to: https://github.com/GarthDB/rust-jira-mcp/releases/new
2. Select tag: `v0.3.0`
3. Release title: `v0.3.0 - Fix Jira Authentication Method`
4. Copy the content above as the release description
5. Check "Set as the latest release"
6. Click "Publish release"

---

## 📦 Assets to Upload (Optional)

If you want to include the binary:
1. Build: `cargo build --release`
2. Upload: `target/release/rust-jira-mcp` (6.1MB)

---

## 🔗 Related Issues

This release resolves authentication issues when using the rust-jira-mcp binary with Jira instances that require Basic authentication for Personal Access Tokens.
