use rust_jira_mcp::types::jira::*;
use rust_jira_mcp::types::mcp::*;
use serde_json::json;
use std::collections::HashMap;

// Jira Types Tests

#[test]
fn test_jira_issue_serialization() {
    let mut fields = HashMap::new();
    fields.insert("summary".to_string(), json!("Test Issue"));
    fields.insert("description".to_string(), json!("Test Description"));

    let issue = JiraIssue {
        id: "10001".to_string(),
        key: "TEST-1".to_string(),
        self_url: "https://example.com/rest/api/2/issue/10001".to_string(),
        fields,
    };

    let serialized = serde_json::to_string(&issue).unwrap();
    let deserialized: JiraIssue = serde_json::from_str(&serialized).unwrap();

    assert_eq!(issue.id, deserialized.id);
    assert_eq!(issue.key, deserialized.key);
    assert_eq!(issue.self_url, deserialized.self_url);
    assert_eq!(issue.fields.len(), deserialized.fields.len());
}

#[test]
fn test_jira_project_serialization() {
    let project = JiraProject {
        id: "10000".to_string(),
        key: "TEST".to_string(),
        name: "Test Project".to_string(),
        project_type_key: "software".to_string(),
        self_url: "https://example.com/rest/api/2/project/10000".to_string(),
    };

    let serialized = serde_json::to_string(&project).unwrap();
    let deserialized: JiraProject = serde_json::from_str(&serialized).unwrap();

    assert_eq!(project.id, deserialized.id);
    assert_eq!(project.key, deserialized.key);
    assert_eq!(project.name, deserialized.name);
    assert_eq!(project.project_type_key, deserialized.project_type_key);
    assert_eq!(project.self_url, deserialized.self_url);
}

#[test]
fn test_jira_user_serialization() {
    let user = JiraUser {
        account_id: "5d5f8b8b8b8b8b8b8b8b8b8b".to_string(),
        display_name: "John Doe".to_string(),
        email_address: Some("john.doe@example.com".to_string()),
        active: true,
        time_zone: Some("America/New_York".to_string()),
    };

    let serialized = serde_json::to_string(&user).unwrap();
    let deserialized: JiraUser = serde_json::from_str(&serialized).unwrap();

    assert_eq!(user.account_id, deserialized.account_id);
    assert_eq!(user.display_name, deserialized.display_name);
    assert_eq!(user.email_address, deserialized.email_address);
    assert_eq!(user.active, deserialized.active);
    assert_eq!(user.time_zone, deserialized.time_zone);
}

#[test]
fn test_jira_issue_type_serialization() {
    let issue_type = JiraIssueType {
        id: "10001".to_string(),
        name: "Bug".to_string(),
        description: Some(
            "A problem which impairs or prevents the functions of the product.".to_string(),
        ),
        icon_url: Some("https://example.com/icons/bug.png".to_string()),
        subtask: false,
    };

    let serialized = serde_json::to_string(&issue_type).unwrap();
    let deserialized: JiraIssueType = serde_json::from_str(&serialized).unwrap();

    assert_eq!(issue_type.id, deserialized.id);
    assert_eq!(issue_type.name, deserialized.name);
    assert_eq!(issue_type.description, deserialized.description);
    assert_eq!(issue_type.icon_url, deserialized.icon_url);
    assert_eq!(issue_type.subtask, deserialized.subtask);
}

#[test]
fn test_jira_priority_serialization() {
    let priority = JiraPriority {
        id: "3".to_string(),
        name: "Medium".to_string(),
        description: Some("Medium priority".to_string()),
        icon_url: Some("https://example.com/icons/medium.png".to_string()),
    };

    let serialized = serde_json::to_string(&priority).unwrap();
    let deserialized: JiraPriority = serde_json::from_str(&serialized).unwrap();

    assert_eq!(priority.id, deserialized.id);
    assert_eq!(priority.name, deserialized.name);
    assert_eq!(priority.description, deserialized.description);
    assert_eq!(priority.icon_url, deserialized.icon_url);
}

