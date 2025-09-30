use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::mcp::{MCPContent, MCPToolResult};
use tracing::info;

/// Get work logs for a Jira issue
pub struct GetIssueWorkLogsTool {
    client: JiraClient,
}

impl GetIssueWorkLogsTool {
    #[must_use]
    /// # Panics
    ///
    /// Panics if the `JiraClient` cannot be created from the provided configuration.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetIssueWorkLogsTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issue_key")
            })?;

        info!("Getting work logs for issue: {}", issue_key);

        let work_logs = self.client.get_issue_work_logs(issue_key).await?;

        let mut content = vec![MCPContent::text(format!(
            "Found {} work logs for issue {}\n\n",
            work_logs.len(),
            issue_key
        ))];

        for work_log in work_logs {
            let work_log_text = format!(
                "• {} - {} by {} on {}\n",
                work_log.id, work_log.time_spent, work_log.author.display_name, work_log.created
            );
            content.push(MCPContent::text(work_log_text));
        }

        Ok(MCPToolResult {
            content,
            is_error: Some(false),
        })
    }
}

/// Add a work log to a Jira issue
pub struct AddWorkLogTool {
    client: JiraClient,
}

impl AddWorkLogTool {
    #[must_use]
    /// # Panics
    ///
    /// Panics if the `JiraClient` cannot be created from the provided configuration.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for AddWorkLogTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issue_key")
            })?;

        let time_spent = args
            .get("time_spent")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: time_spent")
            })?;

        let comment = args.get("comment").and_then(|v| v.as_str());
        let started = args.get("started").and_then(|v| v.as_str());

        info!("Adding work log to issue: {}", issue_key);

        let work_log_request = crate::types::jira::JiraWorkLogCreateRequest {
            time_spent: time_spent.to_string(),
            comment: comment.map(ToString::to_string),
            started: started.map(ToString::to_string),
            visibility: None,
        };

        let created_work_log = self
            .client
            .add_work_log(issue_key, &work_log_request)
            .await?;

        let response_text = format!(
            "Work log added successfully to issue {}\nWork log ID: {}\nTime spent: {}\nAuthor: {}",
            issue_key,
            created_work_log.id,
            created_work_log.time_spent,
            created_work_log.author.display_name
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Update a work log
pub struct UpdateWorkLogTool {
    client: JiraClient,
}

impl UpdateWorkLogTool {
    #[must_use]
    /// # Panics
    ///
    /// Panics if the `JiraClient` cannot be created from the provided configuration.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for UpdateWorkLogTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issue_key")
            })?;

        let work_log_id = args
            .get("work_log_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: work_log_id")
            })?;

        let time_spent = args
            .get("time_spent")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: time_spent")
            })?;

        let comment = args.get("comment").and_then(|v| v.as_str());
        let started = args.get("started").and_then(|v| v.as_str());

        info!("Updating work log {} for issue: {}", work_log_id, issue_key);

        let update_request = crate::types::jira::JiraWorkLogUpdateRequest {
            time_spent: Some(time_spent.to_string()),
            comment: comment.map(ToString::to_string),
            started: started.map(ToString::to_string),
            visibility: None,
        };

        self.client
            .update_work_log(issue_key, work_log_id, &update_request)
            .await?;

        let response_text =
            format!("Work log {work_log_id} updated successfully for issue {issue_key}");

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Delete a work log
pub struct DeleteWorkLogTool {
    client: JiraClient,
}

impl DeleteWorkLogTool {
    #[must_use]
    /// # Panics
    ///
    /// Panics if the `JiraClient` cannot be created from the provided configuration.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for DeleteWorkLogTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issue_key")
            })?;

        let work_log_id = args
            .get("work_log_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: work_log_id")
            })?;

        info!("Deleting work log {} for issue: {}", work_log_id, issue_key);

        self.client.delete_work_log(issue_key, work_log_id).await?;

        let response_text =
            format!("Work log {work_log_id} deleted successfully from issue {issue_key}");

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}
