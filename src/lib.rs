//! # Rust Jira MCP Server
//!
//! A high-performance Rust-based Model Context Protocol (MCP) server for comprehensive Jira API integration.
//! This server provides extensive tooling for issue management, project configuration, bulk operations, and Zephyr test management.
//!
//! ## Features
//!
//! ### Core Jira Operations
//! - **Issue Management**: Create, read, update, search, and transition issues
//! - **Comments**: Add and retrieve comments on issues
//! - **Authentication**: Test Jira API connectivity and authentication
//!
//! ### Project Configuration and Metadata
//! - **Project Configuration**: Retrieve detailed project configuration settings
//! - **Issue Types**: Get issue types available for specific projects
//! - **Issue Type Metadata**: Get detailed information about specific issue types
//! - **Project Components**: Retrieve components associated with projects
//! - **Priorities & Statuses**: Get all available priorities and statuses
//! - **Custom Fields**: Retrieve custom field definitions
//! - **Comprehensive Metadata**: Get all project metadata in a single call
//!
//! ### Bulk Operations
//! - **Bulk Issue Creation**: Create multiple issues at once
//! - **Bulk Issue Updates**: Update multiple issues simultaneously
//! - **Bulk Transitions**: Transition multiple issues to new statuses
//! - **Bulk Comments**: Add comments to multiple issues
//!
//! ### Zephyr Test Management
//! - **Test Cycles**: Manage test cycles and executions
//! - **Test Execution**: Update test execution status and results
//! - **Test Reporting**: Generate test reports and metrics
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rust_jira_mcp::config::JiraConfig;
//! use rust_jira_mcp::jira::client::JiraClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration
//!     let config = JiraConfig::load()?;
//!     
//!     // Create Jira client
//!     let client = JiraClient::new(config)?;
//!     
//!     // Test authentication by getting user info
//!     let user_info = client.get("myself").await?;
//!     println!("Authentication successful: {:?}", user_info);
//!     
//!     // Search for issues
//!     let issues = client.search_issues("project = PROJ AND status = Open", Some(10), None).await?;
//!     println!("Found {} issues", issues.total);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration
//!
//! The server supports multiple configuration sources with priority ordering:
//!
//! 1. Environment Variables (Highest priority)
//! 2. .env file
//! 3. Custom config files (specified via `JIRA_CONFIG_FILE`)
//! 4. config/local.toml
//! 5. config/default.toml
//! 6. Default values (Lowest priority)
//!
//! ### Environment Variables
//!
//! ```bash
//! # Required
//! JIRA_EMAIL=your.email@company.com
//! JIRA_PERSONAL_ACCESS_TOKEN=your_personal_access_token_here
//!
//! # Optional
//! JIRA_API_BASE_URL=https://your-company.atlassian.net/rest/api/2
//! JIRA_DEFAULT_PROJECT=PROJ
//! JIRA_MAX_RESULTS=50
//! JIRA_TIMEOUT_SECONDS=30
//! ```
//!
//! ## MCP Server Usage
//!
//! The server implements the Model Context Protocol (MCP) and can be used with MCP-compatible clients:
//!
//! ```json
//! {
//!   "method": "tools/call",
//!   "params": {
//!     "name": "search_jira_issues",
//!     "arguments": {
//!       "jql": "project = PROJ AND status = Open",
//!       "max_results": 10
//!     }
//!   }
//! }
//! ```
//!
//! ## Error Handling
//!
//! The library provides comprehensive error handling with detailed error types:
//!
//! ```rust,no_run
//! use rust_jira_mcp::error::{JiraError, Result};
//! use rust_jira_mcp::jira::client::JiraClient;
//! use rust_jira_mcp::config::JiraConfig;
//!
//! async fn handle_jira_operation() -> Result<()> {
//!     let config = JiraConfig::load()?;
//!     let client = JiraClient::new(config)?;
//!     
//!     match client.search_issues("invalid jql", Some(10), None).await {
//!         Ok(issues) => println!("Found {} issues", issues.total),
//!         Err(JiraError::ValidationError { field: _, message: msg }) => {
//!             eprintln!("Validation error: {}", msg);
//!         }
//!         Err(JiraError::ApiError { message: msg, error_codes: _ }) => {
//!             eprintln!("API error: {}", msg);
//!         }
//!         Err(e) => {
//!             eprintln!("Unexpected error: {}", e);
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Examples
//!
//! See the `examples/` directory for comprehensive usage examples:
//! - `project_metadata_example.rs` - Project configuration and metadata examples
//! - `bulk_operations_example.rs` - Bulk operation examples
//! - `configuration_example.rs` - Configuration management examples
//! - `simple_config_example.rs` - Basic configuration usage
//!
//! ## Performance
//!
//! The server is optimized for high performance:
//! - **Response Time**: 50-200ms for typical operations
//! - **Throughput**: 100-500 requests/minute
//! - **Memory Usage**: 10-50MB typical, 100MB+ for large operations
//! - **Concurrent Connections**: Up to 100 simultaneous connections
//!
//! See the [Performance Guide](docs/performance.md) for optimization tips.
//!
//! ## Documentation
//!
//! - **[Getting Started Guide](docs/getting-started.md)** - Complete setup and usage guide
//! - **[Tool Examples](docs/tool-examples.md)** - Detailed examples for all MCP tools
//! - **[Configuration Guide](CONFIGURATION.md)** - Comprehensive configuration management
//! - **[Troubleshooting Guide](docs/troubleshooting.md)** - Common issues and solutions
//! - **[Performance Guide](docs/performance.md)** - Optimization and benchmarking
//!
//! ## License
//!
//! This project is licensed under the MIT License - see the LICENSE file for details.

pub mod config;
pub mod error;
pub mod logging;
pub mod mcp;
pub mod performance;
pub mod jira;
pub mod types;
pub mod utils;

#[cfg(test)]
pub mod test_utils;

pub use crate::config::jira::JiraConfig;
pub use crate::types::mcp::{
    CallToolParams, CallToolResult, ClientInfo, InitializeParams, InitializeResult, JsonRpcError,
    JsonRpcRequest, JsonRpcResponse, ListToolsParams, ListToolsResult, MCPContent, MCPTool,
    MCPToolCall, MCPToolResult, ServerCapabilities, ServerInfo, ToolsCapability,
};
pub use error::{JiraError, Result};
