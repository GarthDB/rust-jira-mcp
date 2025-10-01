use rust_jira_mcp::config::JiraConfig;
use rust_jira_mcp::jira::client::JiraClient;
use rust_jira_mcp::logging::{setup_logging, LoggingConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let logging_config = LoggingConfig::development();
    setup_logging(&logging_config);

    // Load configuration from environment
    let config = JiraConfig::load()?;

    println!("=== Jira MCP Debug Tool ===");
    println!("API Base URL: {}", config.api_base_url);
    println!("Email: {}", config.email);
    println!("Auth Header: {}", config.auth_header());
    println!();

    // Create client
    let client = JiraClient::new(config)?;

    println!("=== Testing Authentication ===");

    // Test the /myself endpoint (simpler than search)
    match client.get::<serde_json::Value>("myself").await {
        Ok(user_info) => {
            println!("✅ Authentication successful!");
            println!("User info: {}", serde_json::to_string_pretty(&user_info)?);
        }
        Err(e) => {
            println!("❌ Authentication failed: {}", e);
            println!("Error details: {:?}", e);
        }
    }

    println!();
    println!("=== Testing Search Endpoint ===");

    // Test the search endpoint that's failing
    match client
        .get::<serde_json::Value>("search?jql=project=DNA%20AND%20status=Open&maxResults=1")
        .await
    {
        Ok(search_result) => {
            println!("✅ Search successful!");
            println!(
                "Search result: {}",
                serde_json::to_string_pretty(&search_result)?
            );
        }
        Err(e) => {
            println!("❌ Search failed: {}", e);
            println!("Error details: {:?}", e);
        }
    }

    Ok(())
}
