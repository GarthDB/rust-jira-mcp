//! Test helper functions and utilities

use crate::config::JiraConfig;
use crate::types::jira::*;
use serde_json::json;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Helper function to create a test Jira configuration with custom values
#[must_use]
pub fn create_test_config(
    api_base_url: Option<&str>,
    email: Option<&str>,
    token: Option<&str>,
    project: Option<&str>,
) -> JiraConfig {
    JiraConfig {
        api_base_url: api_base_url
            .unwrap_or("https://test-jira.example.com/rest/api/2")
            .to_string(),
        email: email.unwrap_or("test@example.com").to_string(),
        personal_access_token: token.unwrap_or("test-token-12345").to_string(),
        default_project: project.map(ToString::to_string),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(true),
    }
}

/// Helper function to create a Jira user for testing
#[must_use]
pub fn create_test_user(
    account_id: Option<&str>,
    display_name: Option<&str>,
    email: Option<&str>,
) -> JiraUser {
    JiraUser {
        account_id: account_id.unwrap_or("test-user-123").to_string(),
        display_name: display_name.unwrap_or("Test User").to_string(),
        email_address: email.map(ToString::to_string),
        active: true,
        time_zone: Some("UTC".to_string()),
    }
}

/// Helper function to create a Jira project for testing
#[must_use]
pub fn create_test_project(id: Option<&str>, key: Option<&str>, name: Option<&str>) -> JiraProject {
    JiraProject {
        id: id.unwrap_or("10000").to_string(),
        key: key.unwrap_or("TEST").to_string(),
        name: name.unwrap_or("Test Project").to_string(),
        project_type_key: "software".to_string(),
        self_url: "https://test-jira.example.com/rest/api/2/project/10000".to_string(),
    }
}

/// Helper function to create a Jira status for testing
#[must_use]
pub fn create_test_status(
    id: Option<&str>,
    name: Option<&str>,
    category: Option<&str>,
) -> JiraStatus {
    JiraStatus {
        id: id.unwrap_or("1").to_string(),
        name: name.unwrap_or("To Do").to_string(),
        description: Some("Test status description".to_string()),
        icon_url: Some("https://test-jira.example.com/images/icons/statuses/todo.svg".to_string()),
        status_category: JiraStatusCategory {
            id: 2,
            key: category.unwrap_or("new").to_string(),
            color_name: "blue-gray".to_string(),
            name: "To Do".to_string(),
        },
    }
}

/// Helper function to create a Jira priority for testing
#[must_use]
pub fn create_test_priority(id: Option<&str>, name: Option<&str>) -> JiraPriority {
    JiraPriority {
        id: id.unwrap_or("3").to_string(),
        name: name.unwrap_or("Medium").to_string(),
        description: Some("Medium priority".to_string()),
        icon_url: Some(
            "https://test-jira.example.com/images/icons/priorities/medium.svg".to_string(),
        ),
    }
}

/// Helper function to create a Jira issue type for testing
#[must_use]
pub fn create_test_issue_type(
    id: Option<&str>,
    name: Option<&str>,
    subtask: Option<bool>,
) -> JiraIssueType {
    JiraIssueType {
        id: id.unwrap_or("10001").to_string(),
        name: name.unwrap_or("Story").to_string(),
        description: Some("Test issue type description".to_string()),
        icon_url: Some(
            "https://test-jira.example.com/images/icons/issuetypes/story.svg".to_string(),
        ),
        subtask: subtask.unwrap_or(false),
    }
}

/// Helper function to create a complete Jira issue for testing
#[must_use]
pub fn create_test_issue(
    id: Option<&str>,
    key: Option<&str>,
    summary: Option<&str>,
    description: Option<&str>,
) -> JiraIssue {
    let issue_id = id.unwrap_or("12345");
    let issue_key = key.unwrap_or("TEST-123");

    let mut fields = HashMap::new();
    fields.insert(
        "summary".to_string(),
        serde_json::Value::String(summary.unwrap_or("Test Issue Summary").to_string()),
    );
    if let Some(desc) = description {
        fields.insert(
            "description".to_string(),
            serde_json::Value::String(desc.to_string()),
        );
    }

    JiraIssue {
        id: issue_id.to_string(),
        key: issue_key.to_string(),
        self_url: format!("https://test-jira.example.com/rest/api/2/issue/{issue_id}"),
        fields,
    }
}

/// Helper function to create a Jira comment for testing
#[must_use]
pub fn create_test_comment(
    id: Option<&str>,
    body: Option<&str>,
    author: Option<JiraUser>,
) -> JiraComment {
    JiraComment {
        id: id.unwrap_or("10001").to_string(),
        body: body.unwrap_or("Test comment").to_string(),
        author: author.unwrap_or_else(|| create_test_user(None, None, None)),
        created: current_timestamp(),
        updated: Some(current_timestamp()),
    }
}

/// Helper function to create a Jira work log for testing
#[must_use]
pub fn create_test_work_log(
    id: Option<&str>,
    comment: Option<&str>,
    time_spent: Option<&str>,
    time_spent_seconds: Option<i32>,
) -> JiraWorkLog {
    JiraWorkLog {
        id: id.unwrap_or("10001").to_string(),
        comment: comment.map(ToString::to_string),
        created: current_timestamp(),
        updated: Some(current_timestamp()),
        time_spent: time_spent.unwrap_or("1h 30m").to_string(),
        time_spent_seconds: time_spent_seconds.unwrap_or(5400),
        author: create_test_user(None, None, None),
    }
}

