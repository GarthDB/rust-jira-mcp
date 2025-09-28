use anyhow::Result;
use tracing::info;

mod config;
mod error;
mod jira;
mod mcp;
mod test_usage;
mod types;
mod utils;

use crate::config::jira::JiraConfig;
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

    // Load configuration
    let config = JiraConfig::load()?;
    info!("Configuration loaded successfully");

    // Create and run MCP server
    let mut server = MCPServer::new(config);

    info!("MCP Server initialized, starting stdio transport");
    server.run_stdio().await?;

    Ok(())
}
