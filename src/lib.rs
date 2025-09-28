//! Rust Jira MCP Server
//!
//! A high-performance Rust-based Model Context Protocol (MCP) server for Jira API integration.
//! This server provides comprehensive tooling for issue management, bulk operations, and Zephyr test management.

pub mod config;
pub mod error;
pub mod jira;
pub mod mcp;
pub mod types;
pub mod utils;

pub use crate::config::jira::JiraConfig;
pub use crate::types::mcp::{
    CallToolParams, CallToolResult, ClientInfo, InitializeParams, InitializeResult, JsonRpcError,
    JsonRpcRequest, JsonRpcResponse, ListToolsParams, ListToolsResult, MCPContent, MCPTool,
    MCPToolCall, MCPToolResult, ServerCapabilities, ServerInfo, ToolsCapability,
};
pub use error::{JiraError, Result};
