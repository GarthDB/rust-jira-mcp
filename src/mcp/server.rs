use crate::config::JiraConfig;
use crate::error::Result;
use crate::mcp::tools::{CreateIssueTool, SearchIssuesTool, TestAuthTool, UpdateIssueTool};
use crate::types::mcp::{MCPTool, MCPToolCall, MCPToolResult};
use serde_json::json;
use std::collections::HashMap;
use tracing::info;

pub struct MCPServer {
    config: JiraConfig,
    tools: HashMap<String, Box<dyn MCPToolHandler + Send + Sync>>,
}

#[async_trait::async_trait]
pub trait MCPToolHandler {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult>;
}

impl MCPServer {
    /// Create a new MCP server with the given configuration.
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        let mut tools: HashMap<String, Box<dyn MCPToolHandler + Send + Sync>> = HashMap::new();

        // Register all tools
        tools.insert(
            "test_jira_auth".to_string(),
            Box::new(TestAuthTool::new(config.clone())),
        );
        tools.insert(
            "search_jira_issues".to_string(),
            Box::new(SearchIssuesTool::new(config.clone())),
        );
        tools.insert(
            "create_jira_issue".to_string(),
            Box::new(CreateIssueTool::new(config.clone())),
        );
        tools.insert(
            "update_jira_issue".to_string(),
            Box::new(UpdateIssueTool::new(config.clone())),
        );
        // Add more tools as we implement them

        Self { config, tools }
    }

    /// Run the MCP server with stdio transport.
    ///
    /// # Errors
    ///
    /// Returns an error if the server cannot start or encounters a fatal error.
    pub async fn run_stdio(&mut self) -> Result<()> {
        info!("Starting MCP server with stdio transport");
        info!("Configuration: API URL = {}", self.config.api_base_url);

        // This is a placeholder - we'll need to implement the actual MCP protocol handling
        // For now, we'll just log that the server is running
        info!("MCP server is running and ready to accept requests");

        // In a real implementation, this would handle the MCP protocol over stdio
        // For now, we'll just keep the process alive
        tokio::signal::ctrl_c().await?;
        info!("Received shutdown signal, stopping server");

        Ok(())
    }

    #[must_use]
    pub fn list_tools() -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "test_jira_auth".to_string(),
                description: "Test authentication with Jira API".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "random_string": {
                            "type": "string",
                            "description": "Any string to test the connection"
                        }
                    },
                    "required": ["random_string"]
                }),
            },
            MCPTool {
                name: "search_jira_issues".to_string(),
                description: "Search for Jira issues using JQL (Jira Query Language)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "jql": {
                            "type": "string",
                            "description": "The JQL query string to search for issues"
                        },
                        "start_at": {
                            "type": "integer",
                            "description": "The index of the first item to return"
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "The maximum number of items to return"
                        },
                        "fields": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "List of fields to return for each issue"
                        }
                    },
                    "required": ["jql"]
                }),
            },
            MCPTool {
                name: "create_jira_issue".to_string(),
                description: "Create a new Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "fields": {
                            "type": "object",
                            "description": "The issue field values"
                        }
                    },
                    "required": ["fields"]
                }),
            },
            MCPTool {
                name: "update_jira_issue".to_string(),
                description: "Update an existing Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_id_or_key": {
                            "type": "string",
                            "description": "The ID or key of the issue to update"
                        },
                        "fields": {
                            "type": "object",
                            "description": "The fields to update"
                        }
                    },
                    "required": ["issue_id_or_key", "fields"]
                }),
            },
        ]
    }

    /// Call a tool by name with the given arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if the tool is not found or if the tool execution fails.
    pub async fn call_tool(&self, tool_call: MCPToolCall) -> Result<MCPToolResult> {
        if let Some(handler) = self.tools.get(&tool_call.name) {
            handler.handle(tool_call.arguments).await
        } else {
            Err(crate::error::JiraError::Unknown {
                message: format!("Unknown tool: {}", tool_call.name),
            })
        }
    }
}
