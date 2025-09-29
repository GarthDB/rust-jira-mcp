//! Test data fixtures and sample data for testing

use crate::config::JiraConfig;
use crate::types::jira::*;
use serde_json::json;
use std::collections::HashMap;

/// Create a standard test configuration for unit tests
pub fn test_jira_config() -> JiraConfig {
    JiraConfig {
        api_base_url: "https://test-jira.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token-12345".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(true),
    }
}

/// Create a test configuration for integration tests (with mock server)
pub fn mock_jira_config(base_url: &str) -> JiraConfig {
    JiraConfig {
        api_base_url: base_url.to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token-12345".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(false), // Disable SSL verification for tests
    }
}

/// Create a sample Jira issue for testing
pub fn sample_jira_issue() -> JiraIssue {
    let mut fields = HashMap::new();
    fields.insert("summary".to_string(), serde_json::Value::String("Test Issue Summary".to_string()));
    fields.insert("description".to_string(), serde_json::Value::String("This is a test issue description".to_string()));
    
    JiraIssue {
        id: "12345".to_string(),
        key: "TEST-123".to_string(),
        self_url: "https://test-jira.example.com/rest/api/2/issue/12345".to_string(),
        fields,
    }
}

/// Create a sample Jira comment for testing
pub fn sample_jira_comment() -> JiraComment {
    JiraComment {
        id: "10001".to_string(),
        body: "This is a test comment".to_string(),
        author: JiraUser {
            account_id: "test-user-123".to_string(),
            display_name: "Test User".to_string(),
            email_address: Some("test.user@example.com".to_string()),
            active: true,
            time_zone: Some("UTC".to_string()),
        },
        created: "2024-01-01T10:00:00.000+0000".to_string(),
        updated: Some("2024-01-01T10:00:00.000+0000".to_string()),
    }
}

/// Create a sample Jira work log for testing
pub fn sample_jira_work_log() -> JiraWorkLog {
    JiraWorkLog {
        id: "10001".to_string(),
        comment: Some("Worked on this issue".to_string()),
        created: "2024-01-01T10:00:00.000+0000".to_string(),
        updated: Some("2024-01-01T10:00:00.000+0000".to_string()),
        time_spent: "1h 30m".to_string(),
        time_spent_seconds: 5400,
        author: JiraUser {
            account_id: "test-user-123".to_string(),
            display_name: "Test User".to_string(),
            email_address: Some("test.user@example.com".to_string()),
            active: true,
            time_zone: Some("UTC".to_string()),
        },
    }
}

/// Create a sample Zephyr test case for testing
pub fn sample_zephyr_test_case() -> ZephyrTestCase {
    ZephyrTestCase {
        id: Some("10001".to_string()),
        key: Some("TEST-TC-1".to_string()),
        name: "Test Case 1".to_string(),
        project_key: "TEST".to_string(),
        issue_type: "Test".to_string(),
        status: Some("Approved".to_string()),
        priority: Some("High".to_string()),
        assignee: Some("test-user-123".to_string()),
        description: Some("This is a test case description".to_string()),
        labels: Some(vec!["test".to_string(), "example".to_string()]),
        components: Some(vec![]),
        fix_versions: Some(vec![]),
        custom_fields: Some(HashMap::new()),
    }
}

/// Create a sample Zephyr test step for testing
pub fn sample_zephyr_test_step() -> ZephyrTestStep {
    ZephyrTestStep {
        id: Some("10001".to_string()),
        step: "Step 1: Navigate to the login page".to_string(),
        data: Some("https://example.com/login".to_string()),
        result: Some("User should see the login form".to_string()),
        order: 1,
        test_case_id: Some("10001".to_string()),
    }
}

/// Create sample JSON responses for mocking API calls
pub fn sample_issue_response() -> serde_json::Value {
    json!({
        "id": "12345",
        "key": "TEST-123",
        "self": "https://test-jira.example.com/rest/api/2/issue/12345",
        "fields": {
            "summary": "Test Issue Summary",
            "description": "This is a test issue description",
            "status": {
                "id": "1",
                "name": "To Do",
                "description": "Issue is in To Do status",
                "statusCategory": {
                    "id": 2,
                    "key": "new",
                    "colorName": "blue-gray",
                    "name": "To Do"
                }
            },
            "priority": {
                "id": "3",
                "name": "Medium",
                "iconUrl": "https://test-jira.example.com/images/icons/priorities/medium.svg"
            },
            "issuetype": {
                "id": "10001",
                "name": "Story",
                "description": "A user story",
                "iconUrl": "https://test-jira.example.com/images/icons/issuetypes/story.svg",
                "subtask": false
            },
            "project": {
                "id": "10000",
                "key": "TEST",
                "name": "Test Project",
                "projectTypeKey": "software",
                "avatarUrls": {}
            },
            "assignee": {
                "accountId": "test-user-123",
                "displayName": "Test User",
                "emailAddress": "test.user@example.com",
                "avatarUrls": {}
            },
            "reporter": {
                "accountId": "test-reporter-123",
                "displayName": "Test Reporter",
                "emailAddress": "test.reporter@example.com",
                "avatarUrls": {}
            },
            "created": "2024-01-01T10:00:00.000+0000",
            "updated": "2024-01-01T10:00:00.000+0000",
            "labels": ["test", "example"],
            "components": [],
            "fixVersions": []
        }
    })
}

/// Create sample search results response
pub fn sample_search_response() -> serde_json::Value {
    json!({
        "expand": "names,schema",
        "startAt": 0,
        "maxResults": 50,
        "total": 1,
        "issues": [sample_issue_response()]
    })
}

/// Create sample error response
/// Create a sample Jira issue
pub fn sample_issue() -> JiraIssue {
    JiraIssue {
        id: "12345".to_string(),
        key: "TEST-123".to_string(),
        self_url: "https://test-jira.example.com/rest/api/2/issue/12345".to_string(),
        fields: {
            let mut fields = HashMap::new();
            fields.insert("summary".to_string(), json!("Test Issue Summary"));
            fields.insert("description".to_string(), json!("This is a test issue description"));
            fields
        },
    }
}

pub fn sample_error_response(_status: u16, message: &str) -> serde_json::Value {
    json!({
        "errorMessages": [message],
        "errors": {}
    })
}

/// Create sample bulk operation response
pub fn sample_bulk_operation_response() -> serde_json::Value {
    json!({
        "results": [
            {
                "id": "12345",
                "key": "TEST-123",
                "status": "success"
            },
            {
                "id": "12346", 
                "key": "TEST-124",
                "status": "success"
            }
        ],
        "summary": {
            "total": 2,
            "successful": 2,
            "failed": 0
        }
    })
}
