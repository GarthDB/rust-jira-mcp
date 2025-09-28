use rust_jira_mcp::config::JiraConfig;
use rust_jira_mcp::mcp::server::MCPServer;
use serde_json::json;

#[tokio::test]
async fn test_zephyr_tools_registration() {
    let config = JiraConfig {
        api_base_url: "https://example.atlassian.net".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test_token".to_string(),
        timeout_seconds: Some(30),
        max_results: Some(50),
        strict_ssl: Some(true),
        default_project: None,
        log_file: None,
    };

    let server = MCPServer::new(config);
    
    // Test that Zephyr tools are registered
    let tools = MCPServer::list_tools();
    let tool_names: Vec<&String> = tools.iter().map(|t| &t.name).collect();
    
    // Check that Zephyr tools are present
    assert!(tool_names.contains(&&"get_zephyr_test_steps".to_string()));
    assert!(tool_names.contains(&&"create_zephyr_test_step".to_string()));
    assert!(tool_names.contains(&&"update_zephyr_test_step".to_string()));
    assert!(tool_names.contains(&&"delete_zephyr_test_step".to_string()));
    assert!(tool_names.contains(&&"get_zephyr_test_cases".to_string()));
    assert!(tool_names.contains(&&"create_zephyr_test_case".to_string()));
    assert!(tool_names.contains(&&"get_zephyr_test_executions".to_string()));
    assert!(tool_names.contains(&&"create_zephyr_test_execution".to_string()));
    assert!(tool_names.contains(&&"get_zephyr_test_cycles".to_string()));
    assert!(tool_names.contains(&&"get_zephyr_test_plans".to_string()));
}

#[tokio::test]
async fn test_zephyr_tool_schemas() {
    let tools = MCPServer::list_tools();
    
    // Find Zephyr tools and verify their schemas
    let get_test_steps_tool = tools.iter()
        .find(|t| t.name == "get_zephyr_test_steps")
        .expect("get_zephyr_test_steps tool should be registered");
    
    assert_eq!(get_test_steps_tool.name, "get_zephyr_test_steps");
    assert!(get_test_steps_tool.description.contains("test steps"));
    
    // Verify the schema has required fields
    let schema = &get_test_steps_tool.input_schema;
    assert!(schema["properties"]["test_case_id"].is_object());
    assert!(schema["required"].as_array().unwrap().contains(&json!("test_case_id")));
    
    let create_test_step_tool = tools.iter()
        .find(|t| t.name == "create_zephyr_test_step")
        .expect("create_zephyr_test_step tool should be registered");
    
    assert_eq!(create_test_step_tool.name, "create_zephyr_test_step");
    assert!(create_test_step_tool.description.contains("Create a new test step"));
    
    // Verify required fields for creating test steps
    let schema = &create_test_step_tool.input_schema;
    let required_fields = schema["required"].as_array().unwrap();
    assert!(required_fields.contains(&json!("test_case_id")));
    assert!(required_fields.contains(&json!("step")));
    assert!(required_fields.contains(&json!("order")));
}

#[tokio::test]
async fn test_zephyr_tool_count() {
    let tools = MCPServer::list_tools();
    
    // Count Zephyr tools
    let zephyr_tools: Vec<&rust_jira_mcp::types::mcp::MCPTool> = tools.iter()
        .filter(|t| t.name.contains("zephyr"))
        .collect();
    
    // We should have 10 Zephyr tools
    assert_eq!(zephyr_tools.len(), 10, "Expected 10 Zephyr tools, found {}", zephyr_tools.len());
    
    // Verify all expected tool names are present
    let expected_tools = vec![
        "get_zephyr_test_steps",
        "create_zephyr_test_step", 
        "update_zephyr_test_step",
        "delete_zephyr_test_step",
        "get_zephyr_test_cases",
        "create_zephyr_test_case",
        "get_zephyr_test_executions",
        "create_zephyr_test_execution",
        "get_zephyr_test_cycles",
        "get_zephyr_test_plans",
    ];
    
    for expected_tool in expected_tools {
        assert!(
            zephyr_tools.iter().any(|t| t.name == expected_tool),
            "Expected tool '{}' not found", expected_tool
        );
    }
}
