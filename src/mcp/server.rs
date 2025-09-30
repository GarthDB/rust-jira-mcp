use crate::config::JiraConfig;
use crate::error::Result;
use crate::mcp::tools::{
    AddCommentTool,
    AddIssueWatcherTool,
    AddWorkLogTool,
    BulkAddCommentsTool,
    BulkTransitionIssuesTool,
    BulkUpdateIssuesTool,
    // Issue Cloning Tools
    CloneIssueTool,
    // Issue Component Tools
    CreateComponentTool,
    CreateIssueLinkTool,
    CreateIssueTool,
    CreateLabelTool,
    DeleteAttachmentTool,
    DeleteComponentTool,
    DeleteIssueLinkTool,
    DeleteLabelTool,
    DeleteWorkLogTool,
    DownloadAttachmentTool,
    GetCommentsTool,
    GetCustomFieldsTool,
    // File Attachment Tools
    GetIssueAttachmentsTool,
    GetIssueLinksTool,
    GetIssueTool,
    GetIssueTypeMetadataTool,
    GetIssueTypesTool,
    // Issue Watcher Tools
    GetIssueWatchersTool,
    // Work Log Tools
    GetIssueWorkLogsTool,
    // Issue Label Tools
    GetLabelsTool,
    // Issue Linking Tools
    GetLinkTypesTool,
    GetPrioritiesAndStatusesTool,
    GetProjectComponentsTool,
    GetProjectConfigTool,
    GetProjectMetadataTool,
    GetTransitionsTool,
    LinkIssuesTool,
    MixedBulkOperationsTool,
    RemoveIssueWatcherTool,
    SearchIssuesTool,
    TestAuthTool,
    TransitionIssueTool,
    UpdateComponentTool,
    UpdateIssueTool,
    UpdateLabelTool,
    UpdateWorkLogTool,
    UploadAttachmentTool,
};
use crate::mcp::zephyr_tools::{
    CreateZephyrTestCaseTool, CreateZephyrTestExecutionTool, CreateZephyrTestStepTool,
    DeleteZephyrTestStepTool, GetZephyrTestCasesTool, GetZephyrTestCyclesTool,
    GetZephyrTestExecutionsTool, GetZephyrTestPlansTool, GetZephyrTestStepsTool,
    UpdateZephyrTestStepTool,
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

        Self::register_basic_tools(&mut tools, &config);
        Self::register_project_tools(&mut tools, &config);
        Self::register_bulk_tools(&mut tools, &config);
        Self::register_linking_tools(&mut tools, &config);
        Self::register_attachment_tools(&mut tools, &config);
        Self::register_worklog_tools(&mut tools, &config);
        Self::register_watcher_tools(&mut tools, &config);
        Self::register_label_tools(&mut tools, &config);
        Self::register_component_tools(&mut tools, &config);
        Self::register_cloning_tools(&mut tools, &config);
        Self::register_zephyr_tools(&mut tools, &config);

        Self {
            config,
            tools,
            initialized: false,
        }
    }

    /// Register basic Jira tools
    fn register_basic_tools(
        tools: &mut HashMap<String, Box<dyn MCPToolHandler + Send + Sync>>,
        config: &JiraConfig,
    ) {
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
    }

    /// Register project configuration tools
    fn register_project_tools(
        tools: &mut HashMap<String, Box<dyn MCPToolHandler + Send + Sync>>,
        config: &JiraConfig,
    ) {
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
    }

    /// Register bulk operation tools
    fn register_bulk_tools(
        tools: &mut HashMap<String, Box<dyn MCPToolHandler + Send + Sync>>,
        config: &JiraConfig,
    ) {
        tools.insert(
            "bulk_update_issues".to_string(),
            Box::new(BulkUpdateIssuesTool::new(config.clone())),
        );
        tools.insert(
            "bulk_transition_issues".to_string(),
            Box::new(BulkTransitionIssuesTool::new(config.clone())),
        );
        tools.insert(
            "bulk_add_comments".to_string(),
            Box::new(BulkAddCommentsTool::new(config.clone())),
        );
        tools.insert(
            "mixed_bulk_operations".to_string(),
            Box::new(MixedBulkOperationsTool::new(config.clone())),
        );
    }

    /// Register issue linking tools
    fn register_linking_tools(
        tools: &mut HashMap<String, Box<dyn MCPToolHandler + Send + Sync>>,
        config: &JiraConfig,
    ) {
        tools.insert(
            "get_jira_link_types".to_string(),
            Box::new(GetLinkTypesTool::new(config.clone())),
        );
        tools.insert(
            "get_jira_issue_links".to_string(),
            Box::new(GetIssueLinksTool::new(config.clone())),
        );
        tools.insert(
            "create_jira_issue_link".to_string(),
            Box::new(CreateIssueLinkTool::new(config.clone())),
        );
        tools.insert(
            "link_jira_issues".to_string(),
            Box::new(LinkIssuesTool::new(config.clone())),
        );
        tools.insert(
            "delete_jira_issue_link".to_string(),
            Box::new(DeleteIssueLinkTool::new(config.clone())),
        );
    }

    /// Register file attachment tools
    fn register_attachment_tools(
        tools: &mut HashMap<String, Box<dyn MCPToolHandler + Send + Sync>>,
        config: &JiraConfig,
    ) {
        tools.insert(
            "get_jira_issue_attachments".to_string(),
            Box::new(GetIssueAttachmentsTool::new(config.clone())),
        );
        tools.insert(
            "upload_jira_attachment".to_string(),
            Box::new(UploadAttachmentTool::new(config.clone())),
        );
        tools.insert(
            "delete_jira_attachment".to_string(),
            Box::new(DeleteAttachmentTool::new(config.clone())),
        );
        tools.insert(
            "download_jira_attachment".to_string(),
            Box::new(DownloadAttachmentTool::new(config.clone())),
        );
    }

    /// Register work log tools
    fn register_worklog_tools(
        tools: &mut HashMap<String, Box<dyn MCPToolHandler + Send + Sync>>,
        config: &JiraConfig,
    ) {
        tools.insert(
            "get_jira_issue_work_logs".to_string(),
            Box::new(GetIssueWorkLogsTool::new(config.clone())),
        );
        tools.insert(
            "add_jira_work_log".to_string(),
            Box::new(AddWorkLogTool::new(config.clone())),
        );
        tools.insert(
            "update_jira_work_log".to_string(),
            Box::new(UpdateWorkLogTool::new(config.clone())),
        );
        tools.insert(
            "delete_jira_work_log".to_string(),
            Box::new(DeleteWorkLogTool::new(config.clone())),
        );
    }

    /// Register issue watcher tools
    fn register_watcher_tools(
        tools: &mut HashMap<String, Box<dyn MCPToolHandler + Send + Sync>>,
        config: &JiraConfig,
    ) {
        tools.insert(
            "get_jira_issue_watchers".to_string(),
            Box::new(GetIssueWatchersTool::new(config.clone())),
        );
        tools.insert(
            "add_jira_issue_watcher".to_string(),
            Box::new(AddIssueWatcherTool::new(config.clone())),
        );
        tools.insert(
            "remove_jira_issue_watcher".to_string(),
            Box::new(RemoveIssueWatcherTool::new(config.clone())),
        );
    }

    /// Register issue label tools
    fn register_label_tools(
        tools: &mut HashMap<String, Box<dyn MCPToolHandler + Send + Sync>>,
        config: &JiraConfig,
    ) {
        tools.insert(
            "get_jira_labels".to_string(),
            Box::new(GetLabelsTool::new(config.clone())),
        );
        tools.insert(
            "create_jira_label".to_string(),
            Box::new(CreateLabelTool::new(config.clone())),
        );
        tools.insert(
            "update_jira_label".to_string(),
            Box::new(UpdateLabelTool::new(config.clone())),
        );
        tools.insert(
            "delete_jira_label".to_string(),
            Box::new(DeleteLabelTool::new(config.clone())),
        );
    }

    /// Register issue component tools
    fn register_component_tools(
        tools: &mut HashMap<String, Box<dyn MCPToolHandler + Send + Sync>>,
        config: &JiraConfig,
    ) {
        tools.insert(
            "create_jira_component".to_string(),
            Box::new(CreateComponentTool::new(config.clone())),
        );
        tools.insert(
            "update_jira_component".to_string(),
            Box::new(UpdateComponentTool::new(config.clone())),
        );
        tools.insert(
            "delete_jira_component".to_string(),
            Box::new(DeleteComponentTool::new(config.clone())),
        );
    }

    /// Register issue cloning tools
    fn register_cloning_tools(
        tools: &mut HashMap<String, Box<dyn MCPToolHandler + Send + Sync>>,
        config: &JiraConfig,
    ) {
        tools.insert(
            "clone_jira_issue".to_string(),
            Box::new(CloneIssueTool::new(config.clone())),
        );
    }

    /// Register Zephyr test management tools
    fn register_zephyr_tools(
        tools: &mut HashMap<String, Box<dyn MCPToolHandler + Send + Sync>>,
        config: &JiraConfig,
    ) {
        tools.insert(
            "get_zephyr_test_steps".to_string(),
            Box::new(GetZephyrTestStepsTool::new(config.clone())),
        );
        tools.insert(
            "create_zephyr_test_step".to_string(),
            Box::new(CreateZephyrTestStepTool::new(config.clone())),
        );
        tools.insert(
            "update_zephyr_test_step".to_string(),
            Box::new(UpdateZephyrTestStepTool::new(config.clone())),
        );
        tools.insert(
            "delete_zephyr_test_step".to_string(),
            Box::new(DeleteZephyrTestStepTool::new(config.clone())),
        );
        tools.insert(
            "get_zephyr_test_cases".to_string(),
            Box::new(GetZephyrTestCasesTool::new(config.clone())),
        );
        tools.insert(
            "create_zephyr_test_case".to_string(),
            Box::new(CreateZephyrTestCaseTool::new(config.clone())),
        );
        tools.insert(
            "get_zephyr_test_executions".to_string(),
            Box::new(GetZephyrTestExecutionsTool::new(config.clone())),
        );
        tools.insert(
            "create_zephyr_test_execution".to_string(),
            Box::new(CreateZephyrTestExecutionTool::new(config.clone())),
        );
        tools.insert(
            "get_zephyr_test_cycles".to_string(),
            Box::new(GetZephyrTestCyclesTool::new(config.clone())),
        );
        tools.insert(
            "get_zephyr_test_plans".to_string(),
            Box::new(GetZephyrTestPlansTool::new(config.clone())),
        );
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
                    return Err(crate::error::JiraError::unknown_error(&format!(
                        "IO error: {e}"
                    )));
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

    /// Get basic tool definitions
    fn get_basic_tool_definitions() -> Vec<MCPTool> {
        let mut tools = Vec::new();
        tools.extend(Self::get_auth_and_search_tools());
        tools.extend(Self::get_issue_crud_tools());
        tools.extend(Self::get_comment_and_transition_tools());
        tools
    }

    /// Get authentication and search tools
    fn get_auth_and_search_tools() -> Vec<MCPTool> {
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
        ]
    }

    /// Get issue CRUD tools
    fn get_issue_crud_tools() -> Vec<MCPTool> {
        vec![
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
        ]
    }

    /// Get comment and transition tools
    fn get_comment_and_transition_tools() -> Vec<MCPTool> {
        vec![
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
        ]
    }

    /// Get project tool definitions
    fn get_project_tool_definitions() -> Vec<MCPTool> {
        vec![
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

    /// Get bulk tool definitions
    fn get_bulk_tool_definitions() -> Vec<MCPTool> {
        let mut tools = Vec::new();
        tools.extend(Self::get_simple_bulk_tools());
        tools.extend(Self::get_mixed_bulk_tools());
        tools
    }

    /// Get simple bulk operation tools
    fn get_simple_bulk_tools() -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "bulk_update_issues".to_string(),
                description: "Bulk update multiple Jira issues with the same fields".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_keys": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Array of issue keys to update"
                        },
                        "fields": {
                            "type": "object",
                            "description": "The fields to update on all issues"
                        },
                        "config": {
                            "type": "object",
                            "description": "Optional configuration for batch processing",
                            "properties": {
                                "batch_size": {"type": "integer", "description": "Number of issues to process per batch"},
                                "continue_on_error": {"type": "boolean", "description": "Whether to continue processing if individual operations fail"},
                                "rate_limit_ms": {"type": "integer", "description": "Delay between operations in milliseconds"},
                                "max_retries": {"type": "integer", "description": "Maximum number of retries for failed operations"}
                            }
                        }
                    },
                    "required": ["issue_keys", "fields"]
                }),
            },
            MCPTool {
                name: "bulk_transition_issues".to_string(),
                description: "Bulk transition multiple Jira issues to the same status".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_keys": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Array of issue keys to transition"
                        },
                        "transition_id": {
                            "type": "string",
                            "description": "The ID of the transition to apply to all issues"
                        },
                        "comment": {
                            "type": "string",
                            "description": "Optional comment to add during transition"
                        },
                        "config": {
                            "type": "object",
                            "description": "Optional configuration for batch processing",
                            "properties": {
                                "batch_size": {"type": "integer", "description": "Number of issues to process per batch"},
                                "continue_on_error": {"type": "boolean", "description": "Whether to continue processing if individual operations fail"},
                                "rate_limit_ms": {"type": "integer", "description": "Delay between operations in milliseconds"},
                                "max_retries": {"type": "integer", "description": "Maximum number of retries for failed operations"}
                            }
                        }
                    },
                    "required": ["issue_keys", "transition_id"]
                }),
            },
            MCPTool {
                name: "bulk_add_comments".to_string(),
                description: "Bulk add the same comment to multiple Jira issues".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_keys": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Array of issue keys to add comments to"
                        },
                        "comment_body": {
                            "type": "string",
                            "description": "The comment text to add to all issues"
                        },
                        "config": {
                            "type": "object",
                            "description": "Optional configuration for batch processing",
                            "properties": {
                                "batch_size": {"type": "integer", "description": "Number of issues to process per batch"},
                                "continue_on_error": {"type": "boolean", "description": "Whether to continue processing if individual operations fail"},
                                "rate_limit_ms": {"type": "integer", "description": "Delay between operations in milliseconds"},
                                "max_retries": {"type": "integer", "description": "Maximum number of retries for failed operations"}
                            }
                        }
                    },
                    "required": ["issue_keys", "comment_body"]
                }),
            },
        ]
    }

    /// Get mixed bulk operation tools
    fn get_mixed_bulk_tools() -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "mixed_bulk_operations".to_string(),
                description: "Execute mixed bulk operations on multiple Jira issues (update, transition, add comments, or mixed operations)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operations": {
                            "type": "array",
                            "description": "Array of operation objects",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "issue_key": {"type": "string", "description": "The issue key for this operation"},
                                    "operation_type": {
                                        "type": "string",
                                        "enum": ["update", "transition", "add_comment", "mixed"],
                                        "description": "The type of operation to perform"
                                    },
                                    "data": {
                                        "type": "object",
                                        "description": "The data for the operation (fields for update, transition_id for transition, comment_body for add_comment)"
                                    }
                                },
                                "required": ["issue_key", "operation_type", "data"]
                            }
                        },
                        "config": {
                            "type": "object",
                            "description": "Optional configuration for batch processing",
                            "properties": {
                                "batch_size": {"type": "integer", "description": "Number of issues to process per batch"},
                                "continue_on_error": {"type": "boolean", "description": "Whether to continue processing if individual operations fail"},
                                "rate_limit_ms": {"type": "integer", "description": "Delay between operations in milliseconds"},
                                "max_retries": {"type": "integer", "description": "Maximum number of retries for failed operations"}
                            }
                        }
                    },
                    "required": ["operations"]
                }),
            },
        ]
    }

    /// Get Zephyr tool definitions
    fn get_zephyr_tool_definitions() -> Vec<MCPTool> {
        let mut tools = Vec::new();
        tools.extend(Self::get_zephyr_test_step_tools());
        tools.extend(Self::get_zephyr_test_case_tools());
        tools.extend(Self::get_zephyr_execution_tools());
        tools.extend(Self::get_zephyr_cycle_and_plan_tools());
        tools
    }

    /// Get Zephyr test step tools
    fn get_zephyr_test_step_tools() -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "get_zephyr_test_steps".to_string(),
                description: "Get test steps for a Zephyr test case".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "test_case_id": {
                            "type": "string",
                            "description": "The ID of the test case to get steps for"
                        }
                    },
                    "required": ["test_case_id"]
                }),
            },
            MCPTool {
                name: "create_zephyr_test_step".to_string(),
                description: "Create a new test step in a Zephyr test case".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "test_case_id": {
                            "type": "string",
                            "description": "The ID of the test case to add the step to"
                        },
                        "step": {
                            "type": "string",
                            "description": "The test step description"
                        },
                        "order": {
                            "type": "integer",
                            "description": "The order of the test step"
                        },
                        "data": {
                            "type": "string",
                            "description": "Optional test data for the step"
                        },
                        "result": {
                            "type": "string",
                            "description": "Optional expected result for the step"
                        }
                    },
                    "required": ["test_case_id", "step", "order"]
                }),
            },
            MCPTool {
                name: "update_zephyr_test_step".to_string(),
                description: "Update an existing test step in a Zephyr test case".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "test_case_id": {
                            "type": "string",
                            "description": "The ID of the test case containing the step"
                        },
                        "step_id": {
                            "type": "string",
                            "description": "The ID of the test step to update"
                        },
                        "step": {
                            "type": "string",
                            "description": "The updated test step description"
                        },
                        "order": {
                            "type": "integer",
                            "description": "The updated order of the test step"
                        },
                        "data": {
                            "type": "string",
                            "description": "The updated test data for the step"
                        },
                        "result": {
                            "type": "string",
                            "description": "The updated expected result for the step"
                        }
                    },
                    "required": ["test_case_id", "step_id"]
                }),
            },
            MCPTool {
                name: "delete_zephyr_test_step".to_string(),
                description: "Delete a test step from a Zephyr test case".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "test_case_id": {
                            "type": "string",
                            "description": "The ID of the test case containing the step"
                        },
                        "step_id": {
                            "type": "string",
                            "description": "The ID of the test step to delete"
                        }
                    },
                    "required": ["test_case_id", "step_id"]
                }),
            },
        ]
    }

    /// Get Zephyr test case tools
    fn get_zephyr_test_case_tools() -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "get_zephyr_test_cases".to_string(),
                description: "Search for Zephyr test cases in a project".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_key": {
                            "type": "string",
                            "description": "The key of the project to search test cases in"
                        },
                        "start_at": {
                            "type": "integer",
                            "description": "The index of the first item to return"
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "The maximum number of items to return"
                        }
                    },
                    "required": ["project_key"]
                }),
            },
            MCPTool {
                name: "create_zephyr_test_case".to_string(),
                description: "Create a new Zephyr test case".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "The name of the test case"
                        },
                        "project_key": {
                            "type": "string",
                            "description": "The key of the project to create the test case in"
                        },
                        "issue_type": {
                            "type": "string",
                            "description": "The issue type for the test case"
                        },
                        "priority": {
                            "type": "string",
                            "description": "Optional priority for the test case"
                        },
                        "assignee": {
                            "type": "string",
                            "description": "Optional assignee for the test case"
                        },
                        "description": {
                            "type": "string",
                            "description": "Optional description for the test case"
                        }
                    },
                    "required": ["name", "project_key", "issue_type"]
                }),
            },
        ]
    }

    /// Get Zephyr execution tools
    fn get_zephyr_execution_tools() -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "get_zephyr_test_executions".to_string(),
                description: "Get test executions for a Zephyr test case".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "test_case_id": {
                            "type": "string",
                            "description": "The ID of the test case to get executions for"
                        }
                    },
                    "required": ["test_case_id"]
                }),
            },
            MCPTool {
                name: "create_zephyr_test_execution".to_string(),
                description: "Create a new test execution in Zephyr".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "test_case_id": {
                            "type": "string",
                            "description": "The ID of the test case to execute"
                        },
                        "project_id": {
                            "type": "string",
                            "description": "The ID of the project"
                        },
                        "status": {
                            "type": "string",
                            "description": "The execution status"
                        },
                        "cycle_id": {
                            "type": "string",
                            "description": "Optional test cycle ID"
                        },
                        "version_id": {
                            "type": "string",
                            "description": "Optional version ID"
                        },
                        "assignee": {
                            "type": "string",
                            "description": "Optional assignee for the execution"
                        },
                        "comment": {
                            "type": "string",
                            "description": "Optional comment for the execution"
                        }
                    },
                    "required": ["test_case_id", "project_id", "status"]
                }),
            },
        ]
    }

    /// Get Zephyr cycle and plan tools
    fn get_zephyr_cycle_and_plan_tools() -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "get_zephyr_test_cycles".to_string(),
                description: "Get test cycles for a Zephyr project".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_key": {
                            "type": "string",
                            "description": "The key of the project to get test cycles for"
                        }
                    },
                    "required": ["project_key"]
                }),
            },
            MCPTool {
                name: "get_zephyr_test_plans".to_string(),
                description: "Get test plans for a Zephyr project".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_key": {
                            "type": "string",
                            "description": "The key of the project to get test plans for"
                        }
                    },
                    "required": ["project_key"]
                }),
            },
        ]
    }

    /// Get linking tool definitions
    fn get_linking_tool_definitions() -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "get_jira_link_types".to_string(),
                description: "Get all available issue link types in Jira".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            MCPTool {
                name: "get_jira_issue_links".to_string(),
                description: "Get all links for a specific Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue to get links for"
                        }
                    },
                    "required": ["issue_key"]
                }),
            },
            MCPTool {
                name: "create_jira_issue_link".to_string(),
                description: "Create a link between two Jira issues".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "link_type": {
                            "type": "string",
                            "description": "The name of the link type"
                        },
                        "inward_issue": {
                            "type": "string",
                            "description": "The key of the inward issue"
                        },
                        "outward_issue": {
                            "type": "string",
                            "description": "The key of the outward issue"
                        },
                        "comment": {
                            "type": "string",
                            "description": "Optional comment for the link"
                        }
                    },
                    "required": ["link_type", "inward_issue", "outward_issue"]
                }),
            },
            MCPTool {
                name: "delete_jira_issue_link".to_string(),
                description: "Delete a link between Jira issues".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "link_id": {
                            "type": "string",
                            "description": "The ID of the link to delete"
                        }
                    },
                    "required": ["link_id"]
                }),
            },
        ]
    }

    /// Get attachment tool definitions
    fn get_attachment_tool_definitions() -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "get_jira_issue_attachments".to_string(),
                description: "Get all attachments for a specific Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue to get attachments for"
                        }
                    },
                    "required": ["issue_key"]
                }),
            },
            MCPTool {
                name: "upload_jira_attachment".to_string(),
                description: "Upload a file attachment to a Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue to attach the file to"
                        },
                        "filename": {
                            "type": "string",
                            "description": "The name of the file"
                        },
                        "content": {
                            "type": "string",
                            "description": "The file content as base64 encoded string"
                        }
                    },
                    "required": ["issue_key", "filename", "content"]
                }),
            },
            MCPTool {
                name: "delete_jira_attachment".to_string(),
                description: "Delete an attachment from a Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "attachment_id": {
                            "type": "string",
                            "description": "The ID of the attachment to delete"
                        }
                    },
                    "required": ["attachment_id"]
                }),
            },
            MCPTool {
                name: "download_jira_attachment".to_string(),
                description: "Download an attachment from a Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "attachment_id": {
                            "type": "string",
                            "description": "The ID of the attachment to download"
                        }
                    },
                    "required": ["attachment_id"]
                }),
            },
        ]
    }

    /// Get work log tool definitions
    fn get_worklog_tool_definitions() -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "get_jira_issue_work_logs".to_string(),
                description: "Get all work log entries for a specific Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue to get work logs for"
                        }
                    },
                    "required": ["issue_key"]
                }),
            },
            MCPTool {
                name: "add_jira_work_log".to_string(),
                description: "Add a work log entry to a Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue to add work log to"
                        },
                        "time_spent": {
                            "type": "string",
                            "description": "The time spent (e.g., '1h 30m', '2d', '3w')"
                        },
                        "comment": {
                            "type": "string",
                            "description": "Optional comment for the work log"
                        },
                        "started": {
                            "type": "string",
                            "description": "Optional start time in ISO 8601 format"
                        }
                    },
                    "required": ["issue_key", "time_spent"]
                }),
            },
            MCPTool {
                name: "update_jira_work_log".to_string(),
                description: "Update an existing work log entry".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue containing the work log"
                        },
                        "work_log_id": {
                            "type": "string",
                            "description": "The ID of the work log to update"
                        },
                        "time_spent": {
                            "type": "string",
                            "description": "The updated time spent"
                        },
                        "comment": {
                            "type": "string",
                            "description": "The updated comment"
                        },
                        "started": {
                            "type": "string",
                            "description": "The updated start time"
                        }
                    },
                    "required": ["issue_key", "work_log_id"]
                }),
            },
            MCPTool {
                name: "delete_jira_work_log".to_string(),
                description: "Delete a work log entry from a Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue containing the work log"
                        },
                        "work_log_id": {
                            "type": "string",
                            "description": "The ID of the work log to delete"
                        }
                    },
                    "required": ["issue_key", "work_log_id"]
                }),
            },
        ]
    }

    /// Get watcher tool definitions
    fn get_watcher_tool_definitions() -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "get_jira_issue_watchers".to_string(),
                description: "Get all watchers for a specific Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue to get watchers for"
                        }
                    },
                    "required": ["issue_key"]
                }),
            },
            MCPTool {
                name: "add_jira_issue_watcher".to_string(),
                description: "Add a watcher to a Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue to add watcher to"
                        },
                        "account_id": {
                            "type": "string",
                            "description": "The account ID of the user to add as watcher"
                        }
                    },
                    "required": ["issue_key", "account_id"]
                }),
            },
            MCPTool {
                name: "remove_jira_issue_watcher".to_string(),
                description: "Remove a watcher from a Jira issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_key": {
                            "type": "string",
                            "description": "The key of the issue to remove watcher from"
                        },
                        "account_id": {
                            "type": "string",
                            "description": "The account ID of the user to remove as watcher"
                        }
                    },
                    "required": ["issue_key", "account_id"]
                }),
            },
        ]
    }

    /// Get label tool definitions
    fn get_label_tool_definitions() -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "get_jira_labels".to_string(),
                description: "Get all available labels in Jira".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            MCPTool {
                name: "create_jira_label".to_string(),
                description: "Create a new label in Jira".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "The name of the label to create"
                        }
                    },
                    "required": ["name"]
                }),
            },
            MCPTool {
                name: "update_jira_label".to_string(),
                description: "Update an existing label in Jira".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "old_name": {
                            "type": "string",
                            "description": "The current name of the label"
                        },
                        "new_name": {
                            "type": "string",
                            "description": "The new name for the label"
                        }
                    },
                    "required": ["old_name", "new_name"]
                }),
            },
            MCPTool {
                name: "delete_jira_label".to_string(),
                description: "Delete a label from Jira".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "The name of the label to delete"
                        }
                    },
                    "required": ["name"]
                }),
            },
        ]
    }

    /// Get component tool definitions
    fn get_component_tool_definitions() -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "create_jira_component".to_string(),
                description: "Create a new component in a Jira project".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "The name of the component"
                        },
                        "project": {
                            "type": "string",
                            "description": "The key of the project to create the component in"
                        },
                        "description": {
                            "type": "string",
                            "description": "Optional description for the component"
                        },
                        "assignee_type": {
                            "type": "string",
                            "description": "Optional assignee type for the component"
                        },
                        "lead_account_id": {
                            "type": "string",
                            "description": "Optional lead account ID for the component"
                        }
                    },
                    "required": ["name", "project"]
                }),
            },
            MCPTool {
                name: "update_jira_component".to_string(),
                description: "Update an existing component in Jira".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "component_id": {
                            "type": "string",
                            "description": "The ID of the component to update"
                        },
                        "name": {
                            "type": "string",
                            "description": "The updated name of the component"
                        },
                        "description": {
                            "type": "string",
                            "description": "The updated description"
                        },
                        "assignee_type": {
                            "type": "string",
                            "description": "The updated assignee type"
                        },
                        "lead_account_id": {
                            "type": "string",
                            "description": "The updated lead account ID"
                        }
                    },
                    "required": ["component_id"]
                }),
            },
            MCPTool {
                name: "delete_jira_component".to_string(),
                description: "Delete a component from Jira".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "component_id": {
                            "type": "string",
                            "description": "The ID of the component to delete"
                        }
                    },
                    "required": ["component_id"]
                }),
            },
        ]
    }

    /// Get cloning tool definitions
    fn get_cloning_tool_definitions() -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "clone_jira_issue".to_string(),
                description: "Clone an existing Jira issue with optional copying of attachments, comments, work logs, watchers, links, and field mapping".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "original_issue_key": {
                            "type": "string",
                            "description": "The key of the issue to clone"
                        },
                        "project_key": {
                            "type": "string",
                            "description": "The key of the project to create the cloned issue in"
                        },
                        "issue_type": {
                            "type": "string",
                            "description": "The issue type for the cloned issue"
                        },
                        "summary": {
                            "type": "string",
                            "description": "The summary for the cloned issue"
                        },
                        "description": {
                            "type": "string",
                            "description": "Optional description for the cloned issue"
                        },
                        "copy_attachments": {
                            "type": "boolean",
                            "description": "Whether to copy attachments from the original issue"
                        },
                        "copy_comments": {
                            "type": "boolean",
                            "description": "Whether to copy comments from the original issue"
                        },
                        "copy_work_logs": {
                            "type": "boolean",
                            "description": "Whether to copy work logs from the original issue"
                        },
                        "copy_watchers": {
                            "type": "boolean",
                            "description": "Whether to copy watchers from the original issue"
                        },
                        "copy_links": {
                            "type": "boolean",
                            "description": "Whether to copy links from the original issue"
                        },
                        "field_mapping": {
                            "type": "object",
                            "description": "Field mapping configuration for cloning",
                            "properties": {
                                "copy_fields": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "List of field IDs to copy from the original issue (e.g., ['priority', 'labels', 'components'])"
                                },
                                "exclude_fields": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "List of field IDs to exclude from copying (e.g., ['assignee', 'reporter', 'status'])"
                                },
                                "custom_field_mapping": {
                                    "type": "object",
                                    "description": "Map original field IDs to new field IDs for custom field mapping",
                                    "additionalProperties": {"type": "string"}
                                }
                            }
                        }
                    },
                    "required": ["original_issue_key", "project_key", "issue_type", "summary"]
                }),
            },
        ]
    }

    #[must_use]
    pub fn list_tools() -> Vec<MCPTool> {
        let mut tools = Vec::new();
        tools.extend(Self::get_basic_tool_definitions());
        tools.extend(Self::get_project_tool_definitions());
        tools.extend(Self::get_bulk_tool_definitions());
        tools.extend(Self::get_zephyr_tool_definitions());
        tools.extend(Self::get_linking_tool_definitions());
        tools.extend(Self::get_attachment_tool_definitions());
        tools.extend(Self::get_worklog_tool_definitions());
        tools.extend(Self::get_watcher_tool_definitions());
        tools.extend(Self::get_label_tool_definitions());
        tools.extend(Self::get_component_tool_definitions());
        tools.extend(Self::get_cloning_tool_definitions());
        tools
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
            Err(crate::error::JiraError::unknown_error(&format!(
                "Unknown tool: {}",
                tool_call.name
            )))
        }
    }
}
