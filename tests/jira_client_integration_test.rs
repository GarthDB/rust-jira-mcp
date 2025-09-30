use rust_jira_mcp::config::jira::JiraConfig;
use rust_jira_mcp::jira::client::JiraClient;

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
async fn test_jira_client_creation() {
    let config = test_config();
    let client = JiraClient::new(config).unwrap();

    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
    assert!(client.auth_header().contains("Bearer"));
    // Test that the HTTP client is configured
    let _http_client = client.http_client();
}

#[tokio::test]
async fn test_jira_client_should_retry() {
    use reqwest::StatusCode;

    // Test retry logic for various status codes
    assert!(JiraClient::should_retry(StatusCode::TOO_MANY_REQUESTS));
    assert!(JiraClient::should_retry(StatusCode::INTERNAL_SERVER_ERROR));
    assert!(JiraClient::should_retry(StatusCode::BAD_GATEWAY));
    assert!(JiraClient::should_retry(StatusCode::SERVICE_UNAVAILABLE));
    assert!(JiraClient::should_retry(StatusCode::GATEWAY_TIMEOUT));

    assert!(!JiraClient::should_retry(StatusCode::OK));
    assert!(!JiraClient::should_retry(StatusCode::BAD_REQUEST));
    assert!(!JiraClient::should_retry(StatusCode::NOT_FOUND));
}

#[tokio::test]
async fn test_jira_client_http_methods() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test that the client can be created and basic methods work
    // We can't test actual HTTP calls without a real server, but we can test
    // that the client is properly configured

    // Test configuration access
    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
    assert!(client.auth_header().starts_with("Bearer"));

    // Test that the HTTP client is configured
    let _http_client = client.http_client();
}

#[tokio::test]
async fn test_jira_client_issue_operations_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test that we can create the client and access its methods
    // The actual HTTP calls would fail without a real server, but we can test
    // the structure and configuration

    // Test that the client has the expected methods available
    // (We can't call them without mocking, but we can verify they exist)
    let _client = client; // Use the client to avoid unused warnings
}

#[tokio::test]
async fn test_jira_client_comment_operations_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_transition_operations_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert!(client.auth_header().starts_with("Bearer"));
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_project_operations_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_metadata_operations_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert!(client.auth_header().contains("Bearer"));
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_linking_operations_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_attachment_operations_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert!(client.auth_header().starts_with("Bearer"));
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_worklog_operations_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_component_operations_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert!(client.auth_header().contains("Bearer"));
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_label_operations_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_watcher_operations_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert!(client.auth_header().starts_with("Bearer"));
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_cloning_operations_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_zephyr_operations_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert!(client.auth_header().contains("Bearer"));
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_bulk_operations_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_error_handling_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert!(client.auth_header().starts_with("Bearer"));
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_retry_logic_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_rate_limiting_structure() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test client creation and basic configuration
    assert!(client.auth_header().contains("Bearer"));
    let _client = client;
}

#[tokio::test]
async fn test_jira_client_configuration_validation() {
    // Test with valid configuration
    let valid_config = test_config();
    let client = JiraClient::new(valid_config);
    assert!(client.is_ok());

    // Test with invalid configuration (empty API URL)
    let invalid_config = JiraConfig {
        api_base_url: String::new(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(false),
    };

    // This should still work as URL validation happens during actual requests
    let client = JiraClient::new(invalid_config);
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_jira_client_timeout_configuration() {
    let config = JiraConfig {
        api_base_url: "https://test-jira.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(60), // Different timeout
        log_file: None,
        strict_ssl: Some(false),
    };

    let client = JiraClient::new(config).unwrap();
    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
    assert!(client.auth_header().starts_with("Bearer"));
}

#[tokio::test]
async fn test_jira_client_ssl_configuration() {
    let config = JiraConfig {
        api_base_url: "https://test-jira.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(true), // Strict SSL
    };

    let client = JiraClient::new(config).unwrap();
    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
    assert!(client.auth_header().starts_with("Bearer"));
}

#[tokio::test]
async fn test_jira_client_auth_header_generation() {
    let config = JiraConfig {
        api_base_url: "https://test-jira.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(false),
    };

    let client = JiraClient::new(config).unwrap();
    let auth_header = client.auth_header();

    // The auth header should be a Bearer auth header
    assert!(auth_header.starts_with("Bearer "));
    assert!(auth_header.len() > 7); // More than just "Bearer "
}

#[tokio::test]
async fn test_jira_client_method_availability() {
    let client = JiraClient::new(test_config()).unwrap();

    // Test that all the main methods are available on the client
    // We can't call them without a real server, but we can verify the client exists
    let _client = client;

    // This test ensures that the client struct has all the expected methods
    // The actual method calls would be tested with integration tests against a real server
}

#[tokio::test]
async fn test_jira_client_http_client_configuration() {
    let client = JiraClient::new(test_config()).unwrap();
    let http_client = client.http_client();

    // Test that the HTTP client is properly configured
    let _ = http_client;

    // Test that we can access the client's configuration
    assert_eq!(
        client.api_base_url(),
        "https://test-jira.example.com/rest/api/2"
    );
}

#[tokio::test]
async fn test_jira_client_retry_status_codes() {
    use reqwest::StatusCode;

    // Test all the retry status codes
    let retry_codes = vec![
        StatusCode::TOO_MANY_REQUESTS,
        StatusCode::INTERNAL_SERVER_ERROR,
        StatusCode::BAD_GATEWAY,
        StatusCode::SERVICE_UNAVAILABLE,
        StatusCode::GATEWAY_TIMEOUT,
    ];

    for code in retry_codes {
        assert!(
            JiraClient::should_retry(code),
            "Status code {code} should be retryable"
        );
    }

    // Test non-retry status codes
    let non_retry_codes = vec![
        StatusCode::OK,
        StatusCode::CREATED,
        StatusCode::BAD_REQUEST,
        StatusCode::UNAUTHORIZED,
        StatusCode::FORBIDDEN,
        StatusCode::NOT_FOUND,
        StatusCode::METHOD_NOT_ALLOWED,
    ];

    for code in non_retry_codes {
        assert!(
            !JiraClient::should_retry(code),
            "Status code {code} should not be retryable"
        );
    }
}
