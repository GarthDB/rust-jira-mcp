use base64::Engine;
use rust_jira_mcp::config::secrets::*;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tokio::fs;

#[test]
fn test_secret_manager_new() {
    let _manager = SecretManager::new();
    // Test that manager can be created successfully
    // This is acceptable for this test
}

#[test]
fn test_secret_manager_default() {
    let _manager = SecretManager::default();
    // Test that manager can be created successfully
    // This is acceptable for this test
}

#[test]
fn test_secret_manager_clone() {
    let manager = SecretManager::new();
    let _cloned = manager.clone();
    // Test that manager can be cloned successfully
    // This is acceptable for this test
}

#[test]
fn test_secret_manager_debug() {
    let manager = SecretManager::new();
    let debug_str = format!("{:?}", manager);
    assert!(debug_str.contains("SecretManager"));
}

#[test]
fn test_secret_value_plain() {
    let secret = SecretValue::Plain("test-secret".to_string());
    assert!(matches!(secret, SecretValue::Plain(_)));
}

#[test]
fn test_secret_value_base64() {
    let secret = SecretValue::Base64("dGVzdC1zZWNyZXQ=".to_string());
    assert!(matches!(secret, SecretValue::Base64(_)));
}

#[test]
fn test_secret_value_env_var() {
    let secret = SecretValue::EnvVar("TEST_VAR".to_string());
    assert!(matches!(secret, SecretValue::EnvVar(_)));
}

#[test]
fn test_secret_value_file_path() {
    let path = PathBuf::from("/tmp/secret.txt");
    let secret = SecretValue::FilePath(path.clone());
    assert!(matches!(secret, SecretValue::FilePath(_)));
}

#[test]
fn test_secret_value_encrypted() {
    let secret = SecretValue::Encrypted("encrypted-data".to_string());
    assert!(matches!(secret, SecretValue::Encrypted(_)));
}

#[test]
fn test_secret_value_serialization() {
    let secret = SecretValue::Plain("test-secret".to_string());
    let serialized = serde_json::to_string(&secret).unwrap();
    let deserialized: SecretValue = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        SecretValue::Plain(value) => assert_eq!(value, "test-secret"),
        _ => panic!("Expected Plain variant"),
    }
}

#[test]
fn test_secret_value_clone() {
    let secret = SecretValue::Plain("test-secret".to_string());
    let cloned = secret.clone();

    match (secret, cloned) {
        (SecretValue::Plain(orig), SecretValue::Plain(clone)) => {
            assert_eq!(orig, clone);
        }
        _ => panic!("Expected Plain variants"),
    }
}

#[test]
fn test_secret_value_debug() {
    let secret = SecretValue::Plain("test-secret".to_string());
    let debug_str = format!("{:?}", secret);
    assert!(debug_str.contains("Plain"));
    assert!(debug_str.contains("test-secret"));
}

#[tokio::test]
async fn test_secret_value_plain_resolve() {
    let secret = SecretValue::Plain("test-secret".to_string());
    let result = secret.resolve().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test-secret");
}

#[tokio::test]
async fn test_secret_value_base64_resolve() {
    let encoded = base64::engine::general_purpose::STANDARD.encode("test-secret");
    let secret = SecretValue::Base64(encoded);
    let result = secret.resolve().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test-secret");
}

#[tokio::test]
async fn test_secret_value_base64_invalid_resolve() {
    let secret = SecretValue::Base64("invalid-base64!".to_string());
    let result = secret.resolve().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_secret_value_env_var_resolve() {
    std::env::set_var("TEST_SECRET_VAR", "test-secret-value");
    let secret = SecretValue::EnvVar("TEST_SECRET_VAR".to_string());
    let result = secret.resolve().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test-secret-value");
    std::env::remove_var("TEST_SECRET_VAR");
}

#[tokio::test]
async fn test_secret_value_env_var_not_found() {
    let secret = SecretValue::EnvVar("NONEXISTENT_VAR".to_string());
    let result = secret.resolve().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_secret_value_file_path_resolve() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();
    fs::write(path, "test-secret-content").await.unwrap();

    let secret = SecretValue::FilePath(path.to_path_buf());
    let result = secret.resolve().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test-secret-content");
}

#[tokio::test]
async fn test_secret_value_file_path_not_found() {
    let secret = SecretValue::FilePath(PathBuf::from("/nonexistent/file.txt"));
    let result = secret.resolve().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_secret_value_encrypted_resolve() {
    let secret = SecretValue::Encrypted("encrypted-data".to_string());
    let result = secret.resolve().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "encrypted-data");
}

#[test]
fn test_secret_config_serialization() {
    let mut secrets = std::collections::HashMap::new();
    secrets.insert(
        "token".to_string(),
        SecretValue::Plain("test-token".to_string()),
    );

    let config = SecretConfig {
        secrets,
        encryption_key: Some("test-key".to_string()),
        key_file: Some(PathBuf::from("/tmp/key")),
    };

    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: SecretConfig = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.encryption_key, config.encryption_key);
    assert_eq!(deserialized.key_file, config.key_file);
    assert_eq!(deserialized.secrets.len(), config.secrets.len());
}