#[test]
fn test_jira_status_serialization() {
    let status_category = JiraStatusCategory {
        id: 2,
        key: "new".to_string(),
        color_name: "blue-gray".to_string(),
        name: "To Do".to_string(),
    };

    let status = JiraStatus {
        id: "1".to_string(),
        name: "To Do".to_string(),
        description: Some("This issue is in the backlog".to_string()),
        icon_url: Some("https://example.com/icons/todo.png".to_string()),
        status_category,
    };

    let serialized = serde_json::to_string(&status).unwrap();
    let deserialized: JiraStatus = serde_json::from_str(&serialized).unwrap();

    assert_eq!(status.id, deserialized.id);
    assert_eq!(status.name, deserialized.name);
    assert_eq!(status.description, deserialized.description);
    assert_eq!(status.icon_url, deserialized.icon_url);
    assert_eq!(status.status_category.id, deserialized.status_category.id);
    assert_eq!(status.status_category.key, deserialized.status_category.key);
    assert_eq!(
        status.status_category.color_name,
        deserialized.status_category.color_name
    );
    assert_eq!(
        status.status_category.name,
        deserialized.status_category.name
    );
}

#[test]
fn test_jira_comment_serialization() {
    let author = JiraUser {
        account_id: "5d5f8b8b8b8b8b8b8b8b8b8b".to_string(),
        display_name: "John Doe".to_string(),
        email_address: Some("john.doe@example.com".to_string()),
        active: true,
        time_zone: Some("America/New_York".to_string()),
    };

    let comment = JiraComment {
        id: "10001".to_string(),
        body: "This is a test comment".to_string(),
        author,
        created: "2023-01-01T00:00:00.000+0000".to_string(),
        updated: Some("2023-01-01T00:00:00.000+0000".to_string()),
    };

    let serialized = serde_json::to_string(&comment).unwrap();
    let deserialized: JiraComment = serde_json::from_str(&serialized).unwrap();

    assert_eq!(comment.id, deserialized.id);
    assert_eq!(comment.body, deserialized.body);
    assert_eq!(comment.author.account_id, deserialized.author.account_id);
    assert_eq!(comment.created, deserialized.created);
    assert_eq!(comment.updated, deserialized.updated);
}

#[test]
fn test_transition_properties_new() {
    let props = TransitionProperties::new();

    assert!(matches!(props.screen, ScreenProperty::NoScreen));
    assert!(matches!(props.scope, ScopeProperty::Local));
    assert!(matches!(
        props.availability,
        AvailabilityProperty::Unavailable
    ));
    assert!(matches!(
        props.conditionality,
        ConditionalityProperty::Unconditional
    ));
}

#[test]
fn test_transition_properties_with_all_properties() {
    let props = TransitionProperties::with_all_properties();

    assert!(matches!(props.screen, ScreenProperty::HasScreen));
    assert!(matches!(props.scope, ScopeProperty::Global));
    assert!(matches!(
        props.availability,
        AvailabilityProperty::Available
    ));
    assert!(matches!(
        props.conditionality,
        ConditionalityProperty::Conditional
    ));
}

#[test]
fn test_transition_properties_default() {
    let props = TransitionProperties::default();

    assert!(matches!(props.screen, ScreenProperty::NoScreen));
    assert!(matches!(props.scope, ScopeProperty::Local));
    assert!(matches!(
        props.availability,
        AvailabilityProperty::Unavailable
    ));
    assert!(matches!(
        props.conditionality,
        ConditionalityProperty::Unconditional
    ));
}

#[test]
fn test_jira_search_result_serialization() {
    let mut fields = HashMap::new();
    fields.insert("summary".to_string(), json!("Test Issue"));

    let issue = JiraIssue {
        id: "10001".to_string(),
        key: "TEST-1".to_string(),
        self_url: "https://example.com/rest/api/2/issue/10001".to_string(),
        fields,
    };

    let search_result = JiraSearchResult {
        expand: Some("renderedFields".to_string()),
        start_at: 0,
        max_results: 50,
        total: 1,
        issues: vec![issue],
    };

    let serialized = serde_json::to_string(&search_result).unwrap();
    let deserialized: JiraSearchResult = serde_json::from_str(&serialized).unwrap();

    assert_eq!(search_result.expand, deserialized.expand);
    assert_eq!(search_result.start_at, deserialized.start_at);
    assert_eq!(search_result.max_results, deserialized.max_results);
    assert_eq!(search_result.total, deserialized.total);
    assert_eq!(search_result.issues.len(), deserialized.issues.len());
}

