use rust_jira_mcp::config::JiraConfig;
use rust_jira_mcp::mcp::server::MCPToolHandler;
use rust_jira_mcp::mcp::tools::*;
use serde_json::json;

/// Example demonstrating the new project configuration and metadata tools
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create a test configuration
    let config = JiraConfig {
        api_base_url: "https://your-jira-instance.atlassian.net/rest/api/2".to_string(),
        email: "your-email@example.com".to_string(),
        personal_access_token: "your-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(true),
    };

    println!("🚀 Rust Jira MCP - Project Configuration and Metadata Example");
    println!("=============================================================");

    // Create tools
    let project_config_tool = GetProjectConfigTool::new(config.clone());
    let issue_types_tool = GetIssueTypesTool::new(config.clone());
    let components_tool = GetProjectComponentsTool::new(config.clone());
    let priorities_statuses_tool = GetPrioritiesAndStatusesTool::new(config.clone());
    let custom_fields_tool = GetCustomFieldsTool::new(config.clone());
    let metadata_tool = GetProjectMetadataTool::new(config.clone());

    // Example project key (replace with your actual project key)
    let project_key = "TEST";

    println!("\n📋 Available Project Configuration and Metadata Tools:");
    println!("1. get_project_config - Get project configuration details");
    println!("2. get_project_issue_types - Get issue types for a project");
    println!("3. get_issue_type_metadata - Get detailed issue type information");
    println!("4. get_project_components - Get project components");
    println!("5. get_priorities_and_statuses - Get priorities and statuses");
    println!("6. get_custom_fields - Get custom field definitions");
    println!("7. get_project_metadata - Get comprehensive project metadata");

    println!("\n🔧 Tool Usage Examples:");
    println!("======================");

    // Example 1: Get project configuration
    println!("\n1. Getting project configuration...");
    let config_args = json!({
        "project_key": project_key
    });

    match project_config_tool.handle(config_args).await {
        Ok(result) => {
            println!("✅ Project configuration retrieved successfully!");
            println!("Response: {}", result.content[0].text);
        }
        Err(e) => {
            println!("❌ Error getting project configuration: {}", e);
        }
    }

    // Example 2: Get project issue types
    println!("\n2. Getting project issue types...");
    let issue_types_args = json!({
        "project_key": project_key
    });

    match issue_types_tool.handle(issue_types_args).await {
        Ok(result) => {
            println!("✅ Project issue types retrieved successfully!");
            println!("Response: {}", result.content[0].text);
        }
        Err(e) => {
            println!("❌ Error getting project issue types: {}", e);
        }
    }

    // Example 3: Get project components
    println!("\n3. Getting project components...");
    let components_args = json!({
        "project_key": project_key
    });

    match components_tool.handle(components_args).await {
        Ok(result) => {
            println!("✅ Project components retrieved successfully!");
            println!("Response: {}", result.content[0].text);
        }
        Err(e) => {
            println!("❌ Error getting project components: {}", e);
        }
    }

    // Example 4: Get priorities and statuses
    println!("\n4. Getting priorities and statuses...");
    let priorities_args = json!({});

    match priorities_statuses_tool.handle(priorities_args).await {
        Ok(result) => {
            println!("✅ Priorities and statuses retrieved successfully!");
            println!("Response: {}", result.content[0].text);
        }
        Err(e) => {
            println!("❌ Error getting priorities and statuses: {}", e);
        }
    }

    // Example 5: Get custom fields
    println!("\n5. Getting custom fields...");
    let custom_fields_args = json!({});

    match custom_fields_tool.handle(custom_fields_args).await {
        Ok(result) => {
            println!("✅ Custom fields retrieved successfully!");
            println!("Response: {}", result.content[0].text);
        }
        Err(e) => {
            println!("❌ Error getting custom fields: {}", e);
        }
    }

    // Example 6: Get comprehensive project metadata
    println!("\n6. Getting comprehensive project metadata...");
    let metadata_args = json!({
        "project_key": project_key
    });

    match metadata_tool.handle(metadata_args).await {
        Ok(result) => {
            println!("✅ Comprehensive project metadata retrieved successfully!");
            println!("Response: {}", result.content[0].text);
        }
        Err(e) => {
            println!("❌ Error getting project metadata: {}", e);
        }
    }

    println!("\n🎉 Example completed!");
    println!("\n💡 To use these tools in your MCP client:");
    println!("   - Configure your Jira credentials in the config");
    println!("   - Use the tool names: get_project_config, get_project_issue_types, etc.");
    println!("   - Pass the required parameters as JSON arguments");

    Ok(())
}
