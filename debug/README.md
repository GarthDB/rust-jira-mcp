# Debug Tools

This directory contains debugging utilities for the Jira MCP server.

## Files

### `debug_curl.sh`
A bash script that compares curl requests with MCP server requests to help identify differences in HTTP requests. Useful for troubleshooting authentication and API issues.

**Usage:**
```bash
# Set required environment variables
export JIRA_EMAIL="your-email@example.com"
export JIRA_PERSONAL_ACCESS_TOKEN="your-token"
export JIRA_API_BASE_URL="https://your-domain.atlassian.net/rest/api/2"

# Run the debug script
./debug_curl.sh
```

### `debug_auth.rs`
A Rust binary for testing Jira authentication directly. This tool tests both the `/myself` endpoint and search functionality to verify authentication is working correctly.

**Usage:**
```bash
# Set required environment variables (same as above)
export JIRA_EMAIL="your-email@example.com"
export JIRA_PERSONAL_ACCESS_TOKEN="your-token"
export JIRA_API_BASE_URL="https://your-domain.atlassian.net/rest/api/2"

# Compile and run
cargo run --bin debug_auth
```

## When to Use

These tools are helpful when:
- Troubleshooting authentication issues
- Debugging API request/response differences
- Verifying Jira API connectivity
- Developing new features that interact with Jira

## Environment Variables

Both tools require the same environment variables:
- `JIRA_EMAIL`: Your Jira account email
- `JIRA_PERSONAL_ACCESS_TOKEN`: Your Jira personal access token
- `JIRA_API_BASE_URL`: Your Jira instance API base URL
