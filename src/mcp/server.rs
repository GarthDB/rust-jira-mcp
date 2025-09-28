use crate::config::JiraConfig;
use crate::error::Result;
use crate::mcp::tools::{
    AddCommentTool, CreateIssueTool, GetCommentsTool, GetCustomFieldsTool, GetIssueTool,
    GetIssueTypeMetadataTool, GetIssueTypesTool, GetPrioritiesAndStatusesTool,
    GetProjectComponentsTool, GetProjectConfigTool, GetProjectMetadataTool, GetTransitionsTool,
    SearchIssuesTool, TestAuthTool, TransitionIssueTool, UpdateIssueTool,
};
use crate::types::mcp::{
    CallToolParams, CallToolResult, InitializeParams, InitializeResult, JsonRpcError,
    JsonRpcRequest, JsonRpcResponse, ListToolsParams, ListToolsResult, MCPTool, MCPToolCall,
    MCPToolResult, ServerCapabilities, ServerInfo, ToolsCapability,
};
use serde_json::json;
use std::collections::HashMap;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info, warn};

pub struct MCPServer {
    config: JiraConfig,
    tools: HashMap<String, Box<dyn MCPToolHandler + Send + Sync>>,
    initialized: bool,
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
        tools.insert(
            "get_jira_issue".to_string(),
            Box::new(GetIssueTool::new(config.clone())),
        );
        tools.insert(
            "get_jira_comments".to_string(),
            Box::new(GetCommentsTool::new(config.clone())),
        );
        tools.insert(
            "add_jira_comment".to_string(),
            Box::new(AddCommentTool::new(config.clone())),
        );
        tools.insert(
            "get_jira_transitions".to_string(),
            Box::new(GetTransitionsTool::new(config.clone())),
        );
        tools.insert(
            "transition_jira_issue".to_string(),
            Box::new(TransitionIssueTool::new(config.clone())),
        );

        // Project Configuration and Metadata tools
        tools.insert(
            "get_project_config".to_string(),
            Box::new(GetProjectConfigTool::new(config.clone())),
        );
        tools.insert(
            "get_project_issue_types".to_string(),
            Box::new(GetIssueTypesTool::new(config.clone())),
        );
        tools.insert(
            "get_issue_type_metadata".to_string(),
            Box::new(GetIssueTypeMetadataTool::new(config.clone())),
        );
        tools.insert(
            "get_project_components".to_string(),
            Box::new(GetProjectComponentsTool::new(config.clone())),
        );
        tools.insert(
            "get_priorities_and_statuses".to_string(),
            Box::new(GetPrioritiesAndStatusesTool::new(config.clone())),
        );
        tools.insert(
            "get_custom_fields".to_string(),
            Box::new(GetCustomFieldsTool::new(config.clone())),
        );
        tools.insert(
            "get_project_metadata".to_string(),
            Box::new(GetProjectMetadataTool::new(config.clone())),
        );