#[test]
fn test_jira_work_log_serialization() {
    let author = JiraUser {
        account_id: "5d5f8b8b8b8b8b8b8b8b8b8b".to_string(),
        display_name: "John Doe".to_string(),
        email_address: Some("john.doe@example.com".to_string()),
        active: true,
        time_zone: Some("America/New_York".to_string()),
    };

    let work_log = JiraWorkLog {
        id: "10001".to_string(),
        comment: Some("Worked on this issue".to_string()),
        time_spent: "2h 30m".to_string(),
        time_spent_seconds: 9000,
        author,
        created: "2023-01-01T00:00:00.000+0000".to_string(),
        updated: Some("2023-01-01T00:00:00.000+0000".to_string()),
    };

    let serialized = serde_json::to_string(&work_log).unwrap();
    let deserialized: JiraWorkLog = serde_json::from_str(&serialized).unwrap();

    assert_eq!(work_log.id, deserialized.id);
    assert_eq!(work_log.comment, deserialized.comment);
    assert_eq!(work_log.time_spent, deserialized.time_spent);
    assert_eq!(work_log.time_spent_seconds, deserialized.time_spent_seconds);
    assert_eq!(work_log.author.account_id, deserialized.author.account_id);
    assert_eq!(work_log.created, deserialized.created);
    assert_eq!(work_log.updated, deserialized.updated);
}

#[test]
fn test_zephyr_test_step_serialization() {
    let test_step = ZephyrTestStep {
        id: Some("10001".to_string()),
        step: "Click the login button".to_string(),
        data: Some("Login button should be visible".to_string()),
        result: Some("Login button was clicked successfully".to_string()),
        order: 1,
        test_case_id: Some("TEST-1".to_string()),
    };

    let serialized = serde_json::to_string(&test_step).unwrap();
    let deserialized: ZephyrTestStep = serde_json::from_str(&serialized).unwrap();

    assert_eq!(test_step.id, deserialized.id);
    assert_eq!(test_step.step, deserialized.step);
    assert_eq!(test_step.data, deserialized.data);
    assert_eq!(test_step.result, deserialized.result);
    assert_eq!(test_step.order, deserialized.order);
    assert_eq!(test_step.test_case_id, deserialized.test_case_id);
}

#[test]
fn test_zephyr_test_case_serialization() {
    let test_case = ZephyrTestCase {
        id: Some("10001".to_string()),
        key: Some("TEST-1".to_string()),
        name: "Test Login Functionality".to_string(),
        project_key: "TEST".to_string(),
        issue_type: "Test".to_string(),
        status: Some("Draft".to_string()),
        priority: Some("High".to_string()),
        assignee: Some("john.doe@example.com".to_string()),
        description: Some("Test the login functionality".to_string()),
        labels: Some(vec!["login".to_string(), "authentication".to_string()]),
        components: Some(vec!["UI".to_string(), "Backend".to_string()]),
        fix_versions: Some(vec!["1.0".to_string()]),
        custom_fields: Some(HashMap::new()),
    };

    let serialized = serde_json::to_string(&test_case).unwrap();
    let deserialized: ZephyrTestCase = serde_json::from_str(&serialized).unwrap();

    assert_eq!(test_case.id, deserialized.id);
    assert_eq!(test_case.key, deserialized.key);
    assert_eq!(test_case.name, deserialized.name);
    assert_eq!(test_case.project_key, deserialized.project_key);
    assert_eq!(test_case.issue_type, deserialized.issue_type);
    assert_eq!(test_case.status, deserialized.status);
    assert_eq!(test_case.priority, deserialized.priority);
    assert_eq!(test_case.assignee, deserialized.assignee);
    assert_eq!(test_case.description, deserialized.description);
    assert_eq!(test_case.labels, deserialized.labels);
    assert_eq!(test_case.components, deserialized.components);
    assert_eq!(test_case.fix_versions, deserialized.fix_versions);
}

#[test]
fn test_bulk_operation_config_default() {
    let config = BulkOperationConfig::default();

    assert_eq!(config.batch_size, Some(10));
    assert!(config.continue_on_error);
    assert_eq!(config.rate_limit_ms, Some(100));
    assert_eq!(config.max_retries, Some(3));
}

#[test]
fn test_bulk_operation_summary_new() {
    let summary = BulkOperationSummary::new();

    assert_eq!(summary.total_operations, 0);
    assert_eq!(summary.successful_operations, 0);
    assert_eq!(summary.failed_operations, 0);
    assert!(summary.results.is_empty());
    assert_eq!(summary.duration_ms, 0);
}

