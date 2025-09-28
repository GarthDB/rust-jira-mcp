use rust_jira_mcp::types::jira::{BulkOperationConfig, BulkOperationItem, BulkOperationType};
use serde_json::json;

/// Example demonstrating how to use the bulk operations functionality
///
/// This example shows how to:
/// 1. Configure bulk operations with custom settings
/// 2. Create bulk operation items for different operation types
/// 3. Use the bulk operations through the MCP tools
///
/// Note: This is a demonstration of the API structure. In practice, you would
/// use these through the MCP server tools.
fn main() {
    println!("Bulk Operations Example");
    println!("======================");

    // Example 1: Basic bulk update configuration
    let basic_config = BulkOperationConfig::default();
    println!("Basic config: {:?}", basic_config);

    // Example 2: Custom bulk operation configuration
    let custom_config = BulkOperationConfig {
        batch_size: Some(5),      // Process 5 issues per batch
        continue_on_error: true,  // Continue processing even if some operations fail
        rate_limit_ms: Some(200), // Wait 200ms between operations
        max_retries: Some(3),     // Retry failed operations up to 3 times
    };
    println!("Custom config: {:?}", custom_config);

    // Example 3: Create bulk update operations
    let update_operations = vec![
        BulkOperationItem {
            issue_key: "PROJ-123".to_string(),
            operation_type: BulkOperationType::Update,
            data: json!({
                "fields": {
                    "summary": "Updated via bulk operation",
                    "priority": {"name": "High"}
                }
            }),
        },
        BulkOperationItem {
            issue_key: "PROJ-124".to_string(),
            operation_type: BulkOperationType::Update,
            data: json!({
                "fields": {
                    "summary": "Another bulk update",
                    "assignee": {"name": "john.doe"}
                }
            }),
        },
    ];

    println!("Update operations: {} items", update_operations.len());

    // Example 4: Create bulk transition operations
    let transition_operations = vec![
        BulkOperationItem {
            issue_key: "PROJ-125".to_string(),
            operation_type: BulkOperationType::Transition,
            data: json!({
                "transition_id": "31",
                "comment": "Bulk transition to Done"
            }),
        },
        BulkOperationItem {
            issue_key: "PROJ-126".to_string(),
            operation_type: BulkOperationType::Transition,
            data: json!({
                "transition_id": "31",
                "comment": "Another bulk transition"
            }),
        },
    ];

    println!(
        "Transition operations: {} items",
        transition_operations.len()
    );

    // Example 5: Create bulk comment operations
    let comment_operations = vec![
        BulkOperationItem {
            issue_key: "PROJ-127".to_string(),
            operation_type: BulkOperationType::AddComment,
            data: json!({
                "comment_body": "This is a bulk comment added to multiple issues"
            }),
        },
        BulkOperationItem {
            issue_key: "PROJ-128".to_string(),
            operation_type: BulkOperationType::AddComment,
            data: json!({
                "comment_body": "Another bulk comment"
            }),
        },
    ];

    println!("Comment operations: {} items", comment_operations.len());

    // Example 6: Mixed bulk operations
    let mut mixed_operations = Vec::new();
    mixed_operations.extend(update_operations);
    mixed_operations.extend(transition_operations);
    mixed_operations.extend(comment_operations);

    println!("Mixed operations: {} total items", mixed_operations.len());

    // Example 7: MCP Tool Usage Examples
    println!("\nMCP Tool Usage Examples:");
    println!("========================");

    // Bulk Update Issues Tool
    let bulk_update_example = json!({
        "issue_keys": ["PROJ-123", "PROJ-124", "PROJ-125"],
        "fields": {
            "summary": "Bulk updated summary",
            "priority": {"name": "Medium"}
        },
        "config": {
            "batch_size": 3,
            "continue_on_error": true,
            "rate_limit_ms": 100,
            "max_retries": 2
        }
    });
    println!(
        "Bulk Update Tool Input: {}",
        serde_json::to_string_pretty(&bulk_update_example).unwrap()
    );

    // Bulk Transition Issues Tool
    let bulk_transition_example = json!({
        "issue_keys": ["PROJ-123", "PROJ-124"],
        "transition_id": "31",
        "comment": "Bulk transition to Done status",
        "config": {
            "batch_size": 2,
            "continue_on_error": true,
            "rate_limit_ms": 150
        }
    });
    println!(
        "Bulk Transition Tool Input: {}",
        serde_json::to_string_pretty(&bulk_transition_example).unwrap()
    );

    // Bulk Add Comments Tool
    let bulk_comments_example = json!({
        "issue_keys": ["PROJ-125", "PROJ-126", "PROJ-127"],
        "comment_body": "This comment was added to multiple issues via bulk operation",
        "config": {
            "batch_size": 5,
            "continue_on_error": false,
            "rate_limit_ms": 50
        }
    });
    println!(
        "Bulk Comments Tool Input: {}",
        serde_json::to_string_pretty(&bulk_comments_example).unwrap()
    );

    // Mixed Bulk Operations Tool
    let mixed_operations_example = json!({
        "operations": [
            {
                "issue_key": "PROJ-123",
                "operation_type": "update",
                "data": {
                    "fields": {
                        "summary": "Updated via mixed bulk operation"
                    }
                }
            },
            {
                "issue_key": "PROJ-124",
                "operation_type": "transition",
                "data": {
                    "transition_id": "31",
                    "comment": "Transitioned via mixed bulk operation"
                }
            },
            {
                "issue_key": "PROJ-125",
                "operation_type": "add_comment",
                "data": {
                    "comment_body": "Commented via mixed bulk operation"
                }
            }
        ],
        "config": {
            "batch_size": 3,
            "continue_on_error": true,
            "rate_limit_ms": 100,
            "max_retries": 3
        }
    });
    println!(
        "Mixed Operations Tool Input: {}",
        serde_json::to_string_pretty(&mixed_operations_example).unwrap()
    );

    println!("\nBulk Operations Features:");
    println!("========================");
    println!("✅ Process up to 100 issues per request");
    println!("✅ Configurable batch sizes (default: 10)");
    println!("✅ Continue-on-error mode (default: true)");
    println!("✅ Detailed success/failure reporting");
    println!("✅ Rate limiting to respect API limits (default: 100ms)");
    println!("✅ Progress tracking for large operations");
    println!("✅ Retry logic for failed operations (default: 3 retries)");
    println!("✅ Support for mixed operation types in single request");
    println!("✅ Comprehensive error handling and reporting");

    println!("\nExample completed successfully! 🎉");
}
