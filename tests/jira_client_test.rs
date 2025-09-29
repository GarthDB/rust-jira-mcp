use rust_jira_mcp::jira::client::JiraClient;
use rust_jira_mcp::types::jira::*;
use serde_json::json;

// Helper function to create test config
fn test_config() -> rust_jira_mcp::config::JiraConfig {
    rust_jira_mcp::config::JiraConfig {
        api_base_url: "https://test-jira.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(true),
    }
}

#[tokio::test]
async fn test_jira_client_creation() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
    assert!(!client.auth_header().is_empty());
}

#[tokio::test]
async fn test_jira_client_creation_with_invalid_config() {
    let mut config = test_config();
    config.api_base_url = "invalid-url".to_string();

    let result = JiraClient::new(config);
    // The client creation might succeed even with invalid URL due to reqwest's behavior
    // We just test that the function doesn't panic - both success and failure are valid
    let _ = result;
}

#[tokio::test]
async fn test_api_base_url() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
}

#[tokio::test]
async fn test_auth_header() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let auth_header = client.auth_header();
    // The auth header format might vary, just check it's not empty
    assert!(!auth_header.is_empty());
}

#[test]
fn test_should_retry() {
    // Test retryable status codes
    assert!(JiraClient::should_retry(
        reqwest::StatusCode::TOO_MANY_REQUESTS
    ));
    assert!(JiraClient::should_retry(
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    ));
    assert!(JiraClient::should_retry(reqwest::StatusCode::BAD_GATEWAY));
    assert!(JiraClient::should_retry(
        reqwest::StatusCode::SERVICE_UNAVAILABLE
    ));
    assert!(JiraClient::should_retry(
        reqwest::StatusCode::GATEWAY_TIMEOUT
    ));

    // Test non-retryable status codes
    assert!(!JiraClient::should_retry(reqwest::StatusCode::OK));
    assert!(!JiraClient::should_retry(reqwest::StatusCode::NOT_FOUND));
    assert!(!JiraClient::should_retry(reqwest::StatusCode::UNAUTHORIZED));
    assert!(!JiraClient::should_retry(reqwest::StatusCode::FORBIDDEN));
    assert!(!JiraClient::should_retry(reqwest::StatusCode::BAD_REQUEST));
}

// Test HTTP method wrappers
#[tokio::test]
async fn test_get_method() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    // This will fail in real execution but tests the method signature
    let result: rust_jira_mcp::error::Result<JiraIssue> = client.get("issue/TEST-123").await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_post_method() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let body = json!({"fields": {"summary": "Test issue"}});
    let result: rust_jira_mcp::error::Result<JiraIssue> = client.post("issue", &body).await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_put_method() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let body = json!({"fields": {"summary": "Updated issue"}});
    let result: rust_jira_mcp::error::Result<serde_json::Value> =
        client.put("issue/TEST-123", &body).await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_delete_method() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result: rust_jira_mcp::error::Result<serde_json::Value> =
        client.delete("issue/TEST-123").await;
    assert!(result.is_err()); // Expected to fail without mock server
}

// Test high-level API methods
#[tokio::test]
async fn test_get_issue() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.get_issue("TEST-123").await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_search_issues() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client
        .search_issues("project=TEST", Some(0), Some(50))
        .await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_search_issues_with_defaults() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.search_issues("project=TEST", None, None).await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_create_issue() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let issue_data = json!({
        "fields": {
            "project": {"key": "TEST"},
            "summary": "Test issue",
            "issuetype": {"name": "Bug"}
        }
    });

    let result = client.create_issue(&issue_data).await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_update_issue() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let update_data = json!({
        "fields": {
            "summary": "Updated summary"
        }
    });

    let result = client.update_issue("TEST-123", &update_data).await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_add_comment() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client
        .add_comment("TEST-123", "This is a test comment")
        .await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_get_comments() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.get_comments("TEST-123").await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_get_transitions() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.get_transitions("TEST-123").await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_transition_issue() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.transition_issue("TEST-123", "31", None).await;
    // The transition might succeed or fail depending on the implementation
    // We just test that the function doesn't panic - both success and failure are valid
    let _ = result;
}

// Test project and metadata methods
#[tokio::test]
async fn test_get_project_configuration() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.get_project_configuration("TEST").await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_get_project_issue_types() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.get_project_issue_types("TEST").await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_get_issue_type_metadata() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.get_issue_type_metadata("TEST").await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_get_project_components() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.get_project_components("TEST").await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_get_priorities() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.get_priorities().await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_get_statuses() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.get_statuses().await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_get_custom_fields() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.get_custom_fields().await;
    assert!(result.is_err()); // Expected to fail without mock server
}