#[test]
fn test_secret_config_clone() {
    let mut secrets = std::collections::HashMap::new();
    secrets.insert(
        "token".to_string(),
        SecretValue::Plain("test-token".to_string()),
    );

    let config = SecretConfig {
        secrets,
        encryption_key: Some("test-key".to_string()),
        key_file: Some(PathBuf::from("/tmp/key")),
    };

    let cloned = config.clone();
    assert_eq!(cloned.encryption_key, config.encryption_key);
    assert_eq!(cloned.key_file, config.key_file);
    assert_eq!(cloned.secrets.len(), config.secrets.len());
}

#[test]
fn test_secret_config_debug() {
    let mut secrets = std::collections::HashMap::new();
    secrets.insert(
        "token".to_string(),
        SecretValue::Plain("test-token".to_string()),
    );

    let config = SecretConfig {
        secrets,
        encryption_key: Some("test-key".to_string()),
        key_file: Some(PathBuf::from("/tmp/key")),
    };

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("SecretConfig"));
}

#[tokio::test]
async fn test_secret_manager_load_from_env() {
    std::env::set_var("JIRA_TOKEN", "test-token");
    std::env::set_var("JIRA_EMAIL", "test@example.com");
    std::env::set_var("JIRA_BASE64_SECRET", "base64:dGVzdC1zZWNyZXQ=");
    std::env::set_var("JIRA_FILE_SECRET", "file:/tmp/secret.txt");
    std::env::set_var("JIRA_ENV_SECRET", "env:OTHER_VAR");
    std::env::set_var("OTHER_VAR", "env-secret-value");

    let mut manager = SecretManager::new();
    let result = manager.load_from_env("JIRA_").await;
    assert!(result.is_ok());

    // Check that secrets were loaded
    let token = manager.get_secret("token").await.unwrap();
    assert_eq!(token, Some("test-token".to_string()));

    let email = manager.get_secret("email").await.unwrap();
    assert_eq!(email, Some("test@example.com".to_string()));

    let base64_secret = manager.get_secret("base64_secret").await.unwrap();
    assert_eq!(base64_secret, Some("test-secret".to_string()));

    let env_secret = manager.get_secret("env_secret").await.unwrap();
    assert_eq!(env_secret, Some("env-secret-value".to_string()));

    // Clean up
    std::env::remove_var("JIRA_TOKEN");
    std::env::remove_var("JIRA_EMAIL");
    std::env::remove_var("JIRA_BASE64_SECRET");
    std::env::remove_var("JIRA_FILE_SECRET");
    std::env::remove_var("JIRA_ENV_SECRET");
    std::env::remove_var("OTHER_VAR");
}

#[tokio::test]
async fn test_secret_manager_load_from_env_no_prefix() {
    std::env::set_var("SOME_OTHER_VAR", "value");

    let mut manager = SecretManager::new();
    let result = manager.load_from_env("JIRA_").await;
    assert!(result.is_ok());

    // Should not load variables without the prefix
    let secret = manager.get_secret("some_other_var").await.unwrap();
    assert!(secret.is_none());

    std::env::remove_var("SOME_OTHER_VAR");
}

#[tokio::test]
async fn test_secret_manager_get_secret_nonexistent() {
    let manager = SecretManager::new();
    let result = manager.get_secret("nonexistent").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}
