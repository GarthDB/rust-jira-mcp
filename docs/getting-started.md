# Getting Started Guide

This guide will help you set up and start using the Rust Jira MCP Server quickly and effectively.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Running the Server](#running-the-server)
- [Using with MCP Clients](#using-with-mcp-clients)
- [First Steps](#first-steps)
- [Next Steps](#next-steps)

## Prerequisites

### System Requirements

- **Rust**: Version 1.70 or higher
- **Operating System**: Linux, macOS, or Windows
- **Memory**: At least 512MB RAM
- **Network**: Access to your Jira instance

### Jira Requirements

- **Jira Instance**: Cloud or Server/Data Center
- **API Access**: REST API enabled
- **Authentication**: Personal Access Token or Basic Auth
- **Permissions**: Appropriate permissions for the operations you want to perform

### MCP Client

You'll need an MCP-compatible client to interact with the server:
- **Claude Desktop** (recommended)
- **Custom MCP client**
- **Other MCP-compatible tools**

## Installation

### Option 1: From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/GarthDB/rust-jira-mcp.git
cd rust-jira-mcp

# Build the project
cargo build --release

# The binary will be available at target/release/rust-jira-mcp
```

### Option 2: Using Cargo (Future)

```bash
# This will be available once published to crates.io
cargo install rust-jira-mcp
```

## Configuration

### Quick Setup

1. **Copy the example environment file:**
   ```bash
   cp env.example .env
   ```

2. **Edit the `.env` file with your Jira credentials:**
   ```bash
   # Required
   JIRA_EMAIL=your.email@company.com
   JIRA_PERSONAL_ACCESS_TOKEN=your_personal_access_token_here
   
   # Optional
   JIRA_API_BASE_URL=https://your-company.atlassian.net/rest/api/2
   JIRA_DEFAULT_PROJECT=PROJ
   JIRA_MAX_RESULTS=50
   JIRA_TIMEOUT_SECONDS=30
   ```

3. **Test your configuration:**
   ```bash
   cargo run --release -- --test-auth
   ```

### Advanced Configuration

For more complex setups, see the [Configuration Guide](CONFIGURATION.md) for:
- Multiple configuration sources
- Secret management
- Hot-reloading
- Environment-specific configurations

## Running the Server

### Basic Usage

```bash
# Run the server
cargo run --release
```

The server will start and listen for MCP connections on the configured transport.

### Command Line Options

```bash
# Test authentication only
cargo run --release -- --test-auth

# Run with debug logging
RUST_LOG=debug cargo run --release

# Run with custom config file
JIRA_CONFIG_FILE=config/production.toml cargo run --release
```

### As a Service (Production)

For production deployments, consider running as a system service:

**systemd service example:**
```ini
[Unit]
Description=Rust Jira MCP Server
After=network.target

[Service]
Type=simple
User=jira-mcp
WorkingDirectory=/opt/rust-jira-mcp
ExecStart=/opt/rust-jira-mcp/target/release/rust-jira-mcp
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=JIRA_CONFIG_FILE=/opt/rust-jira-mcp/config/production.toml

[Install]
WantedBy=multi-user.target
```

## Using with MCP Clients

### Claude Desktop Setup

1. **Install Claude Desktop** from [Anthropic's website](https://claude.ai/download)

2. **Configure Claude Desktop** to use the MCP server:
   ```json
   {
     "mcpServers": {
       "rust-jira-mcp": {
         "command": "/path/to/rust-jira-mcp/target/release/rust-jira-mcp",
         "args": [],
         "env": {
           "JIRA_EMAIL": "your.email@company.com",
           "JIRA_PERSONAL_ACCESS_TOKEN": "your_token_here"
         }
       }
     }
   }
   ```

3. **Restart Claude Desktop** and verify the connection

### Custom MCP Client

If you're building a custom MCP client, the server implements the standard MCP protocol. See the [API Documentation](https://docs.rs/rust-jira-mcp) for details.

## First Steps

### 1. Test Authentication

First, verify your connection to Jira:

```json
{
  "method": "tools/call",
  "params": {
    "name": "test_jira_auth",
    "arguments": {}
  }
}
```

### 2. Search for Issues

Try searching for issues in your project:

```json
{
  "method": "tools/call",
  "params": {
    "name": "search_jira_issues",
    "arguments": {
      "jql": "project = YOUR_PROJECT AND status = Open",
      "max_results": 10
    }
  }
}
```

### 3. Get Project Information

Explore your project's configuration:

```json
{
  "method": "tools/call",
  "params": {
    "name": "get_project_metadata",
    "arguments": {
      "project_key": "YOUR_PROJECT"
    }
  }
}
```

### 4. Create a Test Issue

Create a simple test issue:

```json
{
  "method": "tools/call",
  "params": {
    "name": "create_jira_issue",
    "arguments": {
      "project_key": "YOUR_PROJECT",
      "issue_type": "Task",
      "summary": "Test issue from Rust Jira MCP",
      "description": "This is a test issue created using the Rust Jira MCP server."
    }
  }
}
```

## Available Tools

The server provides comprehensive tools for Jira interaction:

### Core Issue Management
- `test_jira_auth` - Test authentication
- `search_jira_issues` - Search issues with JQL
- `create_jira_issue` - Create new issues
- `update_jira_issue` - Update existing issues
- `get_jira_issue` - Get issue details
- `get_jira_comments` - Get issue comments
- `add_jira_comment` - Add comments to issues
- `get_jira_transitions` - Get available transitions
- `transition_jira_issue` - Transition issues

### Project Configuration
- `get_project_config` - Get project configuration
- `get_project_issue_types` - Get issue types
- `get_issue_type_metadata` - Get issue type details
- `get_project_components` - Get project components
- `get_priorities_and_statuses` - Get priorities and statuses
- `get_custom_fields` - Get custom field definitions
- `get_project_metadata` - Get comprehensive metadata

### Bulk Operations
- `bulk_create_issues` - Create multiple issues
- `bulk_update_issues` - Update multiple issues
- `bulk_transition_issues` - Transition multiple issues
- `bulk_add_comments` - Add comments to multiple issues

### Zephyr Test Management
- `get_test_cycles` - Get test cycles
- `create_test_cycle` - Create test cycles
- `get_test_executions` - Get test executions
- `update_test_execution` - Update test executions

## Next Steps

### Explore Examples

Check out the examples in the `examples/` directory:
- `project_metadata_example.rs` - Project configuration examples
- `bulk_operations_example.rs` - Bulk operation examples
- `configuration_example.rs` - Configuration management examples

### Read the Documentation

- **[Tool Examples](tool-examples.md)** - Detailed examples for each tool
- **[Configuration Guide](CONFIGURATION.md)** - Advanced configuration options
- **[Troubleshooting Guide](troubleshooting.md)** - Common issues and solutions
- **[Performance Guide](performance.md)** - Optimization tips

### Join the Community

- **GitHub Issues**: Report bugs or request features
- **Discussions**: Ask questions or share ideas
- **Contributing**: Help improve the project

## Common Use Cases

### 1. Issue Management Workflow

```json
// Search for issues
{
  "method": "tools/call",
  "params": {
    "name": "search_jira_issues",
    "arguments": {
      "jql": "project = PROJ AND assignee = currentUser() AND status = 'In Progress'"
    }
  }
}

// Update an issue
{
  "method": "tools/call",
  "params": {
    "name": "update_jira_issue",
    "arguments": {
      "issue_key": "PROJ-123",
      "fields": {
        "description": "Updated description"
      }
    }
  }
}

// Transition the issue
{
  "method": "tools/call",
  "params": {
    "name": "transition_jira_issue",
    "arguments": {
      "issue_key": "PROJ-123",
      "transition_id": "31"
    }
  }
}
```

### 2. Project Analysis

```json
// Get comprehensive project information
{
  "method": "tools/call",
  "params": {
    "name": "get_project_metadata",
    "arguments": {
      "project_key": "PROJ"
    }
  }
}

// Get all priorities and statuses
{
  "method": "tools/call",
  "params": {
    "name": "get_priorities_and_statuses",
    "arguments": {}
  }
}
```

### 3. Bulk Operations

```json
// Create multiple issues
{
  "method": "tools/call",
  "params": {
    "name": "bulk_create_issues",
    "arguments": {
      "issues": [
        {
          "project_key": "PROJ",
          "issue_type": "Task",
          "summary": "Task 1"
        },
        {
          "project_key": "PROJ",
          "issue_type": "Task",
          "summary": "Task 2"
        }
      ]
    }
  }
}
```

## Troubleshooting

If you encounter issues, check the [Troubleshooting Guide](troubleshooting.md) for common solutions.

### Quick Debug Steps

1. **Check authentication:**
   ```bash
   cargo run --release -- --test-auth
   ```

2. **Enable debug logging:**
   ```bash
   RUST_LOG=debug cargo run --release
   ```

3. **Verify configuration:**
   ```bash
   cargo run --release -- --validate-config
   ```

## Support

- **Documentation**: Check this guide and other docs
- **Issues**: Report bugs on GitHub
- **Discussions**: Ask questions in GitHub Discussions

Welcome to the Rust Jira MCP Server! 🚀
