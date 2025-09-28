use crate::types::mcp::{MCPContent, MCPToolResult};
use serde_json::json;

/// Format a successful response
#[must_use]
pub fn format_success_response(message: &str, data: &serde_json::Value) -> MCPToolResult {
    let response = json!({
        "success": true,
        "message": message,
        "data": data
    });

    MCPToolResult {
        content: vec![MCPContent::text(
            serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string()),
        )],
        is_error: Some(false),
    }
}

/// Format an error response
#[must_use]
pub fn format_error_response(message: &str, error: Option<serde_json::Value>) -> MCPToolResult {
    let mut response = json!({
        "success": false,
        "message": message
    });

    if let Some(error_data) = error {
        response["error"] = error_data;
    }

    MCPToolResult {
        content: vec![MCPContent::text(
            serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string()),
        )],
        is_error: Some(true),
    }
}

/// Format a validation error response
#[must_use]
pub fn format_validation_error(field: &str, message: &str) -> MCPToolResult {
    let response = json!({
        "success": false,
        "message": "Validation error",
        "field": field,
        "error": message
    });

    MCPToolResult {
        content: vec![MCPContent::text(
            serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string()),
        )],
        is_error: Some(true),
    }
}
