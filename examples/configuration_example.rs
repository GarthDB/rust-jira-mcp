use anyhow::Result;
use rust_jira_mcp::config::{ConfigManager, ConfigOptions, JiraConfig, SecretManager};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Jira MCP Configuration Management Example ===\n");

    // Example 1: Basic configuration loading
    println!("1. Basic Configuration Loading:");
    let mut config_manager = ConfigManager::new();
    config_manager
        .load_with_options(ConfigOptions::default())
        .await?;
    let config = config_manager.get_config().await;
    println!("   Loaded config: {config:?}\n");

    // Example 2: Configuration with hot-reloading
    println!("2. Configuration with Hot-Reloading:");
    let mut config_manager = ConfigManager::new();
    let options = ConfigOptions {
        hot_reload: true,
        watch_paths: vec![PathBuf::from("config/default.toml")],
        strict_validation: true,
        fail_on_missing: false, // Don't fail on missing required fields for demo
    };
    config_manager.load_with_options(options).await?;
    println!(
        "   Hot-reload enabled: {}\n",
        config_manager.is_hot_reload_enabled()
    );

    // Example 3: Secret management
    println!("3. Secret Management:");
    let secret_manager = SecretManager::new();
    println!("   Secret management functionality available via SecretManager");

    // Example 4: Configuration with secrets
    println!("\n4. Configuration with Secrets:");
    let config_with_secrets = JiraConfig::load_with_secrets(&secret_manager).await?;
    println!(
        "   Config with secrets: email={}, token={}...",
        config_with_secrets.email,
        &config_with_secrets.personal_access_token[..10]
    );

    // Example 5: Configuration validation
    println!("\n5. Configuration Validation:");
    match config_with_secrets.validate() {
        Ok(()) => println!("   ✓ Configuration is valid"),
        Err(e) => println!("   ✗ Configuration validation failed: {e}"),
    }

    // Example 6: Configuration sources
    println!("\n6. Configuration Sources:");
    let sources = config_manager.get_config_sources();
    for (i, source) in sources.iter().enumerate() {
        println!("   {}. {:?}", i + 1, source);
    }

    // Example 7: Environment variable loading
    println!("\n7. Loading from Environment Variables:");
    let mut env_secret_manager = SecretManager::new();
    env_secret_manager.load_from_env("JIRA_").await?;
    println!("   Environment secrets loaded successfully");

    // Example 8: Error handling
    println!("\n8. Error Handling:");
    let invalid_config = JiraConfig {
        email: "invalid-email".to_string(),
        personal_access_token: "short".to_string(),
        ..Default::default()
    };

    match invalid_config.validate() {
        Ok(()) => println!("   ✓ Invalid config somehow passed validation"),
        Err(e) => println!("   ✗ Expected validation error: {e}"),
    }

    println!("\n=== Configuration Example Complete ===");
    Ok(())
}
