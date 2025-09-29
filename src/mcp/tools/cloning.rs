use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::mcp::{MCPContent, MCPToolResult};
use tracing::info;

/// Clone a Jira issue
pub struct CloneIssueTool {
    client: JiraClient,
}

impl CloneIssueTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for CloneIssueTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let original_issue_key = args
            .get("original_issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: original_issue_key"))?;

        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: project_key"))?;

        let issue_type = args
            .get("issue_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: issue_type"))?;

        let summary = args
            .get("summary")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: summary"))?;

        let description = args.get("description").and_then(|v| v.as_str());
        let copy_attachments = args.get("copy_attachments").and_then(|v| v.as_bool()).unwrap_or(false);
        let copy_work_logs = args.get("copy_work_logs").and_then(|v| v.as_bool()).unwrap_or(false);
        let copy_watchers = args.get("copy_watchers").and_then(|v| v.as_bool()).unwrap_or(false);
        let copy_links = args.get("copy_links").and_then(|v| v.as_bool()).unwrap_or(false);

        info!("Cloning issue {} to project {} as {}", original_issue_key, project_key, issue_type);

        let clone_request = crate::types::jira::JiraIssueCloneRequest {
            project_key: project_key.to_string(),
            issue_type: issue_type.to_string(),
            summary: summary.to_string(),
            description: description.map(|d| d.to_string()),
            field_mapping: None,
            copy_attachments: Some(copy_attachments),
            copy_comments: Some(true),
            copy_work_logs: Some(copy_work_logs),
            copy_watchers: Some(copy_watchers),
            copy_links: Some(copy_links),
        };

        let clone_response = self.client.clone_issue(original_issue_key, &clone_request).await?;

        let response_text = format!(
            "Issue cloned successfully\nOriginal: {}\nCloned: {}\nProject: {}\nType: {}\nSummary: {}",
            clone_response.original_issue_key,
            clone_response.cloned_issue_key,
            project_key,
            issue_type,
            summary
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

