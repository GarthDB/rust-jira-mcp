use rust_jira_mcp::types::mcp::*;
use rust_jira_mcp::utils::response::*;
use serde_json::json;

#[test]
fn test_format_success_response() {
    let message = "Operation completed successfully";
    let data = json!({
        "id": "12345",
        "key": "TEST-123",
        "status": "Done"
    });

    let result = format_success_response(message, &data);

    // Check that it's not an error
    assert_eq!(result.is_error, Some(false));

    // Check that content is present
    assert_eq!(result.content.len(), 1);

    // Check content type
    assert_eq!(result.content[0].content_type, "text");
    assert!(result.content[0].text.contains("success"));
    assert!(result.content[0].text.contains("true"));
    assert!(result.content[0].text.contains(message));
    assert!(result.content[0].text.contains("12345"));
}

#[test]
fn test_format_success_response_with_empty_data() {
    let message = "Operation completed";
    let data = json!({});

    let result = format_success_response(message, &data);

    assert_eq!(result.is_error, Some(false));
    assert_eq!(result.content.len(), 1);

    assert_eq!(result.content[0].content_type, "text");
    assert!(result.content[0].text.contains("success"));
    assert!(result.content[0].text.contains("true"));
    assert!(result.content[0].text.contains(message));
}

#[test]
fn test_format_success_response_with_null_data() {
    let message = "Operation completed";
    let data = json!(null);

    let result = format_success_response(message, &data);

    assert_eq!(result.is_error, Some(false));
    assert_eq!(result.content.len(), 1);

    assert_eq!(result.content[0].content_type, "text");
    assert!(result.content[0].text.contains("success"));
    assert!(result.content[0].text.contains("true"));
    assert!(result.content[0].text.contains(message));
}

#[test]
fn test_format_error_response_without_error_data() {
    let message = "Operation failed";

    let result = format_error_response(message, None);

    // Check that it's an error
    assert_eq!(result.is_error, Some(true));

    // Check that content is present
    assert_eq!(result.content.len(), 1);

    // Check content type
    assert_eq!(result.content[0].content_type, "text");
    assert!(result.content[0].text.contains("success"));
    assert!(result.content[0].text.contains("false"));
    assert!(result.content[0].text.contains(message));
}

#[test]
fn test_format_error_response_with_error_data() {
    let message = "Operation failed";
    let error_data = json!({
        "code": "VALIDATION_ERROR",
        "details": "Invalid input parameters"
    });

    let result = format_error_response(message, Some(error_data));

    // Check that it's an error
    assert_eq!(result.is_error, Some(true));

    // Check that content is present
    assert_eq!(result.content.len(), 1);

    // Check content type
    assert_eq!(result.content[0].content_type, "text");
    assert!(result.content[0].text.contains("success"));
    assert!(result.content[0].text.contains("false"));
    assert!(result.content[0].text.contains(message));
    assert!(result.content[0].text.contains("VALIDATION_ERROR"));
    assert!(result.content[0].text.contains("Invalid input parameters"));
}

#[test]
fn test_format_validation_error() {
    let field = "email";
    let message = "Invalid email format";

    let result = format_validation_error(field, message);

    // Check that it's an error
    assert_eq!(result.is_error, Some(true));

    // Check that content is present
    assert_eq!(result.content.len(), 1);

    // Check content type
    assert_eq!(result.content[0].content_type, "text");
    assert!(result.content[0].text.contains("success"));
    assert!(result.content[0].text.contains("false"));
    assert!(result.content[0].text.contains("Validation error"));
    assert!(result.content[0].text.contains(field));
    assert!(result.content[0].text.contains(message));
}

#[test]
fn test_format_validation_error_with_empty_field() {
    let field = "";
    let message = "Field is required";

    let result = format_validation_error(field, message);

    assert_eq!(result.is_error, Some(true));
    assert_eq!(result.content.len(), 1);

    assert_eq!(result.content[0].content_type, "text");
    assert!(result.content[0].text.contains("Validation error"));
    assert!(result.content[0].text.contains(message));
}

#[test]
fn test_format_validation_error_with_empty_message() {
    let field = "password";
    let message = "";

    let result = format_validation_error(field, message);

    assert_eq!(result.is_error, Some(true));
    assert_eq!(result.content.len(), 1);

    assert_eq!(result.content[0].content_type, "text");
    assert!(result.content[0].text.contains("Validation error"));
    assert!(result.content[0].text.contains(field));
}

#[test]
fn test_mcp_content_text() {
    let text = "Hello, world!".to_string();
    let content = MCPContent::text(text.clone());

    assert_eq!(content.content_type, "text");
    assert_eq!(content.text, text);
}

#[test]
fn test_mcp_content_text_with_empty_string() {
    let text = "".to_string();
    let content = MCPContent::text(text.clone());

    assert_eq!(content.content_type, "text");
    assert_eq!(content.text, text);
}

#[test]
fn test_mcp_content_text_with_long_string() {
    let text = "This is a very long string that contains multiple lines and special characters: !@#$%^&*()_+-=[]{}|;':\",./<>?".to_string();
    let content = MCPContent::text(text.clone());

    assert_eq!(content.content_type, "text");
    assert_eq!(content.text, text);
}

#[test]
fn test_mcp_tool_result_serialization() {
    let result = MCPToolResult {
        content: vec![MCPContent::text("test content".to_string())],
        is_error: Some(false),
    };

    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: MCPToolResult = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.is_error, result.is_error);
    assert_eq!(deserialized.content.len(), result.content.len());
}

#[test]
fn test_mcp_tool_result_with_error() {
    let result = MCPToolResult {
        content: vec![MCPContent::text("error message".to_string())],
        is_error: Some(true),
    };

    assert_eq!(result.is_error, Some(true));
    assert_eq!(result.content.len(), 1);
}

#[test]
fn test_mcp_tool_result_without_error_flag() {
    let result = MCPToolResult {
        content: vec![MCPContent::text("neutral message".to_string())],
        is_error: None,
    };

    assert_eq!(result.is_error, None);
    assert_eq!(result.content.len(), 1);
}

#[test]
fn test_mcp_tool_result_with_multiple_content() {
    let result = MCPToolResult {
        content: vec![
            MCPContent::text("first message".to_string()),
            MCPContent::text("second message".to_string()),
        ],
        is_error: Some(false),
    };

    assert_eq!(result.content.len(), 2);
    assert_eq!(result.is_error, Some(false));
}
