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

The server includes a comprehensive configuration management system with support for environment variables, configuration files, secret management, and hot-reloading.

### Quick Start

Create a `.env` file in your project root:

```bash
# Required
JIRA_EMAIL=your.email@company.com
JIRA_PERSONAL_ACCESS_TOKEN=your_personal_access_token_here

# Optional
JIRA_API_BASE_URL=https://jira.corp.adobe.com/rest/api/2
JIRA_DEFAULT_PROJECT=PROJ
JIRA_MAX_RESULTS=50
JIRA_TIMEOUT_SECONDS=30
```

### Configuration Features

- ✅ **Environment Variables**: Support for all configuration via environment variables
- ✅ **Configuration Files**: TOML, YAML, and JSON configuration file support
- ✅ **Secret Management**: Secure handling of sensitive data with multiple storage options
- ✅ **Configuration Validation**: Comprehensive validation with detailed error messages
- ✅ **Hot-Reloading**: Automatic configuration reloading when files change
- ✅ **Multiple Sources**: Support for multiple configuration sources with priority ordering
- ✅ **Default Values**: Sensible defaults for all configuration options

### Configuration Sources (Priority Order)

1. Environment Variables (Highest priority)
2. .env file
3. Custom config files (specified via `JIRA_CONFIG_FILE`)
4. config/local.toml
5. config/default.toml
6. Default values (Lowest priority)

For detailed configuration documentation, see [CONFIGURATION.md](CONFIGURATION.md).

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
- `simple_config_example.rs` - Shows basic configuration management usage
- `configuration_example.rs` - Comprehensive configuration system demonstration

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
