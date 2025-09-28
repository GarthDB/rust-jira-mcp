# Rust Jira MCP Server

A Model Context Protocol (MCP) server implementation for Jira integration, built in Rust. This server provides comprehensive tools for interacting with Jira APIs through MCP-compatible clients.

## Features

### Core Jira Operations
- **Issue Management**: Create, read, update, search, and transition issues
- **Comments**: Add and retrieve comments on issues
- **Authentication**: Test Jira API connectivity and authentication

### Project Configuration and Metadata (NEW!)
- **Project Configuration**: Retrieve detailed project configuration settings
- **Issue Types**: Get issue types available for specific projects
- **Issue Type Metadata**: Get detailed information about specific issue types
- **Project Components**: Retrieve components associated with projects
- **Priorities & Statuses**: Get all available priorities and statuses
- **Custom Fields**: Retrieve custom field definitions
- **Comprehensive Metadata**: Get all project metadata in a single call

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd rust-jira-mcp
```

2. Build the project:
```bash
cargo build --release
```

3. Configure your Jira credentials in the configuration file or environment variables.

## Configuration

The server requires the following configuration:

```rust
JiraConfig {
    api_base_url: "https://your-jira-instance.atlassian.net/rest/api/2".to_string(),
    email: "your-email@example.com".to_string(),
    personal_access_token: "your-token".to_string(),
    default_project: Some("PROJECT_KEY".to_string()),
    max_results: Some(50),
    timeout_seconds: Some(30),
    log_file: None,
    strict_ssl: Some(true),
}
```

## Available Tools

### Core Tools
- `test_jira_auth` - Test authentication with Jira API
- `search_jira_issues` - Search for issues using JQL
- `create_jira_issue` - Create new issues
- `update_jira_issue` - Update existing issues
- `get_jira_issue` - Get issue details
- `get_jira_comments` - Get issue comments
- `add_jira_comment` - Add comments to issues
- `get_jira_transitions` - Get available transitions
- `transition_jira_issue` - Transition issues to new status

### Project Configuration and Metadata Tools
- `get_project_config` - Get project configuration details
- `get_project_issue_types` - Get issue types for a project
- `get_issue_type_metadata` - Get detailed issue type information
- `get_project_components` - Get project components
- `get_priorities_and_statuses` - Get all priorities and statuses
- `get_custom_fields` - Get custom field definitions
- `get_project_metadata` - Get comprehensive project metadata

## Usage Examples

### Basic Issue Operations

```json
{
  "method": "tools/call",
  "params": {
    "name": "search_jira_issues",
    "arguments": {
      "jql": "project = TEST AND status = Open",
      "max_results": 10
    }
  }
}
```

### Project Metadata Operations

```json
{
  "method": "tools/call",
  "params": {
    "name": "get_project_metadata",
    "arguments": {
      "project_key": "TEST"
    }
  }
}
```

### Get Issue Types for a Project

```json
{
  "method": "tools/call",
  "params": {
    "name": "get_project_issue_types",
    "arguments": {
      "project_key": "TEST"
    }
  }
}
```

### Get All Priorities and Statuses

```json
{
  "method": "tools/call",
  "params": {
    "name": "get_priorities_and_statuses",
    "arguments": {}
  }
}
```

## Running the Server

### As a Standalone Binary
```bash
cargo run --release
```

### As an MCP Server
The server implements the MCP protocol and can be used with any MCP-compatible client.

## Examples

See the `examples/` directory for usage examples:
- `project_metadata_example.rs` - Demonstrates the new project configuration and metadata tools

## Testing

Run the test suite:
```bash
cargo test
```

## Development

### Project Structure
```
src/
├── config/          # Configuration management
├── error/           # Error handling
├── jira/            # Jira API client
│   ├── client.rs    # Main client implementation
│   └── operations/  # Specific operation modules
├── mcp/             # MCP server implementation
│   ├── server.rs    # MCP server
│   └── tools.rs     # Tool implementations
├── types/           # Type definitions
└── utils/           # Utility functions
```

### Adding New Tools

1. Implement the tool struct in `src/mcp/tools.rs`
2. Add the tool to the server registration in `src/mcp/server.rs`
3. Add the tool definition to the `list_tools()` method
4. Add corresponding client methods in `src/jira/client.rs` if needed

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

[Add your license information here]

## Changelog

### Version 0.1.0
- Initial release with core Jira operations
- Added project configuration and metadata tools
- Comprehensive MCP server implementation
- Full test coverage
