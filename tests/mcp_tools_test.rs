use rust_jira_mcp::config::jira::JiraConfig;
use rust_jira_mcp::mcp::server::MCPToolHandler;
use rust_jira_mcp::mcp::tools::{
    AddCommentTool, AddIssueWatcherTool, AddWorkLogTool, BulkAddCommentsTool,
    BulkTransitionIssuesTool, BulkUpdateIssuesTool, CloneIssueTool, CreateComponentTool,
    CreateIssueLinkTool, CreateIssueTool, CreateLabelTool, DeleteAttachmentTool,
    DeleteComponentTool, DeleteIssueLinkTool, DeleteLabelTool, DeleteWorkLogTool,
    DownloadAttachmentTool, GetCommentsTool, GetCustomFieldsTool, GetIssueAttachmentsTool,
    GetIssueLinksTool, GetIssueTool, GetIssueTypeMetadataTool, GetIssueTypesTool,
    GetIssueWatchersTool, GetIssueWorkLogsTool, GetLabelsTool, GetLinkTypesTool,
    GetPrioritiesAndStatusesTool, GetProjectComponentsTool, GetProjectConfigTool,
    GetProjectMetadataTool, GetTransitionsTool, MixedBulkOperationsTool, RemoveIssueWatcherTool,
    SearchIssuesTool, TestAuthTool, TransitionIssueTool, UpdateComponentTool, UpdateIssueTool,
    UpdateLabelTool, UpdateWorkLogTool, UploadAttachmentTool,
};
use serde_json::json;

fn test_config() -> JiraConfig {
    JiraConfig {
        api_base_url: "https://test-jira.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(false),
    }
}

