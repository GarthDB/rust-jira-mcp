#!/bin/bash

# Debug script to compare curl vs MCP server requests
# This helps identify differences in HTTP requests

echo "=== Curl vs MCP Server Debug Tool ==="
echo

# Check if environment variables are set
if [ -z "$JIRA_EMAIL" ] || [ -z "$JIRA_PERSONAL_ACCESS_TOKEN" ] || [ -z "$JIRA_API_BASE_URL" ]; then
    echo "❌ Missing required environment variables:"
    echo "   JIRA_EMAIL: ${JIRA_EMAIL:-'NOT SET'}"
    echo "   JIRA_PERSONAL_ACCESS_TOKEN: ${JIRA_PERSONAL_ACCESS_TOKEN:-'NOT SET'}"
    echo "   JIRA_API_BASE_URL: ${JIRA_API_BASE_URL:-'NOT SET'}"
    echo
    echo "Please set these environment variables and try again."
    exit 1
fi

echo "✅ Environment variables loaded:"
echo "   JIRA_EMAIL: $JIRA_EMAIL"
echo "   JIRA_PERSONAL_ACCESS_TOKEN: ${JIRA_PERSONAL_ACCESS_TOKEN:0:8}..."
echo "   JIRA_API_BASE_URL: $JIRA_API_BASE_URL"
echo

# Test 1: /myself endpoint with curl
echo "=== Test 1: Curl /myself endpoint ==="
echo "Command: curl -H \"Authorization: Bearer \$TOKEN\" \"$JIRA_API_BASE_URL/myself\""
echo

curl -v -H "Authorization: Bearer $JIRA_PERSONAL_ACCESS_TOKEN" \
     -H "Content-Type: application/json" \
     -H "Accept: application/json" \
     -H "User-Agent: curl-debug/1.0" \
     "$JIRA_API_BASE_URL/myself" 2>&1 | head -50

echo
echo "=== Test 2: Curl search endpoint ==="
echo "Command: curl -H \"Authorization: Bearer \$TOKEN\" \"$JIRA_API_BASE_URL/search?jql=project=DNA%20AND%20status=Open&maxResults=1\""
echo

curl -v -H "Authorization: Bearer $JIRA_PERSONAL_ACCESS_TOKEN" \
     -H "Content-Type: application/json" \
     -H "Accept: application/json" \
     -H "User-Agent: curl-debug/1.0" \
     "$JIRA_API_BASE_URL/search?jql=project=DNA%20AND%20status=Open&maxResults=1" 2>&1 | head -50

echo
echo "=== Test 3: MCP Server Debug Tool ==="
echo "Running: cargo run --bin debug_auth"
echo

cargo run --bin debug_auth