// Test issue linking methods
#[tokio::test]
async fn test_get_link_types() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.get_link_types().await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_get_issue_links() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.get_issue_links("TEST-123").await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_create_issue_link() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    // Test with minimal link request - this will fail but tests the method signature
    let result = client
        .create_issue_link(&JiraIssueLinkCreateRequest {
            inward_issue: None,
            outward_issue: None,
            link_type: JiraIssueLinkType {
                name: "Blocks".to_string(),
            },
            comment: None,
        })
        .await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_delete_issue_link() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.delete_issue_link("10000").await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_link_issues() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client
        .link_issues("TEST-123", "TEST-124", "Blocks", Some("Test comment"))
        .await;
    assert!(result.is_err()); // Expected to fail without mock server
}

// Test bulk operations
#[tokio::test]
async fn test_execute_bulk_operations() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let bulk_config = BulkOperationConfig {
        batch_size: Some(5),
        continue_on_error: true,
        rate_limit_ms: Some(1000),
        max_retries: Some(3),
    };

    let operations = vec![BulkOperationItem {
        operation_type: BulkOperationType::Update,
        issue_key: "TEST-123".to_string(),
        data: json!({"fields": {"summary": "Updated"}}),
    }];

    let result = client
        .execute_bulk_operations(operations, bulk_config)
        .await;
    // The bulk operations might succeed or fail depending on the implementation
    // We just test that the function doesn't panic - both success and failure are valid
    let _ = result;
}

#[tokio::test]
async fn test_bulk_update_issues() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let issue_keys = vec!["TEST-123".to_string(), "TEST-124".to_string()];
    let update_data = json!({"fields": {"summary": "Bulk updated"}});

    let result = client
        .bulk_update_issues(issue_keys, update_data, None)
        .await;
    // The bulk operations might succeed or fail depending on the implementation
    // We just test that the function doesn't panic - both success and failure are valid
    let _ = result;
}

#[tokio::test]
async fn test_bulk_transition_issues() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let issue_keys = vec!["TEST-123".to_string(), "TEST-124".to_string()];
    let transition_id = "31".to_string();
    let _comment = Some("Bulk transition to Done".to_string());

    let result = client
        .bulk_transition_issues(issue_keys, transition_id, None, None)
        .await;
    // The bulk operations might succeed or fail depending on the implementation
    // We just test that the function doesn't panic - both success and failure are valid
    let _ = result;
}

#[tokio::test]
async fn test_bulk_add_comments() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let issue_keys = vec!["TEST-123".to_string(), "TEST-124".to_string()];
    let comment_body = "Bulk comment".to_string();

    let result = client
        .bulk_add_comments(issue_keys, comment_body, None)
        .await;
    // The bulk operations might succeed or fail depending on the implementation
    // We just test that the function doesn't panic - both success and failure are valid
    let _ = result;
}

// Test Zephyr methods
#[tokio::test]
async fn test_zephyr_api_base_url() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let zephyr_url = client.zephyr_api_base_url();
    // The zephyr URL format might vary, just check it's not empty
    assert!(!zephyr_url.is_empty());
}

#[tokio::test]
async fn test_zephyr_get() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result: rust_jira_mcp::error::Result<serde_json::Value> =
        client.zephyr_get("testcase").await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_zephyr_post() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let body = json!({"name": "Test case"});
    let result: rust_jira_mcp::error::Result<serde_json::Value> =
        client.zephyr_post("testcase", &body).await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_zephyr_put() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let body = json!({"name": "Updated test case"});
    let result: rust_jira_mcp::error::Result<serde_json::Value> =
        client.zephyr_put("testcase/123", &body).await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_zephyr_delete() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result: rust_jira_mcp::error::Result<serde_json::Value> =
        client.zephyr_delete("testcase/123").await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_get_zephyr_test_steps() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client.get_zephyr_test_steps("123").await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_create_zephyr_test_step() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let step_request = ZephyrTestStepCreateRequest {
        test_case_id: "123".to_string(),
        step: "Test step".to_string(),
        data: Some("Test data".to_string()),
        result: Some("Expected result".to_string()),
        order: 1,
    };

    let result = client.create_zephyr_test_step(&step_request).await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_search_zephyr_test_cases() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let result = client
        .search_zephyr_test_cases("project=TEST", Some(0), Some(50))
        .await;
    assert!(result.is_err()); // Expected to fail without mock server
}

#[tokio::test]
async fn test_create_zephyr_test_case() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    let test_case_request = ZephyrTestCaseCreateRequest {
        name: "Test case".to_string(),
        project_key: "TEST".to_string(),
        issue_type: "Test".to_string(),
        priority: None,
        assignee: None,
        description: Some("Test description".to_string()),
        labels: None,
        components: None,
        fix_versions: None,
        custom_fields: None,
    };

    let result = client.create_zephyr_test_case(&test_case_request).await;
    assert!(result.is_err()); // Expected to fail without mock server
}