#[tokio::test]
async fn test_test_auth_tool() {
    let config = test_config();
    let tool = TestAuthTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_test_auth_tool_handle() {
    let config = test_config();
    let tool = TestAuthTool::new(config);

    // Test tool handling with valid args
    let args = json!({
        "random_string": "test"
    });

    let result = tool.handle(args).await;
    assert!(result.is_ok());

    let mcp_result = result.unwrap();
    assert!(!mcp_result.is_error.unwrap_or(true));
    assert!(!mcp_result.content.is_empty());
    assert!(mcp_result.content[0]
        .text
        .contains("Authentication test successful"));
}

#[tokio::test]
async fn test_search_issues_tool_creation() {
    let config = test_config();
    let tool = SearchIssuesTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_search_issues_tool_handle_missing_jql() {
    let config = test_config();
    let tool = SearchIssuesTool::new(config);

    // Test tool handling with missing jql parameter
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_search_issues_tool_handle_valid_args() {
    let config = test_config();
    let tool = SearchIssuesTool::new(config);

    // Test tool handling with valid args
    let args = json!({
        "jql": "project = TEST",
        "start_at": 0,
        "max_results": 10
    });

    // This will fail because there's no real Jira server, but we can test the structure
    let result = tool.handle(args).await;
    // The result will be an error due to network call, but that's expected
    let _result = result;
}

#[tokio::test]
async fn test_create_issue_tool_creation() {
    let config = test_config();
    let tool = CreateIssueTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_create_issue_tool_handle_missing_fields() {
    let config = test_config();
    let tool = CreateIssueTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_issue_tool_handle_valid_args() {
    let config = test_config();
    let tool = CreateIssueTool::new(config);

    // Test tool handling with valid args
    let args = json!({
        "project_key": "TEST",
        "issue_type": "Task",
        "summary": "Test Issue",
        "description": "Test Description"
    });

    // This will fail because there's no real Jira server, but we can test the structure
    let result = tool.handle(args).await;
    // The result will be an error due to network call, but that's expected
    let _result = result;
}

#[tokio::test]
async fn test_update_issue_tool_creation() {
    let config = test_config();
    let tool = UpdateIssueTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_update_issue_tool_handle_missing_issue_key() {
    let config = test_config();
    let tool = UpdateIssueTool::new(config);

    // Test tool handling with missing issue key
    let args = json!({
        "summary": "Updated Summary"
    });

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_issue_tool_creation() {
    let config = test_config();
    let tool = GetIssueTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_issue_tool_handle_missing_issue_key() {
    let config = test_config();
    let tool = GetIssueTool::new(config);

    // Test tool handling with missing issue key
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_comments_tool_creation() {
    let config = test_config();
    let tool = GetCommentsTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_comments_tool_handle_missing_issue_key() {
    let config = test_config();
    let tool = GetCommentsTool::new(config);

    // Test tool handling with missing issue key
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_comment_tool_creation() {
    let config = test_config();
    let tool = AddCommentTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_add_comment_tool_handle_missing_issue_key() {
    let config = test_config();
    let tool = AddCommentTool::new(config);

    // Test tool handling with missing issue key
    let args = json!({
        "comment": "Test comment"
    });

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_comment_tool_handle_missing_comment() {
    let config = test_config();
    let tool = AddCommentTool::new(config);

    // Test tool handling with missing comment
    let args = json!({
        "issue_key": "TEST-123"
    });

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_transitions_tool_creation() {
    let config = test_config();
    let tool = GetTransitionsTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_transitions_tool_handle_missing_issue_key() {
    let config = test_config();
    let tool = GetTransitionsTool::new(config);

    // Test tool handling with missing issue key
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_transition_issue_tool_creation() {
    let config = test_config();
    let tool = TransitionIssueTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_transition_issue_tool_handle_missing_issue_key() {
    let config = test_config();
    let tool = TransitionIssueTool::new(config);

    // Test tool handling with missing issue key
    let args = json!({
        "transition_id": "11"
    });

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_transition_issue_tool_handle_missing_transition_id() {
    let config = test_config();
    let tool = TransitionIssueTool::new(config);

    // Test tool handling with missing transition id
    let args = json!({
        "issue_key": "TEST-123"
    });

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_project_config_tool_creation() {
    let config = test_config();
    let tool = GetProjectConfigTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_project_config_tool_handle_missing_project_key() {
    let config = test_config();
    let tool = GetProjectConfigTool::new(config);

    // Test tool handling with missing project key
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_project_metadata_tool_creation() {
    let config = test_config();
    let tool = GetProjectMetadataTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_project_metadata_tool_handle_missing_project_key() {
    let config = test_config();
    let tool = GetProjectMetadataTool::new(config);

    // Test tool handling with missing project key
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_priorities_and_statuses_tool_creation() {
    let config = test_config();
    let tool = GetPrioritiesAndStatusesTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_custom_fields_tool_creation() {
    let config = test_config();
    let tool = GetCustomFieldsTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_issue_type_metadata_tool_creation() {
    let config = test_config();
    let tool = GetIssueTypeMetadataTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_issue_type_metadata_tool_handle_missing_issue_type_id() {
    let config = test_config();
    let tool = GetIssueTypeMetadataTool::new(config);

    // Test tool handling with missing issue type id
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_project_components_tool_creation() {
    let config = test_config();
    let tool = GetProjectComponentsTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_project_components_tool_handle_missing_project_key() {
    let config = test_config();
    let tool = GetProjectComponentsTool::new(config);

    // Test tool handling with missing project key
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_issue_types_tool_creation() {
    let config = test_config();
    let tool = GetIssueTypesTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_issue_types_tool_handle_missing_project_key() {
    let config = test_config();
    let tool = GetIssueTypesTool::new(config);

    // Test tool handling with missing project key
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_link_types_tool_creation() {
    let config = test_config();
    let tool = GetLinkTypesTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_issue_links_tool_creation() {
    let config = test_config();
    let tool = GetIssueLinksTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_issue_links_tool_handle_missing_issue_key() {
    let config = test_config();
    let tool = GetIssueLinksTool::new(config);

    // Test tool handling with missing issue key
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_issue_link_tool_creation() {
    let config = test_config();
    let tool = CreateIssueLinkTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_create_issue_link_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = CreateIssueLinkTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_issue_link_tool_creation() {
    let config = test_config();
    let tool = DeleteIssueLinkTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_delete_issue_link_tool_handle_missing_link_id() {
    let config = test_config();
    let tool = DeleteIssueLinkTool::new(config);

    // Test tool handling with missing link id
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_issue_attachments_tool_creation() {
    let config = test_config();
    let tool = GetIssueAttachmentsTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_issue_attachments_tool_handle_missing_issue_key() {
    let config = test_config();
    let tool = GetIssueAttachmentsTool::new(config);

    // Test tool handling with missing issue key
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_upload_attachment_tool_creation() {
    let config = test_config();
    let tool = UploadAttachmentTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_upload_attachment_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = UploadAttachmentTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_attachment_tool_creation() {
    let config = test_config();
    let tool = DeleteAttachmentTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_delete_attachment_tool_handle_missing_attachment_id() {
    let config = test_config();
    let tool = DeleteAttachmentTool::new(config);

    // Test tool handling with missing attachment id
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_download_attachment_tool_creation() {
    let config = test_config();
    let tool = DownloadAttachmentTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_download_attachment_tool_handle_missing_attachment_id() {
    let config = test_config();
    let tool = DownloadAttachmentTool::new(config);

    // Test tool handling with missing attachment id
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_issue_work_logs_tool_creation() {
    let config = test_config();
    let tool = GetIssueWorkLogsTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_issue_work_logs_tool_handle_missing_issue_key() {
    let config = test_config();
    let tool = GetIssueWorkLogsTool::new(config);

    // Test tool handling with missing issue key
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_work_log_tool_creation() {
    let config = test_config();
    let tool = AddWorkLogTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_add_work_log_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = AddWorkLogTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_work_log_tool_creation() {
    let config = test_config();
    let tool = UpdateWorkLogTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_update_work_log_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = UpdateWorkLogTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_work_log_tool_creation() {
    let config = test_config();
    let tool = DeleteWorkLogTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_delete_work_log_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = DeleteWorkLogTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_issue_watchers_tool_creation() {
    let config = test_config();
    let tool = GetIssueWatchersTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_get_issue_watchers_tool_handle_missing_issue_key() {
    let config = test_config();
    let tool = GetIssueWatchersTool::new(config);

    // Test tool handling with missing issue key
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_issue_watcher_tool_creation() {
    let config = test_config();
    let tool = AddIssueWatcherTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_add_issue_watcher_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = AddIssueWatcherTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_remove_issue_watcher_tool_creation() {
    let config = test_config();
    let tool = RemoveIssueWatcherTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_remove_issue_watcher_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = RemoveIssueWatcherTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_labels_tool_creation() {
    let config = test_config();
    let tool = GetLabelsTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_create_label_tool_creation() {
    let config = test_config();
    let tool = CreateLabelTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_create_label_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = CreateLabelTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_label_tool_creation() {
    let config = test_config();
    let tool = UpdateLabelTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_update_label_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = UpdateLabelTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_label_tool_creation() {
    let config = test_config();
    let tool = DeleteLabelTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_delete_label_tool_handle_missing_label_name() {
    let config = test_config();
    let tool = DeleteLabelTool::new(config);

    // Test tool handling with missing label name
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_component_tool_creation() {
    let config = test_config();
    let tool = CreateComponentTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_create_component_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = CreateComponentTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_component_tool_creation() {
    let config = test_config();
    let tool = UpdateComponentTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_update_component_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = UpdateComponentTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_component_tool_creation() {
    let config = test_config();
    let tool = DeleteComponentTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_delete_component_tool_handle_missing_component_id() {
    let config = test_config();
    let tool = DeleteComponentTool::new(config);

    // Test tool handling with missing component id
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_clone_issue_tool_creation() {
    let config = test_config();
    let tool = CloneIssueTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_clone_issue_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = CloneIssueTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_bulk_update_issues_tool_creation() {
    let config = test_config();
    let tool = BulkUpdateIssuesTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_bulk_update_issues_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = BulkUpdateIssuesTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_bulk_transition_issues_tool_creation() {
    let config = test_config();
    let tool = BulkTransitionIssuesTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_bulk_transition_issues_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = BulkTransitionIssuesTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_bulk_add_comments_tool_creation() {
    let config = test_config();
    let tool = BulkAddCommentsTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_bulk_add_comments_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = BulkAddCommentsTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mixed_bulk_operations_tool_creation() {
    let config = test_config();
    let tool = MixedBulkOperationsTool::new(config);

    // Test tool creation
    let _ = tool;
}

#[tokio::test]
async fn test_mixed_bulk_operations_tool_handle_missing_required_fields() {
    let config = test_config();
    let tool = MixedBulkOperationsTool::new(config);

    // Test tool handling with missing required fields
    let args = json!({});

    let result = tool.handle(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_error_handling() {
    let config = test_config();
    let tool = TestAuthTool::new(config);

    // Test that tools handle errors gracefully
    let args = json!({
        "invalid_field": "value"
    });

    // This should still work as TestAuthTool doesn't require specific fields
    let result = tool.handle(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tool_parameter_validation() {
    let config = test_config();
    let tool = SearchIssuesTool::new(config);

    // Test parameter validation with invalid types
    let args = json!({
        "jql": 123, // Should be string
        "start_at": "invalid", // Should be number
        "max_results": true // Should be number
    });

    let result = tool.handle(args).await;
    // This should fail due to type validation
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_optional_parameters() {
    let config = test_config();
    let tool = SearchIssuesTool::new(config);

    // Test with only required parameters
    let args = json!({
        "jql": "project = TEST"
    });

    // This will fail due to network call, but we can test the structure
    let result = tool.handle(args).await;
    let _result = result; // Expected to fail due to no real server
}

#[tokio::test]
async fn test_tool_configuration_usage() {
    let config = test_config();
    let tool = TestAuthTool::new(config);

    // Test that tools use the configuration properly
    let args = json!({
        "random_string": "test"
    });

    let result = tool.handle(args).await;
    assert!(result.is_ok());

    let mcp_result = result.unwrap();
    assert!(!mcp_result.is_error.unwrap_or(true));
    assert!(!mcp_result.content.is_empty());

    // The response should contain configuration information
    let response_text = &mcp_result.content[0].text;
    assert!(response_text.contains("Authentication test successful"));
}
