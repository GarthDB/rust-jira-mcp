use rust_jira_mcp::config::jira::JiraConfig;
use std::path::PathBuf;

#[test]
fn test_jira_config_default() {
    let config = JiraConfig::default();

    assert_eq!(
        config.api_base_url,
        "https://jira.corp.adobe.com/rest/api/2"
    );
    assert!(config.email.is_empty());
    assert!(config.personal_access_token.is_empty());
    assert!(config.default_project.is_none());
    assert_eq!(config.max_results, Some(50));
    assert_eq!(config.timeout_seconds, Some(30));
    assert!(config.log_file.is_none());
    assert_eq!(config.strict_ssl, Some(true));
}

#[test]
fn test_jira_config_serialization() {
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(100),
        timeout_seconds: Some(60),
        log_file: Some(PathBuf::from("/tmp/test.log")),
        strict_ssl: Some(false),
    };

    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: JiraConfig = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.api_base_url, config.api_base_url);
    assert_eq!(deserialized.email, config.email);
    assert_eq!(
        deserialized.personal_access_token,
        config.personal_access_token
    );
    assert_eq!(deserialized.default_project, config.default_project);
    assert_eq!(deserialized.max_results, config.max_results);
    assert_eq!(deserialized.timeout_seconds, config.timeout_seconds);
    assert_eq!(deserialized.log_file, config.log_file);
    assert_eq!(deserialized.strict_ssl, config.strict_ssl);
}

#[test]
fn test_jira_config_clone() {
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(100),
        timeout_seconds: Some(60),
        log_file: Some(PathBuf::from("/tmp/test.log")),
        strict_ssl: Some(false),
    };

    let cloned = config.clone();

    assert_eq!(cloned.api_base_url, config.api_base_url);
    assert_eq!(cloned.email, config.email);
    assert_eq!(cloned.personal_access_token, config.personal_access_token);
    assert_eq!(cloned.default_project, config.default_project);
    assert_eq!(cloned.max_results, config.max_results);
    assert_eq!(cloned.timeout_seconds, config.timeout_seconds);
    assert_eq!(cloned.log_file, config.log_file);
    assert_eq!(cloned.strict_ssl, config.strict_ssl);
}

#[test]
fn test_jira_config_debug() {
    let config = JiraConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("JiraConfig"));
    assert!(debug_str.contains("api_base_url"));
    assert!(debug_str.contains("email"));
    assert!(debug_str.contains("personal_access_token"));
}

#[test]
fn test_auth_header_bearer_token() {
    // Test Adobe Jira format (Bearer token)
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "YOUR_ADOBE_JIRA_TOKEN_HERE".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let auth_header = config.auth_header();
        assert_eq!(auth_header, "Bearer YOUR_ADOBE_JIRA_TOKEN_HERE");
}

#[test]
fn test_auth_header_basic_token() {
    // Test standard Jira format (Basic auth)
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token-123".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let auth_header = config.auth_header();
    assert_eq!(auth_header, "Basic dGVzdEBleGFtcGxlLmNvbTp0ZXN0LXRva2VuLTEyMw==");
}

#[test]
fn test_auth_header_with_colon_token() {
    // Test token with colon (should use Basic auth)
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "user:password".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let auth_header = config.auth_header();
    assert_eq!(auth_header, "Basic dGVzdEBleGFtcGxlLmNvbTp1c2VyOnBhc3N3b3Jk");
}

#[test]
fn test_auth_header_with_empty_token() {
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: String::new(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let auth_header = config.auth_header();
    assert_eq!(auth_header, "Basic dGVzdEBleGFtcGxlLmNvbTo=");
}

#[test]
fn test_auth_header_short_token() {
    // Test short token (should use Basic auth)
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "short".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let auth_header = config.auth_header();
    assert_eq!(auth_header, "Basic dGVzdEBleGFtcGxlLmNvbTpzaG9ydA==");
}

#[test]
fn test_timeout_duration_default() {
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let duration = config.timeout_duration();
    assert_eq!(duration, std::time::Duration::from_secs(30));
}

#[test]
fn test_timeout_duration_custom() {
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: Some(120),
        log_file: None,
        strict_ssl: None,
    };

    let duration = config.timeout_duration();
    assert_eq!(duration, std::time::Duration::from_secs(120));
}

#[test]
fn test_validate_success() {
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token-1234567890".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_invalid_email() {
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "invalid-email".to_string(),
        personal_access_token: "test-token-1234567890".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("email"));
}

#[test]
fn test_validate_empty_email() {
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: String::new(),
        personal_access_token: "test-token-1234567890".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("email"));
}

#[test]
fn test_validate_short_token() {
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "short".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("personal_access_token"));
}

#[test]
fn test_validate_invalid_url() {
    let config = JiraConfig {
        api_base_url: "invalid-url".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token-1234567890".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("api_base_url"));
}

#[test]
fn test_validate_http_url() {
    let config = JiraConfig {
        api_base_url: "http://test.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token-1234567890".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_https_url() {
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token-1234567890".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_email_with_at_start() {
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "@example.com".to_string(),
        personal_access_token: "test-token-1234567890".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_email_with_at_end() {
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@".to_string(),
        personal_access_token: "test-token-1234567890".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_email_without_dot() {
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test@example".to_string(),
        personal_access_token: "test-token-1234567890".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_email_without_at() {
    let config = JiraConfig {
        api_base_url: "https://test.example.com/rest/api/2".to_string(),
        email: "test.example.com".to_string(),
        personal_access_token: "test-token-1234567890".to_string(),
        default_project: None,
        max_results: None,
        timeout_seconds: None,
        log_file: None,
        strict_ssl: None,
    };

    let result = config.validate();
    assert!(result.is_err());
}
