use anyhow::Result;
use rust_jira_mcp::config::{ConfigManager, ConfigOptions, JiraConfig, SecretManager};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Simple Jira MCP Configuration Example ===\n");

    // Set up environment variables for this example
    std::env::set_var("JIRA_EMAIL", "demo@example.com");
    std::env::set_var("JIRA_PERSONAL_ACCESS_TOKEN", "demo_token_12345");

    // Example 1: Basic configuration loading
    println!("1. Basic Configuration Loading:");
    let mut config_manager = ConfigManager::new();
    let options = ConfigOptions {
        hot_reload: false,
        watch_paths: vec![],
        strict_validation: true,
        fail_on_missing: false, // Don't fail for this demo
    };

    match config_manager.load_with_options(options).await {
        Ok(()) => {
            let config = config_manager.get_config().await;
            println!("   ✓ Configuration loaded successfully");
            println!("   API URL: {}", config.api_base_url);
            println!("   Email: {}", config.email);
            println!("   Max Results: {:?}", config.max_results);
        }
        Err(e) => {
            println!("   ✗ Configuration loading failed: {e}");
        }
    }

    // Example 2: Secret management
    println!("\n2. Secret Management:");
    let mut secret_manager = SecretManager::new();

    // Add some secrets
    // Load secrets from environment
    secret_manager.load_from_env("JIRA_").await?;
    println!("   Environment secrets loaded successfully");

    // Example 3: Configuration validation
    println!("\n3. Configuration Validation:");
    let test_config = JiraConfig {
        email: "test@example.com".to_string(),
        personal_access_token: "valid_token_12345".to_string(),
        api_base_url: "https://jira.example.com/rest/api/2".to_string(),
        ..Default::default()
    };

    match test_config.validate() {
        Ok(()) => println!("   ✓ Configuration validation passed"),
        Err(e) => println!("   ✗ Configuration validation failed: {e}"),
    }

    // Example 4: Configuration sources
    println!("\n4. Configuration Sources:");
    let sources = config_manager.get_config_sources();
    for (i, source) in sources.iter().enumerate() {
        println!("   {}. {:?}", i + 1, source);
    }

    // Example 5: Hot-reload status
    println!("\n5. Hot-Reload Status:");
    println!(
        "   Hot-reload enabled: {}",
        config_manager.is_hot_reload_enabled()
    );

    println!("\n=== Configuration Example Complete ===");

    // Clean up environment variables
    std::env::remove_var("JIRA_EMAIL");
    std::env::remove_var("JIRA_PERSONAL_ACCESS_TOKEN");

    Ok(())
}