/// Helper function to create a Zephyr test case for testing
#[must_use]
pub fn create_test_zephyr_test_case(
    id: Option<&str>,
    key: Option<&str>,
    name: Option<&str>,
    project_key: Option<&str>,
) -> ZephyrTestCase {
    ZephyrTestCase {
        id: Some(id.unwrap_or("10001").to_string()),
        key: Some(key.unwrap_or("TEST-TC-1").to_string()),
        name: name.unwrap_or("Test Case 1").to_string(),
        project_key: project_key.unwrap_or("TEST").to_string(),
        issue_type: "Test".to_string(),
        status: Some("Approved".to_string()),
        priority: Some("High".to_string()),
        assignee: Some("test-user-123".to_string()),
        description: Some("Test case description".to_string()),
        labels: Some(vec!["test".to_string(), "example".to_string()]),
        components: Some(vec![]),
        fix_versions: Some(vec![]),
        custom_fields: Some(HashMap::new()),
    }
}

/// Helper function to create a Zephyr test step for testing
#[must_use]
pub fn create_test_zephyr_test_step(
    id: Option<&str>,
    step: Option<&str>,
    data: Option<&str>,
    result: Option<&str>,
) -> ZephyrTestStep {
    ZephyrTestStep {
        id: Some(id.unwrap_or("10001").to_string()),
        step: step
            .unwrap_or("Step 1: Navigate to the login page")
            .to_string(),
        data: data.map(ToString::to_string),
        result: result.map(ToString::to_string),
        order: 1,
        test_case_id: Some("10001".to_string()),
    }
}

/// Helper function to get current timestamp in Jira format
#[must_use]
pub fn current_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    // Convert to Jira timestamp format (ISO 8601)
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();
    let _millis = secs * 1000 + u64::from(nanos / 1_000_000);

    // This is a simplified version - in real code you'd use chrono
    "2024-01-01T10:00:00.000+0000".to_string()
}

/// Helper function to create a JSON response for testing
#[must_use]
pub fn create_json_response(data: &serde_json::Value) -> String {
    data.to_string()
}

/// Helper function to create an error response for testing
#[must_use]
pub fn create_error_response(_status: u16, message: &str) -> serde_json::Value {
    json!({
        "errorMessages": [message],
        "errors": {}
    })
}

/// Helper function to create a success response for testing
#[must_use]
pub fn create_success_response(data: &serde_json::Value) -> serde_json::Value {
    json!({
        "status": "success",
        "data": data
    })
}

/// Helper function to create a bulk operation response for testing
#[must_use]
pub fn create_bulk_operation_response(
    total: usize,
    successful: usize,
    failed: usize,
) -> serde_json::Value {
    json!({
        "results": (0..total).map(|i| {
            json!({
                "id": format!("{}", 12345 + i),
                "key": format!("TEST-{}", 123 + i),
                "status": if i < successful { "success" } else { "failed" }
            })
        }).collect::<Vec<_>>(),
        "summary": {
            "total": total,
            "successful": successful,
            "failed": failed
        }
    })
}

/// Helper function to assert that a result is an error with a specific message
/// # Panics
/// This function panics if the result is Ok or if the error message doesn't contain the expected text.
pub fn assert_error_message<T>(
    result: Result<T, Box<dyn std::error::Error>>,
    expected_message: &str,
) {
    match result {
        Ok(_) => panic!("Expected error but got Ok"),
        Err(e) => {
            let error_message = e.to_string();
            assert!(
                error_message.contains(expected_message),
                "Expected error message to contain '{expected_message}', but got '{error_message}'"
            );
        }
    }
}

/// Helper function to assert that a result is an error of a specific type
/// # Panics
/// This function panics if the result is Ok or if the error doesn't match the expected error.
pub fn assert_error_type<T, E: std::fmt::Debug + PartialEq>(
    result: Result<T, E>,
    expected_error: &E,
) {
    match result {
        Ok(_) => panic!("Expected error but got Ok"),
        Err(e) => assert_eq!(e, *expected_error),
    }
}

/// Helper function to create a test environment with temporary files
/// # Panics
/// This function panics if it fails to create a temporary directory.
#[must_use]
pub fn create_test_env() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

/// Helper function to create a test configuration file
/// # Panics
/// This function panics if it fails to serialize the config or write the file.
#[must_use]
pub fn create_test_config_file(
    temp_dir: &tempfile::TempDir,
    config: &JiraConfig,
) -> std::path::PathBuf {
    let config_path = temp_dir.path().join("test_config.toml");
    let config_content = toml::to_string(config).expect("Failed to serialize config");
    std::fs::write(&config_path, config_content).expect("Failed to write config file");
    config_path
}

/// Helper function to create a test log file
/// # Panics
/// This function panics if it fails to write the log file.
#[must_use]
pub fn create_test_log_file(temp_dir: &tempfile::TempDir) -> std::path::PathBuf {
    let log_path = temp_dir.path().join("test.log");
    std::fs::write(&log_path, "Test log content\n").expect("Failed to write log file");
    log_path
}
