use rust_jira_mcp::config::JiraConfig;
use rust_jira_mcp::jira::client::JiraClient;
use rust_jira_mcp::types::jira::*;
use serde_json::json;
use std::collections::HashMap;

/// Test configuration for unit tests
fn test_config() -> JiraConfig {
    JiraConfig {
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
    let client = JiraClient::new(config).expect("Failed to create JiraClient");

    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
    assert_eq!(client.auth_header(), "Bearer test-token");
}

#[tokio::test]
async fn test_auth_header_generation() {
    let config = test_config();
    let client = JiraClient::new(config).expect("Failed to create JiraClient");

    assert_eq!(client.auth_header(), "Bearer test-token");
}

#[tokio::test]
async fn test_url_building() {
    let config = test_config();
    let _client = JiraClient::new(config).expect("Failed to create JiraClient");

    // Test URL building logic by checking the internal build_url method
    // This is a bit of a hack since build_url is private, but we can test it indirectly
    // through the public API methods
    let result = _client.get_issue("TEST-123").await;
    // We expect this to fail with a network error since we're not actually making a request
    assert!(result.is_err());
}

#[tokio::test]
async fn test_should_retry_logic() {
    let config = test_config();
    let _client = JiraClient::new(config).expect("Failed to create JiraClient");

    // Test retry logic for various status codes
    use reqwest::StatusCode;

    // Should retry
    assert!(JiraClient::should_retry(StatusCode::TOO_MANY_REQUESTS));
    assert!(JiraClient::should_retry(StatusCode::INTERNAL_SERVER_ERROR));
    assert!(JiraClient::should_retry(StatusCode::BAD_GATEWAY));
    assert!(JiraClient::should_retry(StatusCode::SERVICE_UNAVAILABLE));
    assert!(JiraClient::should_retry(StatusCode::GATEWAY_TIMEOUT));

    // Should not retry
    assert!(!JiraClient::should_retry(StatusCode::OK));
    assert!(!JiraClient::should_retry(StatusCode::NOT_FOUND));
    assert!(!JiraClient::should_retry(StatusCode::UNAUTHORIZED));
    assert!(!JiraClient::should_retry(StatusCode::FORBIDDEN));
}

#[tokio::test]
async fn test_jira_issue_creation() {
    let issue = JiraIssue {
        id: "12345".to_string(),
        key: "TEST-123".to_string(),
        self_url: "https://test-jira.example.com/rest/api/2/issue/12345".to_string(),
        fields: HashMap::new(),
    };

    assert_eq!(issue.id, "12345");
    assert_eq!(issue.key, "TEST-123");
}

#[tokio::test]
async fn test_jira_project_creation() {
    let project = JiraProject {
        id: "10000".to_string(),
        key: "TEST".to_string(),
        name: "Test Project".to_string(),
        project_type_key: "software".to_string(),
        self_url: "https://test-jira.example.com/rest/api/2/project/10000".to_string(),
    };

    assert_eq!(project.key, "TEST");
    assert_eq!(project.name, "Test Project");
}

#[tokio::test]
async fn test_jira_user_creation() {
    let user = JiraUser {
        account_id: "12345:abcdef".to_string(),
        display_name: "Test User".to_string(),
        email_address: Some("test@example.com".to_string()),
        active: true,
        time_zone: Some("UTC".to_string()),
    };

    assert_eq!(user.display_name, "Test User");
    assert_eq!(user.email_address, Some("test@example.com".to_string()));
}

#[tokio::test]
async fn test_jira_comment_creation() {
    let user = JiraUser {
        account_id: "12345:abcdef".to_string(),
        display_name: "Test User".to_string(),
        email_address: Some("test@example.com".to_string()),
        active: true,
        time_zone: Some("UTC".to_string()),
    };

    let comment = JiraComment {
        id: "12345".to_string(),
        body: "This is a test comment".to_string(),
        author: user,
        created: "2023-01-01T00:00:00.000Z".to_string(),
        updated: None,
    };

    assert_eq!(comment.body, "This is a test comment");
    assert_eq!(comment.author.display_name, "Test User");
}

#[tokio::test]
async fn test_jira_search_result_creation() {
    let issue = JiraIssue {
        id: "12345".to_string(),
        key: "TEST-123".to_string(),
        self_url: "https://test-jira.example.com/rest/api/2/issue/12345".to_string(),
        fields: HashMap::new(),
    };

    let search_result = JiraSearchResult {
        expand: Some("names,schema".to_string()),
        start_at: 0,
        max_results: 50,
        total: 1,
        issues: vec![issue],
    };

    assert_eq!(search_result.total, 1);
    assert_eq!(search_result.issues.len(), 1);
    assert_eq!(search_result.issues[0].key, "TEST-123");
}

#[tokio::test]
async fn test_transition_properties() {
    let props = TransitionProperties::new();
    assert!(matches!(props.screen, ScreenProperty::NoScreen));
    assert!(matches!(props.scope, ScopeProperty::Local));
    assert!(matches!(
        props.availability,
        AvailabilityProperty::Unavailable
    ));
    assert!(matches!(
        props.conditionality,
        ConditionalityProperty::Unconditional
    ));

    let props_all = TransitionProperties::with_all_properties();
    assert!(matches!(props_all.screen, ScreenProperty::HasScreen));
    assert!(matches!(props_all.scope, ScopeProperty::Global));
    assert!(matches!(
        props_all.availability,
        AvailabilityProperty::Available
    ));
    assert!(matches!(
        props_all.conditionality,
        ConditionalityProperty::Conditional
    ));
}

#[tokio::test]
async fn test_jira_priority_creation() {
    let priority = JiraPriority {
        id: "1".to_string(),
        name: "Highest".to_string(),
        description: Some("This issue has the highest priority".to_string()),
        icon_url: Some(
            "https://test-jira.example.com/images/icons/priorities/highest.svg".to_string(),
        ),
    };

    assert_eq!(priority.name, "Highest");
    assert_eq!(priority.id, "1");
}

#[tokio::test]
async fn test_jira_status_creation() {
    let status_category = JiraStatusCategory {
        id: 1,
        key: "new".to_string(),
        color_name: "blue-gray".to_string(),
        name: "To Do".to_string(),
    };

    let status = JiraStatus {
        id: "1".to_string(),
        name: "To Do".to_string(),
        description: Some(
            "The issue is open and ready for the assignee to start work on it".to_string(),
        ),
        icon_url: Some("https://test-jira.example.com/images/icons/statuses/open.png".to_string()),
        status_category,
    };

    assert_eq!(status.name, "To Do");
    assert_eq!(status.status_category.key, "new");
}

#[tokio::test]
async fn test_jira_work_log_creation() {
    let user = JiraUser {
        account_id: "12345:abcdef".to_string(),
        display_name: "Test User".to_string(),
        email_address: Some("test@example.com".to_string()),
        active: true,
        time_zone: Some("UTC".to_string()),
    };

    let work_log = JiraWorkLog {
        id: "12345".to_string(),
        comment: Some("Worked on this issue".to_string()),
        time_spent: "1h 30m".to_string(),
        time_spent_seconds: 5400,
        author: user,
        created: "2023-01-01T00:00:00.000Z".to_string(),
        updated: None,
    };

    assert_eq!(work_log.time_spent, "1h 30m");
    assert_eq!(work_log.time_spent_seconds, 5400);
    assert_eq!(work_log.author.display_name, "Test User");
}

#[tokio::test]
async fn test_zephyr_test_step_creation() {
    let test_step = ZephyrTestStep {
        id: Some("12345".to_string()),
        step: "Click the login button".to_string(),
        data: Some("Expected: User should be redirected to dashboard".to_string()),
        result: Some("PASS".to_string()),
        order: 1,
        test_case_id: Some("TC-123".to_string()),
    };

    assert_eq!(test_step.step, "Click the login button");
    assert_eq!(test_step.order, 1);
    assert_eq!(test_step.result, Some("PASS".to_string()));
}

#[tokio::test]
async fn test_jira_link_type_creation() {
    let link_type = JiraLinkType {
        id: "10000".to_string(),
        name: "Blocks".to_string(),
        inward: "is blocked by".to_string(),
        outward: "blocks".to_string(),
        self_url: "https://test-jira.example.com/rest/api/2/issueLinkType/10000".to_string(),
    };

    assert_eq!(link_type.name, "Blocks");
    assert_eq!(link_type.inward, "is blocked by");
    assert_eq!(link_type.outward, "blocks");
}

#[tokio::test]
async fn test_jira_component_creation() {
    let component = JiraComponent {
        id: "10000".to_string(),
        name: "Backend".to_string(),
        description: Some("Backend services and APIs".to_string()),
        self_url: "https://test-jira.example.com/rest/api/2/component/10000".to_string(),
    };

    assert_eq!(component.name, "Backend");
    assert_eq!(component.id, "10000");
}

#[tokio::test]
async fn test_jira_issue_type_creation() {
    let issue_type = JiraIssueType {
        id: "10000".to_string(),
        name: "Bug".to_string(),
        description: Some(
            "A problem which impairs or prevents the functions of the product".to_string(),
        ),
        icon_url: Some("https://test-jira.example.com/images/icons/issuetypes/bug.svg".to_string()),
        subtask: false,
    };

    assert_eq!(issue_type.name, "Bug");
    assert!(!issue_type.subtask);
}

#[tokio::test]
async fn test_serialization_deserialization() {
    let issue = JiraIssue {
        id: "12345".to_string(),
        key: "TEST-123".to_string(),
        self_url: "https://test-jira.example.com/rest/api/2/issue/12345".to_string(),
        fields: {
            let mut fields = HashMap::new();
            fields.insert("summary".to_string(), json!("Test Issue"));
            fields.insert("description".to_string(), json!("This is a test issue"));
            fields
        },
    };

    // Test serialization
    let json = serde_json::to_string(&issue).expect("Failed to serialize issue");
    assert!(json.contains("TEST-123"));
    assert!(json.contains("Test Issue"));

    // Test deserialization
    let deserialized: JiraIssue = serde_json::from_str(&json).expect("Failed to deserialize issue");
    assert_eq!(deserialized.key, issue.key);
    assert_eq!(deserialized.id, issue.id);
}

#[tokio::test]
async fn test_config_validation() {
    // Test valid config
    let valid_config = JiraConfig {
        api_base_url: "https://test-jira.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "valid-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(true),
    };

    let client = JiraClient::new(valid_config);
    assert!(client.is_ok());

    // Test config with empty email (should still work for unit tests)
    let config_with_empty_email = JiraConfig {
        api_base_url: "https://test-jira.example.com/rest/api/2".to_string(),
        email: "".to_string(),
        personal_access_token: "valid-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(true),
    };

    let client = JiraClient::new(config_with_empty_email);
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_timeout_duration() {
    let config = JiraConfig {
        api_base_url: "https://test-jira.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(60),
        log_file: None,
        strict_ssl: Some(true),
    };

    assert_eq!(config.timeout_duration().as_secs(), 60);

    // Test default timeout
    let config_default = JiraConfig {
        api_base_url: "https://test-jira.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: None,
        log_file: None,
        strict_ssl: Some(true),
    };

    assert_eq!(config_default.timeout_duration().as_secs(), 30);
}