#[test]
fn test_bulk_operation_summary_add_result() {
    let mut summary = BulkOperationSummary::new();

    let success_result = BulkOperationResult {
        issue_key: "TEST-1".to_string(),
        success: true,
        error_message: None,
        operation_type: BulkOperationType::Update,
    };

    let failure_result = BulkOperationResult {
        issue_key: "TEST-2".to_string(),
        success: false,
        error_message: Some("Error occurred".to_string()),
        operation_type: BulkOperationType::Update,
    };

    summary.add_result(success_result);
    summary.add_result(failure_result);

    assert_eq!(summary.total_operations, 2);
    assert_eq!(summary.successful_operations, 1);
    assert_eq!(summary.failed_operations, 1);
    assert_eq!(summary.results.len(), 2);
}

#[test]
fn test_bulk_operation_summary_success_rate() {
    let mut summary = BulkOperationSummary::new();

    // Test with no operations
    assert_eq!(summary.success_rate(), 0.0);

    // Test with all successful operations
    summary.add_result(BulkOperationResult {
        issue_key: "TEST-1".to_string(),
        success: true,
        error_message: None,
        operation_type: BulkOperationType::Update,
    });
    summary.add_result(BulkOperationResult {
        issue_key: "TEST-2".to_string(),
        success: true,
        error_message: None,
        operation_type: BulkOperationType::Update,
    });

    assert_eq!(summary.success_rate(), 100.0);

    // Test with mixed results
    summary.add_result(BulkOperationResult {
        issue_key: "TEST-3".to_string(),
        success: false,
        error_message: Some("Error".to_string()),
        operation_type: BulkOperationType::Update,
    });

    // Use approximate equality for floating point comparison
    let rate = summary.success_rate();
    assert!((rate - 66.66666666666667).abs() < 0.1); // More lenient tolerance
}

#[test]
fn test_bulk_operation_summary_default() {
    let summary = BulkOperationSummary::default();

    assert_eq!(summary.total_operations, 0);
    assert_eq!(summary.successful_operations, 0);
    assert_eq!(summary.failed_operations, 0);
    assert!(summary.results.is_empty());
    assert_eq!(summary.duration_ms, 0);
}

#[test]
fn test_jira_field_mapping_default() {
    let mapping = JiraFieldMapping::default();

    assert!(mapping.copy_fields.contains(&"summary".to_string()));
    assert!(mapping.copy_fields.contains(&"description".to_string()));
    assert!(mapping.exclude_fields.contains(&"assignee".to_string()));
    assert!(mapping.exclude_fields.contains(&"reporter".to_string()));
    assert!(mapping.custom_field_mapping.is_none());
}

// MCP Types Tests

#[test]
fn test_json_rpc_request_serialization() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!("123")),
        method: "initialize".to_string(),
        params: Some(json!({"protocol_version": "2024-11-05"})),
    };

    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();

    assert_eq!(request.jsonrpc, deserialized.jsonrpc);
    assert_eq!(request.id, deserialized.id);
    assert_eq!(request.method, deserialized.method);
    assert_eq!(request.params, deserialized.params);
}

#[test]
fn test_json_rpc_response_serialization() {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(json!("123")),
        result: Some(json!({"success": true})),
        error: None,
    };

    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: JsonRpcResponse = serde_json::from_str(&serialized).unwrap();

    assert_eq!(response.jsonrpc, deserialized.jsonrpc);
    assert_eq!(response.id, deserialized.id);
    assert_eq!(response.result, deserialized.result);
    // JsonRpcError doesn't implement PartialEq, so we compare by matching variants
    match (&response.error, &deserialized.error) {
        (None, None) => {}
        (Some(e1), Some(e2)) => {
            assert_eq!(e1.code, e2.code);
            assert_eq!(e1.message, e2.message);
            assert_eq!(e1.data, e2.data);
        }
        _ => panic!("Error field mismatch"),
    }
}

#[test]
fn test_json_rpc_error_serialization() {
    let error = JsonRpcError {
        code: -32601,
        message: "Method not found".to_string(),
        data: Some(json!({"method": "unknown"})),
    };

    let serialized = serde_json::to_string(&error).unwrap();
    let deserialized: JsonRpcError = serde_json::from_str(&serialized).unwrap();

    assert_eq!(error.code, deserialized.code);
    assert_eq!(error.message, deserialized.message);
    assert_eq!(error.data, deserialized.data);
}

