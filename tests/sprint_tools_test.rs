use rust_jira_mcp::config::JiraConfig;
use rust_jira_mcp::mcp::server::MCPToolHandler;
use rust_jira_mcp::mcp::tools::sprints::*;
use serde_json::json;

#[tokio::test]
async fn test_sprint_tools_creation() {
    // Create a mock config for testing
    let config = JiraConfig {
        email: "test@example.com".to_string(),
        personal_access_token: "test_token".to_string(),
        api_base_url: "https://test.atlassian.net/rest/api/2".to_string(),
        default_project: None,
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(true),
    };

    // Test that all sprint tools can be created without panicking
    let _get_sprint_tool = GetSprintTool::new(config.clone());
    let _create_sprint_tool = CreateSprintTool::new(config.clone());
    let _add_issues_tool = AddIssuesToSprintTool::new(config.clone());
    let _get_sprint_issues_tool = GetSprintIssuesTool::new(config.clone());
    let _start_sprint_tool = StartSprintTool::new(config.clone());
    let _close_sprint_tool = CloseSprintTool::new(config.clone());
    let _get_board_sprints_tool = GetBoardSprintsTool::new(config.clone());

    // If we get here, all tools were created successfully
}

#[tokio::test]
async fn test_sprint_tool_parameter_validation() {
    let config = JiraConfig {
        email: "test@example.com".to_string(),
        personal_access_token: "test_token".to_string(),
        api_base_url: "https://test.atlassian.net/rest/api/2".to_string(),
        default_project: None,
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(true),
    };

    let get_sprint_tool = GetSprintTool::new(config);

    // Test missing sprint_id parameter
    let args = json!({});
    let result = get_sprint_tool.handle(args).await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e
            .to_string()
            .contains("Missing required parameter: sprint_id"));
    }
}

#[tokio::test]
async fn test_create_sprint_tool_parameter_validation() {
    let config = JiraConfig {
        email: "test@example.com".to_string(),
        personal_access_token: "test_token".to_string(),
        api_base_url: "https://test.atlassian.net/rest/api/2".to_string(),
        default_project: None,
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(true),
    };

    let create_sprint_tool = CreateSprintTool::new(config);

    // Test missing name parameter
    let args = json!({
        "rapid_view_id": 123
    });
    let result = create_sprint_tool.handle(args).await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("Missing required parameter: name"));
    }

    // Test missing rapid_view_id parameter
    let args = json!({
        "name": "Test Sprint"
    });
    let result = create_sprint_tool.handle(args).await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e
            .to_string()
            .contains("Missing required parameter: rapid_view_id"));
    }
}

#[tokio::test]
async fn test_add_issues_to_sprint_tool_parameter_validation() {
    let config = JiraConfig {
        email: "test@example.com".to_string(),
        personal_access_token: "test_token".to_string(),
        api_base_url: "https://test.atlassian.net/rest/api/2".to_string(),
        default_project: None,
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(true),
    };

    let add_issues_tool = AddIssuesToSprintTool::new(config);

    // Test missing sprint_id parameter
    let args = json!({
        "issues": ["TEST-1", "TEST-2"]
    });
    let result = add_issues_tool.handle(args).await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e
            .to_string()
            .contains("Missing required parameter: sprint_id"));
    }

    // Test missing issues parameter
    let args = json!({
        "sprint_id": 123
    });
    let result = add_issues_tool.handle(args).await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("Missing required parameter: issues"));
    }

    // Test empty issues array
    let args = json!({
        "sprint_id": 123,
        "issues": []
    });
    let result = add_issues_tool.handle(args).await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("No valid issue keys provided"));
    }
}
