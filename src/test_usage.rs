// This module is used to test that our code compiles and uses the functions
// to avoid dead code warnings during development

use crate::config::jira::JiraConfig;
use crate::error::JiraError;
use crate::jira::client::JiraClient;
use crate::mcp::server::MCPServer;
use crate::types::jira::TransitionProperties;
use crate::types::mcp::{MCPContent, MCPToolCall};
use crate::utils::{format_error_response, format_success_response, format_validation_error};

pub fn test_usage() {
    // Test configuration
    let config = JiraConfig::default();

    // Test client creation and methods
    let client = JiraClient::new(config.clone()).unwrap();
    let _api_url = client.api_base_url();
    let _auth_header = client.auth_header();
    let _http_client = client.http_client();

    // Test server creation and methods
    let server = MCPServer::new(config.clone());
    let _tools = MCPServer::list_tools();

    // Test tool call
    let tool_call = MCPToolCall {
        name: "test".to_string(),
        arguments: serde_json::json!({}),
    };
    let _result = futures::executor::block_on(server.call_tool(tool_call));

    // Test error creation
    let _error = JiraError::auth_error("test");
    let _validation_error = JiraError::validation_error("field", "message");
    let _config_error = JiraError::config_error("config error");
    let _http_error = JiraError::HttpError {
        status: reqwest::StatusCode::BAD_REQUEST,
        message: "test".to_string(),
    };
    let _api_error = JiraError::api_error("test");
    let _unknown_error = JiraError::unknown_error("test");

    // Test from_jira_response
    let _jira_error = JiraError::from_jira_response(
        reqwest::StatusCode::BAD_REQUEST,
        &serde_json::json!({"errorMessages": ["test error"]}),
    );

    // Test transition properties
    let _props = TransitionProperties::new();
    let _all_props = TransitionProperties::with_all_properties();

    // Test MCP content
    let _content = MCPContent::text("test".to_string());

    // Test response formatting
    let _success = format_success_response("test", &serde_json::json!({}));
    let _error_resp = format_error_response("error", None);
    let _validation_resp = format_validation_error("field", "message");

    // Test config methods
    let _auth_header = config.auth_header();
    let _timeout = config.timeout_duration();
}