#[test]
fn test_initialize_params_serialization() {
    let client_info = ClientInfo {
        name: "Test Client".to_string(),
        version: "1.0.0".to_string(),
    };

    let params = InitializeParams {
        protocol_version: "2024-11-05".to_string(),
        capabilities: json!({"tools": {}}),
        client_info,
    };

    let serialized = serde_json::to_string(&params).unwrap();
    let deserialized: InitializeParams = serde_json::from_str(&serialized).unwrap();

    assert_eq!(params.protocol_version, deserialized.protocol_version);
    assert_eq!(params.capabilities, deserialized.capabilities);
    assert_eq!(params.client_info.name, deserialized.client_info.name);
    assert_eq!(params.client_info.version, deserialized.client_info.version);
}

#[test]
fn test_initialize_result_serialization() {
    let server_capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
    };

    let server_info = ServerInfo {
        name: "Jira MCP Server".to_string(),
        version: "1.0.0".to_string(),
    };

    let result = InitializeResult {
        protocol_version: "2024-11-05".to_string(),
        capabilities: server_capabilities,
        server_info,
    };

    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: InitializeResult = serde_json::from_str(&serialized).unwrap();

    assert_eq!(result.protocol_version, deserialized.protocol_version);
    assert_eq!(
        result.capabilities.tools.is_some(),
        deserialized.capabilities.tools.is_some()
    );
    assert_eq!(result.server_info.name, deserialized.server_info.name);
    assert_eq!(result.server_info.version, deserialized.server_info.version);
}

#[test]
fn test_mcp_tool_serialization() {
    let tool = MCPTool {
        name: "get_issue".to_string(),
        description: "Get a Jira issue by key".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "issue_key": {
                    "type": "string",
                    "description": "The issue key"
                }
            },
            "required": ["issue_key"]
        }),
    };

    let serialized = serde_json::to_string(&tool).unwrap();
    let deserialized: MCPTool = serde_json::from_str(&serialized).unwrap();

    assert_eq!(tool.name, deserialized.name);
    assert_eq!(tool.description, deserialized.description);
    assert_eq!(tool.input_schema, deserialized.input_schema);
}

#[test]
fn test_mcp_tool_call_serialization() {
    let tool_call = MCPToolCall {
        name: "get_issue".to_string(),
        arguments: json!({
            "issue_key": "TEST-1"
        }),
    };

    let serialized = serde_json::to_string(&tool_call).unwrap();
    let deserialized: MCPToolCall = serde_json::from_str(&serialized).unwrap();

    assert_eq!(tool_call.name, deserialized.name);
    assert_eq!(tool_call.arguments, deserialized.arguments);
}

#[test]
fn test_mcp_tool_result_serialization() {
    let content = MCPContent::text("Issue retrieved successfully".to_string());

    let result = MCPToolResult {
        content: vec![content],
        is_error: Some(false),
    };

    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: MCPToolResult = serde_json::from_str(&serialized).unwrap();

    assert_eq!(result.content.len(), deserialized.content.len());
    assert_eq!(result.is_error, deserialized.is_error);
}

#[test]
fn test_mcp_content_text() {
    let content = MCPContent::text("Hello, world!".to_string());

    assert_eq!(content.content_type, "text");
    assert_eq!(content.text, "Hello, world!");
}

#[test]
fn test_mcp_content_serialization() {
    let content = MCPContent {
        content_type: "text".to_string(),
        text: "Test content".to_string(),
    };

    let serialized = serde_json::to_string(&content).unwrap();
    let deserialized: MCPContent = serde_json::from_str(&serialized).unwrap();

    assert_eq!(content.content_type, deserialized.content_type);
    assert_eq!(content.text, deserialized.text);
}

#[test]
fn test_list_tools_params_serialization() {
    let params = ListToolsParams {};

    let serialized = serde_json::to_string(&params).unwrap();
    let _deserialized: ListToolsParams = serde_json::from_str(&serialized).unwrap();

    // Empty struct should serialize/deserialize correctly - if we get here, it worked
}

