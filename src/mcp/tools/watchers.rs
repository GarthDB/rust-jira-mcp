use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::mcp::{MCPContent, MCPToolResult};
use tracing::info;

/// Get watchers for a Jira issue
pub struct GetIssueWatchersTool {
    client: JiraClient,
}

impl GetIssueWatchersTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetIssueWatchersTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: issue_key"))?;

        info!("Getting watchers for issue: {}", issue_key);

        let watchers = self.client.get_issue_watchers(issue_key).await?;

        let response_text = format!(
            "Issue {} has {} watchers\n",
            issue_key,
            watchers.watch_count
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Add a watcher to a Jira issue
pub struct AddIssueWatcherTool {
    client: JiraClient,
}

impl AddIssueWatcherTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for AddIssueWatcherTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: issue_key"))?;

        let account_id = args
            .get("account_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: account_id"))?;

        info!("Adding watcher {} to issue: {}", account_id, issue_key);

        self.client.add_issue_watcher(issue_key, account_id).await?;

        let response_text = format!(
            "Watcher {} added successfully to issue {}",
            account_id,
            issue_key
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Remove a watcher from a Jira issue
pub struct RemoveIssueWatcherTool {
    client: JiraClient,
}

impl RemoveIssueWatcherTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for RemoveIssueWatcherTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: issue_key"))?;

        let account_id = args
            .get("account_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: account_id"))?;

        info!("Removing watcher {} from issue: {}", account_id, issue_key);

        self.client.remove_issue_watcher(issue_key, account_id).await?;

        let response_text = format!(
            "Watcher {} removed successfully from issue {}",
            account_id,
            issue_key
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

