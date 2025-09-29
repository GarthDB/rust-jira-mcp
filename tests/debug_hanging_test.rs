use rust_jira_mcp::config::JiraConfig;
use rust_jira_mcp::jira::client::JiraClient;

#[test]
fn test_jira_client_creation() {
    let config = JiraConfig {
        api_base_url: "https://test-jira.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(false),
    };

    let _client = JiraClient::new(config);
    // If we get here, the client creation succeeded
}

#[test]
fn test_mcp_server_list_tools() {
    use rust_jira_mcp::mcp::server::MCPServer;
    
    let tools = MCPServer::list_tools();
    assert!(!tools.is_empty());
}

#[test]
fn test_mcp_server_creation_minimal() {
    use rust_jira_mcp::mcp::server::MCPServer;
    
    let config = JiraConfig {
        api_base_url: "https://test-jira.example.com/rest/api/2".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(false),
    };

    let _server = MCPServer::new(config);
    // If we get here, the server creation succeeded
}
