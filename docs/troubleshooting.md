# Troubleshooting Guide

This guide helps you diagnose and resolve common issues with the Rust Jira MCP Server.

## Table of Contents

- [Quick Diagnostics](#quick-diagnostics)
- [Authentication Issues](#authentication-issues)
- [Connection Problems](#connection-problems)
- [Configuration Issues](#configuration-issues)
- [API Errors](#api-errors)
- [Performance Issues](#performance-issues)
- [MCP Client Issues](#mcp-client-issues)
- [Debugging Tools](#debugging-tools)
- [Common Error Messages](#common-error-messages)
- [Getting Help](#getting-help)

## Quick Diagnostics

### 1. Test Authentication

First, verify your Jira connection:

```bash
cargo run --release -- --test-auth
```

**Expected Output:**
```
✅ Authentication successful!

Connection Details:
- Jira URL: https://your-company.atlassian.net
- User: your.email@company.com
- API Version: 2
- Response Time: 245ms
```

### 2. Check Configuration

Validate your configuration:

```bash
cargo run --release -- --validate-config
```

### 3. Enable Debug Logging

Get detailed information about what's happening:

```bash
RUST_LOG=debug cargo run --release
```

## Authentication Issues

### Problem: "Invalid credentials provided"

**Symptoms:**
- Authentication test fails
- All API calls return authentication errors
- Error message: "Invalid credentials provided"

**Solutions:**

1. **Check your email address:**
   ```bash
   echo $JIRA_EMAIL
   ```
   Ensure it matches your Jira account email exactly.

2. **Verify your personal access token:**
   ```bash
   echo $JIRA_PERSONAL_ACCESS_TOKEN
   ```
   - Token should be 24+ characters long
   - No spaces or special characters at the beginning/end
   - Token should be active (not expired)

3. **Test token manually:**
   ```bash
   curl -u "your.email@company.com:your_token" \
        "https://your-company.atlassian.net/rest/api/2/myself"
   ```
   
   **Note:** Jira Personal Access Tokens use Basic authentication (email:token), not Bearer authentication.

4. **Check token permissions:**
   - Go to Jira → Account Settings → Security → API tokens
   - Ensure token has appropriate permissions
   - Create a new token if needed

### Problem: "Authentication failed: 401 Unauthorized"

**Solutions:**

1. **Check Jira URL format:**
   ```bash
   # Correct format
   JIRA_API_BASE_URL=https://your-company.atlassian.net/rest/api/2
   
   # Incorrect formats
   JIRA_API_BASE_URL=https://your-company.atlassian.net  # Missing /rest/api/2
   JIRA_API_BASE_URL=https://your-company.atlassian.net/rest/api/3  # Wrong API version
   ```

2. **Verify SSL/TLS settings:**
   ```bash
   # For self-signed certificates
   JIRA_STRICT_SSL=false
   ```

3. **Check network connectivity:**
   ```bash
   ping your-company.atlassian.net
   curl -I https://your-company.atlassian.net
   ```

### Problem: "Token expired or invalid"

**Solutions:**

1. **Generate new token:**
   - Go to Jira → Account Settings → Security → API tokens
   - Delete old token
   - Create new token
   - Update your configuration

2. **Check token expiration:**
   - Some tokens have expiration dates
   - Create long-lived tokens for automation

## Connection Problems

### Problem: "Connection timeout"

**Symptoms:**
- Requests hang and eventually timeout
- Error: "Connection timeout after 30 seconds"

**Solutions:**

1. **Increase timeout:**
   ```bash
   JIRA_TIMEOUT_SECONDS=60
   ```

2. **Check network connectivity:**
   ```bash
   # Test basic connectivity
   ping your-company.atlassian.net
   
   # Test HTTPS connectivity
   curl -I https://your-company.atlassian.net
   
   # Test API endpoint
   curl -I https://your-company.atlassian.net/rest/api/2
   ```

3. **Check firewall/proxy settings:**
   - Ensure outbound HTTPS (443) is allowed
   - Configure proxy if behind corporate firewall
   - Check if Jira instance is accessible from your network

### Problem: "DNS resolution failed"

**Solutions:**

1. **Check DNS settings:**
   ```bash
   nslookup your-company.atlassian.net
   dig your-company.atlassian.net
   ```

2. **Try different DNS servers:**
   ```bash
   # Use Google DNS
   echo "nameserver 8.8.8.8" | sudo tee -a /etc/resolv.conf
   echo "nameserver 8.8.4.4" | sudo tee -a /etc/resolv.conf
   ```

3. **Check hosts file:**
   ```bash
   cat /etc/hosts | grep atlassian
   ```

### Problem: "SSL certificate verification failed"

**Solutions:**

1. **For self-signed certificates:**
   ```bash
   JIRA_STRICT_SSL=false
   ```

2. **Update CA certificates:**
   ```bash
   # Ubuntu/Debian
   sudo apt-get update && sudo apt-get install ca-certificates
   
   # macOS
   brew install ca-certificates
   
   # Windows
   # Update Windows certificates
   ```

3. **Check certificate validity:**
   ```bash
   openssl s_client -connect your-company.atlassian.net:443 -servername your-company.atlassian.net
   ```

## Configuration Issues

### Problem: "Configuration file not found"

**Solutions:**

1. **Check file path:**
   ```bash
   ls -la config/
   ls -la .env
   ```

2. **Use absolute path:**
   ```bash
   JIRA_CONFIG_FILE=/full/path/to/config.toml
   ```

3. **Create missing files:**
   ```bash
   cp config/default.toml config/local.toml
   cp env.example .env
   ```

### Problem: "Invalid configuration format"

**Solutions:**

1. **Validate TOML syntax:**
   ```bash
   # Install toml validator
   cargo install toml-cli
   
   # Validate config file
   toml get config/default.toml
   ```

2. **Check JSON syntax (if using JSON config):**
   ```bash
   python -m json.tool config/config.json
   ```

3. **Check YAML syntax (if using YAML config):**
   ```bash
   python -c "import yaml; yaml.safe_load(open('config/config.yaml'))"
   ```

### Problem: "Required field missing"

**Solutions:**

1. **Check environment variables:**
   ```bash
   env | grep JIRA_
   ```

2. **Verify .env file:**
   ```bash
   cat .env | grep -v "^#"
   ```

3. **Check configuration file:**
   ```bash
   cat config/local.toml
   ```

### Problem: "Configuration validation failed"

**Solutions:**

1. **Check field types:**
   ```toml
   # Correct
   max_results = 50
   timeout_seconds = 30
   strict_ssl = true
   
   # Incorrect
   max_results = "50"  # Should be number
   timeout_seconds = "30"  # Should be number
   strict_ssl = "true"  # Should be boolean
   ```

2. **Check required fields:**
   ```toml
   [default]
   email = "your.email@company.com"  # Required
   personal_access_token = "your_token"  # Required
   ```

3. **Validate email format:**
   ```bash
   # Email should contain @ and .
   echo $JIRA_EMAIL | grep -E "^[^@]+@[^@]+\.[^@]+$"
   ```

## API Errors

### Problem: "Issue not found (404)"

**Solutions:**

1. **Check issue key format:**
   ```bash
   # Correct format
   PROJ-123
   
   # Incorrect formats
   proj-123  # Wrong case
   PROJ123   # Missing dash
   123       # Missing project key
   ```

2. **Verify issue exists:**
   ```bash
   # Search for the issue
   curl -u "email:token" \
        "https://your-company.atlassian.net/rest/api/2/search?jql=key=PROJ-123"
   ```

3. **Check project permissions:**
   - Ensure you have access to the project
   - Verify the project key is correct

### Problem: "Invalid JQL query"

**Solutions:**

1. **Test JQL in Jira UI:**
   - Go to Issues → Search for issues
   - Test your JQL query in the advanced search
   - Copy the working query

2. **Common JQL mistakes:**
   ```jql
   # Correct
   project = PROJ AND status = "In Progress"
   
   # Incorrect
   project = PROJ AND status = In Progress  # Missing quotes
   project = proj AND status = "In Progress"  # Wrong case
   ```

3. **Use JQL validator:**
   ```bash
   # Test JQL with a simple search first
   curl -u "email:token" \
        "https://your-company.atlassian.net/rest/api/2/search?jql=project=PROJ&maxResults=1"
   ```

### Problem: "Field not found"

**Solutions:**

1. **Get available fields:**
   ```json
   {
     "method": "tools/call",
     "params": {
       "name": "get_custom_fields",
       "arguments": {}
     }
   }
   ```

2. **Check field names:**
   - Use exact field names from Jira
   - Custom fields use customfield_XXXXX format
   - System fields use standard names (summary, description, etc.)

3. **Get issue type metadata:**
   ```json
   {
     "method": "tools/call",
     "params": {
       "name": "get_issue_type_metadata",
       "arguments": {
         "project_key": "PROJ",
         "issue_type_id": "10001"
       }
     }
   }
   ```

### Problem: "Transition not available"

**Solutions:**

1. **Get available transitions:**
   ```json
   {
     "method": "tools/call",
     "params": {
       "name": "get_jira_transitions",
       "arguments": {
         "issue_key": "PROJ-123"
       }
     }
   }
   ```

2. **Check transition requirements:**
   - Some transitions require specific fields
   - Some transitions require permissions
   - Some transitions are only available in certain statuses

3. **Use transition name instead of ID:**
   ```json
   {
     "method": "tools/call",
     "params": {
       "name": "transition_jira_issue",
       "arguments": {
         "issue_key": "PROJ-123",
         "transition_name": "Start Progress"
       }
     }
   }
   ```

## Performance Issues

### Problem: "Request timeout"

**Solutions:**

1. **Increase timeout:**
   ```bash
   JIRA_TIMEOUT_SECONDS=60
   ```

2. **Reduce result set size:**
   ```json
   {
     "method": "tools/call",
     "params": {
       "name": "search_jira_issues",
       "arguments": {
         "jql": "project = PROJ",
         "max_results": 10
       }
     }
   }
   ```

3. **Use pagination:**
   ```json
   {
     "method": "tools/call",
     "params": {
       "name": "search_jira_issues",
       "arguments": {
         "jql": "project = PROJ",
         "max_results": 50,
         "start_at": 0
       }
     }
   }
   ```

### Problem: "Rate limit exceeded"

**Solutions:**

1. **Implement backoff:**
   ```rust
   // Add delay between requests
   tokio::time::sleep(Duration::from_millis(100)).await;
   ```

2. **Reduce request frequency:**
   - Use bulk operations instead of individual calls
   - Cache frequently accessed data
   - Batch multiple operations

3. **Check rate limits:**
   ```bash
   # Check response headers for rate limit info
   curl -I -u "email:token" \
        "https://your-company.atlassian.net/rest/api/2/myself"
   ```

### Problem: "Memory usage high"

**Solutions:**

1. **Limit result sets:**
   ```json
   {
     "method": "tools/call",
     "params": {
       "name": "search_jira_issues",
       "arguments": {
         "jql": "project = PROJ",
         "max_results": 100
       }
     }
   }
   ```

2. **Use specific fields:**
   ```json
   {
     "method": "tools/call",
     "params": {
       "name": "search_jira_issues",
       "arguments": {
         "jql": "project = PROJ",
         "fields": ["summary", "status", "assignee"]
       }
     }
   }
   ```

3. **Process data in chunks:**
   - Use pagination for large datasets
   - Process results incrementally
   - Avoid loading all data into memory

## MCP Client Issues

### Problem: "MCP connection failed"

**Solutions:**

1. **Check MCP client configuration:**
   ```json
   {
     "mcpServers": {
       "rust-jira-mcp": {
         "command": "/full/path/to/rust-jira-mcp",
         "args": [],
         "env": {
           "JIRA_EMAIL": "your.email@company.com",
           "JIRA_PERSONAL_ACCESS_TOKEN": "your_token"
         }
       }
     }
   }
   ```

2. **Test server manually:**
   ```bash
   # Run server in foreground
   cargo run --release
   ```

3. **Check server logs:**
   ```bash
   RUST_LOG=debug cargo run --release
   ```

### Problem: "Tool not found"

**Solutions:**

1. **List available tools:**
   ```json
   {
     "method": "tools/list",
     "params": {}
   }
   ```

2. **Check tool name spelling:**
   - Use exact tool names from documentation
   - Case-sensitive
   - No spaces or special characters

3. **Verify server version:**
   - Check if tool is available in your server version
   - Update server if needed

### Problem: "Invalid tool parameters"

**Solutions:**

1. **Check parameter names:**
   - Use exact parameter names from documentation
   - Case-sensitive
   - Required parameters must be provided

2. **Validate parameter types:**
   - Strings should be in quotes
   - Numbers should not be quoted
   - Booleans should be true/false

3. **Check parameter values:**
   - Ensure values are valid for the field
   - Check enum values for restricted fields
   - Verify date formats

## Debugging Tools

### 1. Enable Debug Logging

```bash
RUST_LOG=debug cargo run --release
```

### 2. Test Individual Components

```bash
# Test authentication only
cargo run --release -- --test-auth

# Validate configuration only
cargo run --release -- --validate-config

# Test specific tool
cargo run --release -- --test-tool search_jira_issues
```

### 3. Use Verbose Output

```bash
RUST_LOG=trace cargo run --release
```

### 4. Check System Resources

```bash
# Check memory usage
ps aux | grep rust-jira-mcp

# Check network connections
netstat -an | grep :443

# Check disk space
df -h
```

### 5. Monitor API Calls

```bash
# Enable request/response logging
RUST_LOG=debug cargo run --release 2>&1 | grep -E "(Request|Response)"
```

## Common Error Messages

### Authentication Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "Invalid credentials provided" | Wrong email/token | Check credentials |
| "Authentication failed: 401" | Invalid URL or token | Verify URL and token |
| "Token expired" | Token has expired | Generate new token |
| "Access denied" | Insufficient permissions | Check user permissions |

### Configuration Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "Configuration file not found" | Missing config file | Create config file |
| "Invalid configuration format" | Syntax error | Fix syntax |
| "Required field missing" | Missing required field | Add required field |
| "Invalid email format" | Bad email format | Fix email format |

### API Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "Issue not found (404)" | Invalid issue key | Check issue key |
| "Invalid JQL query" | Bad JQL syntax | Fix JQL query |
| "Field not found" | Invalid field name | Check field name |
| "Transition not available" | Invalid transition | Check available transitions |

### Network Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "Connection timeout" | Network issue | Check connectivity |
| "DNS resolution failed" | DNS issue | Check DNS settings |
| "SSL certificate error" | Certificate issue | Check SSL settings |
| "Rate limit exceeded" | Too many requests | Implement backoff |

## Getting Help

### 1. Check Documentation

- [Getting Started Guide](getting-started.md)
- [Tool Examples](tool-examples.md)
- [Configuration Guide](CONFIGURATION.md)
- [Performance Guide](performance.md)

### 2. Enable Debug Logging

```bash
RUST_LOG=debug cargo run --release
```

### 3. Check Logs

Look for error messages in:
- Console output
- Log files (if configured)
- System logs

### 4. Test with Minimal Configuration

```bash
# Test with minimal config
JIRA_EMAIL=your.email@company.com \
JIRA_PERSONAL_ACCESS_TOKEN=your_token \
cargo run --release -- --test-auth
```

### 5. Report Issues

When reporting issues, include:
- Error message
- Debug logs
- Configuration (without secrets)
- Steps to reproduce
- Expected vs actual behavior

### 6. Community Support

- **GitHub Issues**: Report bugs and request features
- **GitHub Discussions**: Ask questions and share ideas
- **Documentation**: Check existing docs first

## Prevention Tips

1. **Test authentication regularly**
2. **Keep tokens up to date**
3. **Use environment variables for secrets**
4. **Validate configuration before deployment**
5. **Monitor API rate limits**
6. **Use bulk operations when possible**
7. **Implement proper error handling**
8. **Keep server updated**
9. **Test with small datasets first**
10. **Document your configuration**

Remember: Most issues can be resolved by checking authentication, configuration, and network connectivity first!
