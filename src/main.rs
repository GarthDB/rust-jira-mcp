use anyhow::Result;
use std::collections::HashMap;
use tracing::info;

mod config;
mod error;
mod jira;
mod logging;
mod mcp;
mod test_usage;
mod types;
mod utils;

use crate::config::{jira::JiraConfig, ConfigManager, ConfigOptions, SecretManager};
use crate::logging::{Logger, LoggingConfig, MetricsCollector};
use crate::mcp::server::MCPServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize comprehensive logging system
    let logging_config = if std::env::var("RUST_LOG").is_ok() {
        // Use environment-based configuration
        LoggingConfig::default()
    } else if std::env::var("JIRA_ENV").unwrap_or_default() == "production" {
        LoggingConfig::production()
    } else {
        LoggingConfig::development()
    };

    // Set up logging
    crate::logging::setup_logging(&logging_config);

    // Initialize metrics collector
    let metrics_collector = MetricsCollector::new();
    let logger = Logger::new(metrics_collector.clone());

    // Health monitoring removed for now

    info!("Starting Rust Jira MCP Server with comprehensive logging and monitoring");

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

    // Health checks removed for now

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

    // Log startup metrics
    let mut startup_metadata = HashMap::new();
    startup_metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    startup_metadata.insert(
        "api_url".to_string(),
        config_with_secrets.api_base_url.clone(),
    );
    startup_metadata.insert("email".to_string(), config_with_secrets.email.clone());
    startup_metadata.insert(
        "hot_reload".to_string(),
        config_manager.is_hot_reload_enabled().to_string(),
    );

    logger.log_operation_success(
        "server_startup",
        std::time::Duration::from_millis(0),
        &startup_metadata,
    );

    // Create and run MCP server with the configuration that includes secrets
    let mut server = MCPServer::new(config_with_secrets);

    info!("MCP Server initialized, starting stdio transport");

    // Log server startup completion
    let mut server_metadata = HashMap::new();
    server_metadata.insert("transport".to_string(), "stdio".to_string());
    logger.log_operation_success(
        "server_initialization",
        std::time::Duration::from_millis(0),
        &server_metadata,
    );

    // Run the server
    server.run_stdio().await?;

    // Log server shutdown
    let mut shutdown_metadata = HashMap::new();
    shutdown_metadata.insert("reason".to_string(), "normal_shutdown".to_string());
    logger.log_operation_success(
        "server_shutdown",
        std::time::Duration::from_millis(0),
        &shutdown_metadata,
    );

    Ok(())
}
