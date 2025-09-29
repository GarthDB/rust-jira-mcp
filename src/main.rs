use anyhow::Result;
use tracing::info;

mod config;
mod error;
mod jira;
mod mcp;
mod test_usage;
mod types;
mod utils;

use crate::config::{jira::JiraConfig, ConfigManager, ConfigOptions, SecretManager};
use crate::mcp::server::MCPServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Rust Jira MCP Server");

    // Test that our code compiles and uses all functions
    crate::test_usage::test_usage();

    // Initialize configuration manager with hot-reloading
    let mut config_manager = ConfigManager::new();
    let config_options = ConfigOptions {
        hot_reload: std::env::var("JIRA_HOT_RELOAD").is_ok(),
        watch_paths: vec![
            std::path::PathBuf::from("config/default.toml"),
            std::path::PathBuf::from("config/local.toml"),
        ],
        strict_validation: true,
        fail_on_missing: true,
    };

    // Load configuration with options
    config_manager.load_with_options(config_options).await?;
    let _config = config_manager.get_config().await;

    // Initialize secret manager
    let mut secret_manager = SecretManager::new();

    // Load secrets from file if it exists
    if std::path::Path::new("config/secrets.toml").exists() {
        secret_manager
            .load_from_file(&std::path::PathBuf::from("config/secrets.toml"))
            .await?;
        info!("Loaded secrets from file");
    }

    // Load secrets from environment variables
    secret_manager.load_from_env("JIRA_").await?;
    info!("Loaded secrets from environment variables");

    // Load configuration with secrets
    let config_with_secrets = JiraConfig::load_with_secrets(&secret_manager).await?;

    // Validate configuration
    config_with_secrets.validate()?;

    info!(
        "Configuration loaded successfully: API URL = {}, Email = {}",
        config_with_secrets.api_base_url, config_with_secrets.email
    );

    // Show configuration sources
    let sources = config_manager.get_config_sources();
    info!("Configuration sources: {:?}", sources);

    // Show if hot-reloading is enabled
    if config_manager.is_hot_reload_enabled() {
        info!("Hot-reloading is enabled");
    }

    // Create and run MCP server with the configuration that includes secrets
    let mut server = MCPServer::new(config_with_secrets);

    info!("MCP Server initialized, starting stdio transport");
    server.run_stdio().await?;

    Ok(())
}