        Self {
            config,
            tools,
            initialized: false,
        }
    }

    /// Run the MCP server with stdio transport.
    ///
    /// # Errors
    ///
    /// Returns an error if the server cannot start or encounters a fatal error.
    pub async fn run_stdio(&mut self) -> Result<()> {
        info!("Starting MCP server with stdio transport");
        info!("Configuration: API URL = {}", self.config.api_base_url);

        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut writer = stdout;

        let mut line = String::new();

        info!("MCP server is running and ready to accept requests");

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    info!("EOF received, shutting down");
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    match self.handle_request(trimmed).await {
                        Ok(Some(response)) => {
                            let response_json = serde_json::to_string(&response)?;
                            writer.write_all(response_json.as_bytes()).await?;
                            writer.write_all(b"\n").await?;
                            writer.flush().await?;
                        }
                        Ok(None) => {
                            // Notification, no response needed
                        }
                        Err(e) => {
                            error!("Error handling request: {}", e);
                            // Send error response
                            let error_response = JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: None,
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32603,
                                    message: "Internal error".to_string(),
                                    data: Some(json!({ "details": e.to_string() })),
                                }),
                            };
                            let error_json = serde_json::to_string(&error_response)?;
                            writer.write_all(error_json.as_bytes()).await?;
                            writer.write_all(b"\n").await?;
                            writer.flush().await?;
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    return Err(crate::error::JiraError::Unknown {
                        message: format!("IO error: {e}"),
                    });
                }
            }
        }

        info!("MCP server shutdown complete");
        Ok(())
    }

    /// Handle a JSON-RPC request
    async fn handle_request(&mut self, request_str: &str) -> Result<Option<JsonRpcResponse>> {
        let request: JsonRpcRequest = serde_json::from_str(request_str)?;

        let response = match request.method.as_str() {
            "initialize" => Self::handle_initialize(request)?,
            "tools/list" => Self::handle_list_tools(request)?,
            "tools/call" => self.handle_call_tool(request).await?,
            "notifications/initialized" => {
                // Handle initialization notification
                self.initialized = true;
                info!("MCP client initialized successfully");
                return Ok(None);
            }
            _ => {
                warn!("Unknown method: {}", request.method);
                return Ok(Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: "Method not found".to_string(),
                        data: None,
                    }),
                }));
            }
        };

        Ok(Some(response))
    }

    /// Handle initialize request
    fn handle_initialize(request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let _params: InitializeParams = if let Some(params) = request.params {
            serde_json::from_value(params)?
        } else {
            return Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Invalid params".to_string(),
                    data: None,
                }),
            });
        };

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
            },
            server_info: ServerInfo {
                name: "rust-jira-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::to_value(result)?),
            error: None,
        })
    }

    /// Handle list tools request
    fn handle_list_tools(request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let _params: ListToolsParams = if let Some(params) = request.params {
            serde_json::from_value(params)?
        } else {
            return Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Invalid params".to_string(),
                    data: None,
                }),
            });
        };

        let tools = Self::list_tools();
        let result = ListToolsResult { tools };

        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::to_value(result)?),
            error: None,
        })
    }

    /// Handle call tool request
    async fn handle_call_tool(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let params: CallToolParams = if let Some(params) = request.params {
            serde_json::from_value(params)?
        } else {
            return Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Invalid params".to_string(),
                    data: None,
                }),
            });
        };

        let tool_call = MCPToolCall {
            name: params.name,
            arguments: params.arguments.unwrap_or(json!({})),
        };

        match self.call_tool(tool_call).await {
            Ok(tool_result) => {
                let result = CallToolResult {
                    content: tool_result.content,
                    is_error: tool_result.is_error.unwrap_or(false),
                };

                Ok(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::to_value(result)?),
                    error: None,
                })
            }
            Err(e) => {
                error!("Tool execution error: {}", e);
                Ok(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: "Tool execution failed".to_string(),
                        data: Some(json!({ "details": e.to_string() })),
                    }),
                })
            }
        }
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
            MCPTool {
                name: "get_jira_issue".to_string(),
                description: "Get details of a specific Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue to retrieve"
                        }
                    },
                    "required": ["issue_key"]
                }),
            },
            MCPTool {
                name: "get_jira_comments".to_string(),
                description: "Get all comments for a Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue to get comments for"
                        }
                    },
                    "required": ["issue_key"]
                }),
            },
            MCPTool {
                name: "add_jira_comment".to_string(),
                description: "Add a comment to a Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue to add a comment to"
                        },
                        "comment_body": {
                            "type": "string",
                            "description": "The comment text to add"
                        }
                    },
                    "required": ["issue_key", "comment_body"]
                }),
            },
            MCPTool {
                name: "get_jira_transitions".to_string(),
                description: "Get available transitions for a Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue to get transitions for"
                        }
                    },
                    "required": ["issue_key"]
                }),
            },
            MCPTool {
                name: "transition_jira_issue".to_string(),
                description: "Transition a Jira issue to a new status".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue to transition"
                        },
                        "transition_id": {
                            "type": "string",
                            "description": "The ID of the transition to apply"
                        },
                        "comment": {
                            "type": "string",
                            "description": "Optional comment to add during transition"
                        }
                    },
                    "required": ["issue_key", "transition_id"]
                }),
            },
            // Project Configuration and Metadata tools
            MCPTool {
                name: "get_project_config".to_string(),
                description: "Get project configuration details".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_key": {
                            "type": "string",
                            "description": "The key of the project to get configuration for"
                        }
                    },
                    "required": ["project_key"]
                }),
            },
            MCPTool {
                name: "get_project_issue_types".to_string(),
                description: "Get issue types for a specific project".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_key": {
                            "type": "string",
                            "description": "The key of the project to get issue types for"
                        }
                    },
                    "required": ["project_key"]
                }),
            },
            MCPTool {
                name: "get_issue_type_metadata".to_string(),
                description: "Get detailed metadata for a specific issue type".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_type_id": {
                            "type": "string",
                            "description": "The ID of the issue type to get metadata for"
                        }
                    },
                    "required": ["issue_type_id"]
                }),
            },
            MCPTool {
                name: "get_project_components".to_string(),
                description: "Get components for a specific project".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_key": {
                            "type": "string",
                            "description": "The key of the project to get components for"
                        }
                    },
                    "required": ["project_key"]
                }),
            },
            MCPTool {
                name: "get_priorities_and_statuses".to_string(),
                description: "Get all priorities and statuses available in Jira".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            MCPTool {
                name: "get_custom_fields".to_string(),
                description: "Get all custom fields available in Jira".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            MCPTool {
                name: "get_project_metadata".to_string(),
                description: "Get comprehensive project metadata including configuration, issue types, components, priorities, statuses, and custom fields".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_key": {
                            "type": "string",
                            "description": "The key of the project to get comprehensive metadata for"
                        }
                    },
                    "required": ["project_key"]
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
