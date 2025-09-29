use anyhow::Result;
use rust_jira_mcp::config::{
    ConfigManager, ConfigOptions, JiraConfig, SecretManager,
};
use rust_jira_mcp::config::validation::{ConfigValidator, ValidationRule};
use rust_jira_mcp::config::manager::ConfigSource;

#[tokio::test]
async fn test_config_manager_basic_loading() -> Result<()> {
    // Set up environment variables for the test
    std::env::set_var("JIRA_API_BASE_URL", "https://test.atlassian.net/rest/api/2");
    std::env::set_var("JIRA_EMAIL", "test@example.com");
    std::env::set_var("JIRA_PERSONAL_ACCESS_TOKEN", "test_token_12345");
    
    let mut config_manager = ConfigManager::new();
    config_manager.load_with_options(ConfigOptions::default()).await?;
    
    // Test that we can get the config
    let _config = config_manager.get_config().await;
    
    // Test that we can get config sources
    let sources = config_manager.get_config_sources();
    assert!(!sources.is_empty());
    
    // Should always have at least the default source
    assert!(sources
        .iter()
        .any(|s| matches!(s, ConfigSource::Default)));

    Ok(())
}

#[tokio::test]
async fn test_secret_manager_basic() -> Result<()> {
    let mut secret_manager = SecretManager::new();
    
    // Test loading from environment (should not fail even if no env vars)
    secret_manager.load_from_env("JIRA_").await?;
    
    // Test that we can get a secret (should return None for non-existent)
    let result = secret_manager.get_secret("nonexistent").await?;
    assert!(result.is_none());

    Ok(())
}

#[tokio::test]
async fn test_jira_config_validation() -> Result<()> {
    // Test valid configuration
    let valid_config = JiraConfig {
        email: "test@example.com".to_string(),
        personal_access_token: "valid_token_12345".to_string(),
        api_base_url: "https://jira.example.com/rest/api/2".to_string(),
        ..Default::default()
    };

    assert!(valid_config.validate().is_ok());

    // Test invalid email
    let mut invalid_config = valid_config.clone();
    invalid_config.email = "invalid-email".to_string();
    assert!(invalid_config.validate().is_err());

    // Test short token
    let mut short_token_config = valid_config.clone();
    short_token_config.personal_access_token = "short".to_string();
    assert!(short_token_config.validate().is_err());

    Ok(())
}

#[tokio::test]
async fn test_config_validation_system() -> Result<()> {
    let mut validator = ConfigValidator::new();
    
    // Add validation rules
    validator = validator.add_rule(
        ValidationRule::new("email".to_string())
            .required()
            .min_length(5)
    );
    
    // Test valid email
    assert!(validator.validate("email", "test@example.com").is_ok());
    
    // Test invalid email (too short)
    assert!(validator.validate("email", "a@b").is_err());
    
    // Test missing required field
    assert!(validator.validate("email", "").is_err());

    Ok(())
}

#[tokio::test]
async fn test_config_manager_with_options() -> Result<()> {
    // Set up environment variables for the test
    std::env::set_var("JIRA_API_BASE_URL", "https://test.atlassian.net/rest/api/2");
    std::env::set_var("JIRA_EMAIL", "test@example.com");
    std::env::set_var("JIRA_PERSONAL_ACCESS_TOKEN", "test_token_12345");
    
    let mut config_manager = ConfigManager::new();
    
    let options = ConfigOptions {
        hot_reload: false,
        watch_paths: vec![],
        fail_on_missing: false,
        strict_validation: false,
    };
    
    config_manager.load_with_options(options).await?;
    
    let _config = config_manager.get_config().await;
    let sources = config_manager.get_config_sources();
    assert!(!sources.is_empty());

    Ok(())
}