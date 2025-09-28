use rust_jira_mcp::types::jira::{
    BulkOperationConfig, BulkOperationItem, BulkOperationSummary, BulkOperationType,
};
use serde_json::json;

#[tokio::test]
async fn test_bulk_operation_summary() {
    let mut summary = BulkOperationSummary::new();

    // Test initial state
    assert_eq!(summary.total_operations, 0);
    assert_eq!(summary.successful_operations, 0);
    assert_eq!(summary.failed_operations, 0);
    assert!((summary.success_rate() - 0.0).abs() < f64::EPSILON);

    // Add some results
    summary.add_result(rust_jira_mcp::types::jira::BulkOperationResult {
        issue_key: "TEST-1".to_string(),
        success: true,
        error_message: None,
        operation_type: BulkOperationType::Update,
    });

    summary.add_result(rust_jira_mcp::types::jira::BulkOperationResult {
        issue_key: "TEST-2".to_string(),
        success: false,
        error_message: Some("Test error".to_string()),
        operation_type: BulkOperationType::Transition,
    });

    summary.add_result(rust_jira_mcp::types::jira::BulkOperationResult {
        issue_key: "TEST-3".to_string(),
        success: true,
        error_message: None,
        operation_type: BulkOperationType::AddComment,
    });

    // Test final state
    assert_eq!(summary.total_operations, 3);
    assert_eq!(summary.successful_operations, 2);
    assert_eq!(summary.failed_operations, 1);
    assert!((summary.success_rate() - 66.666_666_666_666_66).abs() < f64::EPSILON);
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
fn test_bulk_operation_item_creation() {
    let item = BulkOperationItem {
        issue_key: "TEST-123".to_string(),
        operation_type: BulkOperationType::Update,
        data: json!({
            "fields": {
                "summary": "Test issue"
            }
        }),
    };

    assert_eq!(item.issue_key, "TEST-123");
    assert!(matches!(item.operation_type, BulkOperationType::Update));
    assert!(item.data.get("fields").is_some());
}

#[test]
fn test_bulk_operation_types() {
    // Test all operation types
    let update = BulkOperationType::Update;
    let transition = BulkOperationType::Transition;
    let add_comment = BulkOperationType::AddComment;
    let mixed = BulkOperationType::Mixed;

    // These should compile without issues
    match update {
        BulkOperationType::Update => {}
        _ => unreachable!(),
    }

    match transition {
        BulkOperationType::Transition => {}
        _ => unreachable!(),
    }

    match add_comment {
        BulkOperationType::AddComment => {}
        _ => unreachable!(),
    }

    match mixed {
        BulkOperationType::Mixed => {}
        _ => unreachable!(),
    }
}

#[test]
fn test_bulk_operation_config_serialization() {
    let config = BulkOperationConfig {
        batch_size: Some(5),
        continue_on_error: false,
        rate_limit_ms: Some(200),
        max_retries: Some(5),
    };

    // Test serialization
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("batch_size"));
    assert!(json.contains("continue_on_error"));
    assert!(json.contains("rate_limit_ms"));
    assert!(json.contains("max_retries"));

    // Test deserialization
    let deserialized: BulkOperationConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.batch_size, config.batch_size);
    assert_eq!(deserialized.continue_on_error, config.continue_on_error);
    assert_eq!(deserialized.rate_limit_ms, config.rate_limit_ms);
    assert_eq!(deserialized.max_retries, config.max_retries);
}
