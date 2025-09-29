use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::mcp::{MCPContent, MCPToolResult};
use tracing::info;

/// Bulk update issues
pub struct BulkUpdateIssuesTool {
    client: JiraClient,
}

impl BulkUpdateIssuesTool {
    #[must_use]
    /// # Panics
    /// This function panics if `JiraClient::new` fails.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for BulkUpdateIssuesTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_keys = args
            .get("issue_keys")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issue_keys")
            })?;

        let update_data = args.get("update_data").ok_or_else(|| {
            crate::error::JiraError::api_error("Missing required parameter: update_data")
        })?;

        let config = args
            .get("config")
            .map(|c| serde_json::from_value(c.clone()).unwrap_or_default())
            .unwrap_or_default();

        info!("Bulk updating {} issues", issue_keys.len());

        let issue_keys_vec: Vec<String> = issue_keys
            .iter()
            .filter_map(|v| v.as_str().map(ToString::to_string))
            .collect();

        self.client
            .bulk_update_issues(issue_keys_vec, update_data.clone(), Some(config))
            .await?;

        let response_text = format!(
            "Bulk update completed successfully for {} issues",
            issue_keys.len()
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Bulk transition issues
pub struct BulkTransitionIssuesTool {
    client: JiraClient,
}

impl BulkTransitionIssuesTool {
    #[must_use]
    /// # Panics
    /// This function panics if `JiraClient::new` fails.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for BulkTransitionIssuesTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_keys = args
            .get("issue_keys")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issue_keys")
            })?;

        let transition_id = args
            .get("transition_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: transition_id")
            })?;

        let comment = args.get("comment").and_then(|v| v.as_str());
        let config = args
            .get("config")
            .map(|c| serde_json::from_value(c.clone()).unwrap_or_default())
            .unwrap_or_default();

        info!(
            "Bulk transitioning {} issues to {}",
            issue_keys.len(),
            transition_id
        );

        let issue_keys_vec: Vec<String> = issue_keys
            .iter()
            .filter_map(|v| v.as_str().map(ToString::to_string))
            .collect();

        self.client
            .bulk_transition_issues(
                issue_keys_vec,
                transition_id.to_string(),
                comment.map(ToString::to_string),
                Some(config),
            )
            .await?;

        let response_text = format!(
            "Bulk transition completed successfully for {} issues to transition {}",
            issue_keys.len(),
            transition_id
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Bulk add comments
pub struct BulkAddCommentsTool {
    client: JiraClient,
}

impl BulkAddCommentsTool {
    #[must_use]
    /// # Panics
    /// This function panics if `JiraClient::new` fails.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for BulkAddCommentsTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_keys = args
            .get("issue_keys")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issue_keys")
            })?;

        let comment = args
            .get("comment")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: comment")
            })?;

        let config = args
            .get("config")
            .map(|c| serde_json::from_value(c.clone()).unwrap_or_default())
            .unwrap_or_default();

        info!("Bulk adding comments to {} issues", issue_keys.len());

        let issue_keys_vec: Vec<String> = issue_keys
            .iter()
            .filter_map(|v| v.as_str().map(ToString::to_string))
            .collect();

        self.client
            .bulk_add_comments(issue_keys_vec, comment.to_string(), Some(config))
            .await?;

        let response_text = format!(
            "Bulk comment addition completed successfully for {} issues",
            issue_keys.len()
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Mixed bulk operations
pub struct MixedBulkOperationsTool {
    client: JiraClient,
}

impl MixedBulkOperationsTool {
    #[must_use]
    /// # Panics
    /// This function panics if `JiraClient::new` fails.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for MixedBulkOperationsTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let operations = args
            .get("operations")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: operations")
            })?;

        let config = args
            .get("config")
            .map(|c| serde_json::from_value(c.clone()).unwrap_or_default())
            .unwrap_or_default();

        info!("Executing {} mixed bulk operations", operations.len());

        let operations_vec: Vec<crate::types::jira::BulkOperationItem> = operations
            .iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect();

        let results = self
            .client
            .execute_bulk_operations(operations_vec, config)
            .await?;

        let response_text = format!(
            "Mixed bulk operations completed successfully\nTotal operations: {}\nSuccessful: {}\nFailed: {}",
            results.total_operations,
            results.successful_operations,
            results.failed_operations
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}
