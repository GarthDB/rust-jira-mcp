# Rust Jira MCP Server

[![CI](https://github.com/GarthDB/rust-jira-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/GarthDB/rust-jira-mcp/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/GarthDB/rust-jira-mcp/branch/main/graph/badge.svg)](https://codecov.io/gh/GarthDB/rust-jira-mcp)
[![Coverage Status](https://img.shields.io/badge/coverage-75%25-brightgreen)](https://codecov.io/gh/GarthDB/rust-jira-mcp)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Rust-based Model Context Protocol (MCP) server for comprehensive Jira API integration. This server provides extensive tooling for issue management, project configuration, bulk operations, and Zephyr test management through MCP-compatible clients.

## 🚀 Quick Start

```bash
# Clone and build
git clone https://github.com/GarthDB/rust-jira-mcp.git
cd rust-jira-mcp
cargo build --release

# Configure your credentials
cp env.example .env
# Edit .env with your Jira credentials

# Run the server
cargo run --release
```

## 🧪 Testing & Development

This project includes a comprehensive xtask-based testing framework for safe development and testing:

### Testing Commands

```bash
# Test MCP operations
cargo run --package xtask -- test --suite read-only --project DNA
cargo run --package xtask -- test --suite issues --project DNA
cargo run --package xtask -- test --suite write --safe

# Collect test fixtures from live API
cargo run --package xtask -- collect-fixtures --project DNA

# Generate synthetic test data
cargo run --package xtask -- generate-fixtures --project TEST-MCP

# Run comprehensive test suite
cargo run --package xtask -- test-suite --project TEST-MCP

# Clean up test data
cargo run --package xtask -- cleanup --project TEST-MCP
```

### Makefile Shortcuts

```bash
# Quick test commands
make test-readonly    # Run read-only tests
make test-issues      # Run issue tests  
make test-write       # Run write tests (safe project only)
make test-suite       # Run comprehensive test suite
make test-cleanup     # Clean up test data

# Fixture management
make collect-fixtures # Collect real API data
make generate-fixtures # Generate synthetic data
```

### Safety Features

- **Safe Testing**: Use `--safe` flag or `TEST-MCP` project for write operations
- **Data Anonymization**: Automatic anonymization of sensitive data in fixtures
- **Cleanup**: Automatic cleanup of test data after operations
- **Dry Run**: Preview cleanup operations without making changes

## 📖 Documentation

- **[Documentation Index](docs/README.md)** - Complete documentation overview and navigation
- **[Getting Started Guide](docs/getting-started.md)** - Complete setup and usage guide
- **[API Documentation](https://docs.rs/rust-jira-mcp)** - Full API reference (generated with rustdoc)
- **[Configuration Guide](CONFIGURATION.md)** - Comprehensive configuration management
- **[Configuration Examples](docs/configuration-examples.md)** - Detailed configuration examples
- **[Tool Examples](docs/tool-examples.md)** - Detailed examples for all MCP tools
- **[Troubleshooting Guide](docs/troubleshooting.md)** - Common issues and solutions
- **[Performance Guide](docs/performance.md)** - Optimization and benchmarking

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

### 🔐 Authentication Setup

The server supports **two authentication methods** and automatically detects which one to use based on your token format:

#### Method 1: Adobe Jira (Bearer Token Authentication)
For Adobe Jira instances, use your Bearer token directly:

```bash
# Required for Adobe Jira
JIRA_EMAIL=your.email@adobe.com
JIRA_PERSONAL_ACCESS_TOKEN=YOUR_ADOBE_JIRA_TOKEN_HERE
JIRA_API_BASE_URL=https://jira.corp.adobe.com/rest/api/2
```

#### Method 2: Standard Jira (Basic Authentication)
For standard Atlassian Jira instances, use Basic authentication:

```bash
# Required for Standard Jira
JIRA_EMAIL=your.email@company.com
JIRA_PERSONAL_ACCESS_TOKEN=your_standard_pat_token
JIRA_API_BASE_URL=https://your-company.atlassian.net/rest/api/2
```

### 🧠 Smart Authentication Detection

The server automatically detects your authentication method:

- **Bearer Token**: If your token is long (>20 chars) and contains no colons
- **Basic Auth**: If your token is short or contains colons (like `user:password`)

### Quick Start

Create a `.env` file in your project root:

```bash
# Required - Choose one based on your Jira instance
JIRA_EMAIL=your.email@company.com
JIRA_PERSONAL_ACCESS_TOKEN=your_token_here

# Optional
JIRA_API_BASE_URL=https://your-company.atlassian.net/rest/api/2
JIRA_DEFAULT_PROJECT=PROJ
JIRA_MAX_RESULTS=50
JIRA_TIMEOUT_SECONDS=30
```

### 🔍 Testing Your Authentication

Test your authentication setup:

```bash
# Test with curl (Adobe Jira - Bearer)
curl -H "Authorization: Bearer YOUR_TOKEN" \
     "https://jira.corp.adobe.com/rest/api/2/myself"

# Test with curl (Standard Jira - Basic)
curl -u "your.email@company.com:YOUR_TOKEN" \
     "https://your-company.atlassian.net/rest/api/2/myself"

# Test with the MCP server
cargo run --release
# Then call the test_jira_auth tool
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

### Sprint Management Tools (NEW!)
- `get_sprint` - Get sprint details by sprint ID
- `create_sprint` - Create a new sprint
- `add_issues_to_sprint` - Add issues to a sprint
- `get_sprint_issues` - Get all issues in a sprint
- `start_sprint` - Start a sprint (set state to active)
- `close_sprint` - Close a sprint (set state to closed)
- `get_board_sprints` - Get all sprints for a board

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

### Sprint Management Operations

```json
{
  "method": "tools/call",
  "params": {
    "name": "get_sprint_issues",
    "arguments": {
      "sprint_id": 12345,
      "max_results": 50
    }
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

## Coverage

This project maintains high test coverage with comprehensive testing tools:

### Coverage Status
- **Overall Coverage**: ~75% (excluding test utilities)
- **Target Coverage**: 80%
- **Application Code**: 70-75% coverage

### Coverage Tools
```bash
# Quick coverage check
make coverage-check

# Detailed analysis
make coverage-analyze

# Get test suggestions for a module
make coverage-suggest MODULE=main

# Open HTML coverage report
make coverage-dashboard
```

### Coverage Monitoring
- **Codecov.io**: Continuous coverage monitoring
- **GitHub Actions**: Automated coverage reporting on PRs
- **Coverage Badges**: Real-time coverage status in README

For detailed coverage information, see [COVERAGE.md](COVERAGE.md).

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests (aim for 80%+ coverage)
5. Run `make coverage-check` to verify coverage
6. Submit a pull request

### Coverage Requirements
- New features must maintain or improve overall coverage
- Critical modules (main.rs, jira_client, mcp_tools) should have 80%+ coverage
- Use `make coverage-suggest MODULE=<name>` for guidance

## License

[Add your license information here]

## Changelog

### Version 0.1.0
- Initial release with core Jira operations
- Added project configuration and metadata tools
- Comprehensive MCP server implementation
- Full test coverage
