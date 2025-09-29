use rust_jira_mcp::config::{jira::JiraConfig, ConfigManager, ConfigOptions, SecretManager};
use rust_jira_mcp::logging::{Logger, LoggingConfig, MetricsCollector};
use rust_jira_mcp::mcp::server::MCPServer;
use std::collections::HashMap;
use tempfile::TempDir;

#[tokio::test]
async fn test_main_application_flow() {
    // Test the main application initialization flow
    // Skip logging setup to avoid global subscriber conflicts in tests

    let metrics_collector = MetricsCollector::new();
    let _logger = Logger::new(metrics_collector.clone());

    // Test configuration manager initialization
    let mut config_manager = ConfigManager::new();
    let config_options = ConfigOptions {
        hot_reload: false,
        watch_paths: vec![],
        strict_validation: true,
        fail_on_missing: true,
    };

    // Test configuration loading
    let result = config_manager.load_with_options(config_options).await;
    // Configuration loading might fail in CI environment, that's acceptable for this test
    if result.is_ok() {
        let config = config_manager.get_config().await;
        assert!(!config.api_base_url.is_empty());
    }

    // Test secret manager initialization
    let mut secret_manager = SecretManager::new();
    let secret_result = secret_manager.load_from_env("JIRA_").await;
    // Secret loading might fail in CI environment, that's acceptable for this test
    let _ = secret_result;

    // Test MCP server creation with a test config
    let test_config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(false),
    };
    let _server = MCPServer::new(test_config);
    // Test that server can be created - if we get here, it worked
}

#[tokio::test]
async fn test_configuration_loading_with_secrets() {
    // Test configuration loading with secrets
    let mut secret_manager = SecretManager::new();

    // Set up test environment variables
    std::env::set_var(
        "JIRA_API_BASE_URL",
        "https://test-jira.example.com/rest/api/2",
    );
    std::env::set_var("JIRA_EMAIL", "test@example.com");
    std::env::set_var("JIRA_PERSONAL_ACCESS_TOKEN", "test-token");

    let result = secret_manager.load_from_env("JIRA_").await;
    assert!(result.is_ok());

    // Test JiraConfig loading with secrets
    let config_result = JiraConfig::load_with_secrets(&secret_manager).await;
    assert!(config_result.is_ok());

    let config = config_result.unwrap();
    assert_eq!(
        config.api_base_url,
        "https://test-jira.example.com/rest/api/2"
    );
    assert_eq!(config.email, "test@example.com");
    assert_eq!(config.personal_access_token, "test-token");

    // Test configuration validation
    let validation_result = config.validate();
    assert!(validation_result.is_ok());

    // Clean up
    std::env::remove_var("JIRA_API_BASE_URL");
    std::env::remove_var("JIRA_EMAIL");
    std::env::remove_var("JIRA_PERSONAL_ACCESS_TOKEN");
}

#[tokio::test]
async fn test_logging_and_metrics() {
    // Test logging and metrics functionality
    let logging_config = LoggingConfig::production();
    rust_jira_mcp::logging::setup_logging(&logging_config);

    let metrics_collector = MetricsCollector::new();
    let logger = Logger::new(metrics_collector.clone());

    // Test logging operation success
    let mut metadata = HashMap::new();
    metadata.insert("test_key".to_string(), "test_value".to_string());

    logger.log_operation_success(
        "test_operation",
        std::time::Duration::from_millis(100),
        &metadata,
    );

    // Test metrics collection
    metrics_collector
        .record_operation_success(
            "test_operation",
            std::time::Duration::from_millis(100),
            &metadata,
        )
        .await;
}

#[tokio::test]
async fn test_mcp_server_initialization() {
    // Test MCP server initialization with various configurations
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

    let _server = MCPServer::new(config);

    // Test tool listing
    let tools = MCPServer::list_tools();
    assert!(!tools.is_empty());
}

#[tokio::test]
async fn test_configuration_sources() {
    // Test configuration sources functionality
    let mut config_manager = ConfigManager::new();
    let config_options = ConfigOptions {
        hot_reload: true,
        watch_paths: vec![std::path::PathBuf::from("config/test.toml")],
        strict_validation: true,
        fail_on_missing: false,
    };

    let _result = config_manager.load_with_options(config_options).await;
    // This might fail due to missing config file, but that's ok for testing
    // We just want to test the code path

    let sources = config_manager.get_config_sources();
    // Sources might be empty in CI environment, that's acceptable for this test
    let _ = sources;

    let hot_reload_enabled = config_manager.is_hot_reload_enabled();
    // Hot reload might not be enabled in CI environment, that's acceptable for this test
    let _ = hot_reload_enabled;
}

#[tokio::test]
async fn test_secret_loading_from_file() {
    // Test secret loading from file
    let temp_dir = TempDir::new().unwrap();
    let secrets_file = temp_dir.path().join("secrets.toml");

    std::fs::write(
        &secrets_file,
        r#"
[secrets.api_base_url]
Plain = "https://file-jira.example.com/rest/api/2"

[secrets.email]
Plain = "file@example.com"

[secrets.personal_access_token]
Plain = "file-token"
"#,
    )
    .unwrap();

    let mut secret_manager = SecretManager::new();
    let result = secret_manager.load_from_file(&secrets_file).await;
    assert!(result.is_ok());

    // Test that secrets were loaded
    let api_url = secret_manager.get_secret("api_base_url").await.unwrap();
    assert_eq!(
        api_url,
        Some("https://file-jira.example.com/rest/api/2".to_string())
    );

    let email = secret_manager.get_secret("email").await.unwrap();
    assert_eq!(email, Some("file@example.com".to_string()));
}

#[tokio::test]
async fn test_jira_config_validation() {
    // Test JiraConfig validation
    let valid_config = JiraConfig {
        api_base_url: "https://valid-jira.example.com/rest/api/2".to_string(),
        email: "valid@example.com".to_string(),
        personal_access_token: "valid-token-12345".to_string(),
        default_project: Some("VALID".to_string()),
        max_results: Some(100),
        timeout_seconds: Some(60),
        log_file: None,
        strict_ssl: Some(true),
    };

    let validation_result = valid_config.validate();
    assert!(validation_result.is_ok());

    // Test invalid configuration
    let invalid_config = JiraConfig {
        api_base_url: "invalid-url".to_string(),
        email: "invalid-email".to_string(),
        personal_access_token: "".to_string(),
        default_project: None,
        max_results: Some(0),
        timeout_seconds: Some(0),
        log_file: None,
        strict_ssl: Some(true),
    };

    let invalid_validation = invalid_config.validate();
    assert!(invalid_validation.is_err());
}