#[test]
fn test_list_tools_result_serialization() {
    let tool = MCPTool {
        name: "get_issue".to_string(),
        description: "Get a Jira issue by key".to_string(),
        input_schema: json!({}),
    };

    let result = ListToolsResult { tools: vec![tool] };

    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: ListToolsResult = serde_json::from_str(&serialized).unwrap();

    assert_eq!(result.tools.len(), deserialized.tools.len());
    assert_eq!(result.tools[0].name, deserialized.tools[0].name);
}

#[test]
fn test_call_tool_params_serialization() {
    let params = CallToolParams {
        name: "get_issue".to_string(),
        arguments: Some(json!({
            "issue_key": "TEST-1"
        })),
    };

    let serialized = serde_json::to_string(&params).unwrap();
    let deserialized: CallToolParams = serde_json::from_str(&serialized).unwrap();

    assert_eq!(params.name, deserialized.name);
    assert_eq!(params.arguments, deserialized.arguments);
}

#[test]
fn test_call_tool_result_serialization() {
    let content = MCPContent::text("Tool executed successfully".to_string());

    let result = CallToolResult {
        content: vec![content],
        is_error: false,
    };

    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: CallToolResult = serde_json::from_str(&serialized).unwrap();

    assert_eq!(result.content.len(), deserialized.content.len());
    assert_eq!(result.is_error, deserialized.is_error);
}

#[test]
fn test_bulk_operation_type_serialization() {
    let operation_types = vec![
        BulkOperationType::Update,
        BulkOperationType::Transition,
        BulkOperationType::AddComment,
        BulkOperationType::Mixed,
    ];

    for operation_type in operation_types {
        let serialized = serde_json::to_string(&operation_type).unwrap();
        let deserialized: BulkOperationType = serde_json::from_str(&serialized).unwrap();

        // Compare by matching the variants since PartialEq is not implemented
        match (operation_type, deserialized) {
            (BulkOperationType::Update, BulkOperationType::Update) => {}
            (BulkOperationType::Transition, BulkOperationType::Transition) => {}
            (BulkOperationType::AddComment, BulkOperationType::AddComment) => {}
            (BulkOperationType::Mixed, BulkOperationType::Mixed) => {}
            _ => panic!("Serialization/deserialization mismatch"),
        }
    }
}

#[test]
fn test_bulk_operation_item_serialization() {
    let item = BulkOperationItem {
        issue_key: "TEST-1".to_string(),
        operation_type: BulkOperationType::Update,
        data: json!({"summary": "Updated issue"}),
    };

    let serialized = serde_json::to_string(&item).unwrap();
    let deserialized: BulkOperationItem = serde_json::from_str(&serialized).unwrap();

    assert_eq!(item.issue_key, deserialized.issue_key);
    assert_eq!(item.data, deserialized.data);

    // Compare operation types
    match (item.operation_type, deserialized.operation_type) {
        (BulkOperationType::Update, BulkOperationType::Update) => {}
        _ => panic!("Operation type mismatch"),
    }
}

#[test]
fn test_bulk_operation_result_serialization() {
    let result = BulkOperationResult {
        issue_key: "TEST-1".to_string(),
        success: true,
        error_message: None,
        operation_type: BulkOperationType::Update,
    };

    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: BulkOperationResult = serde_json::from_str(&serialized).unwrap();

    assert_eq!(result.issue_key, deserialized.issue_key);
    assert_eq!(result.success, deserialized.success);
    assert_eq!(result.error_message, deserialized.error_message);

    // Compare operation types
    match (result.operation_type, deserialized.operation_type) {
        (BulkOperationType::Update, BulkOperationType::Update) => {}
        _ => panic!("Operation type mismatch"),
    }
}

#[test]
fn test_bulk_operation_summary_serialization() {
    let mut summary = BulkOperationSummary::new();

    summary.add_result(BulkOperationResult {
        issue_key: "TEST-1".to_string(),
        success: true,
        error_message: None,
        operation_type: BulkOperationType::Update,
    });

    let serialized = serde_json::to_string(&summary).unwrap();
    let deserialized: BulkOperationSummary = serde_json::from_str(&serialized).unwrap();

    assert_eq!(summary.total_operations, deserialized.total_operations);
    assert_eq!(
        summary.successful_operations,
        deserialized.successful_operations
    );
    assert_eq!(summary.failed_operations, deserialized.failed_operations);
    assert_eq!(summary.results.len(), deserialized.results.len());
    assert_eq!(summary.duration_ms, deserialized.duration_ms);
}
