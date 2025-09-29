use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::mcp::{MCPContent, MCPToolResult};
use tracing::info;

/// Get comments for a Jira issue
pub struct GetCommentsTool {
    client: JiraClient,
}

impl GetCommentsTool {
    #[must_use]
    /// # Panics
    /// This function does not panic.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetCommentsTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: issue_key"))?;

        info!("Getting comments for issue: {}", issue_key);

        let comments = self.client.get_comments(issue_key).await?;

        let mut content = vec![MCPContent::text(format!("Found {} comments for issue {}\n\n", comments.len(), issue_key))];

        for comment in comments {
            let author = comment.author.display_name;
            let created = comment.created;
            let body = comment.body;

            let comment_text = format!(
                "• {} by {} on {}\n{}\n",
                comment.id,
                author,
                created,
                body
            );
            content.push(MCPContent::text(comment_text));
        }

        Ok(MCPToolResult {
            content,
            is_error: Some(false),
        })
    }
}

/// Add a comment to a Jira issue
pub struct AddCommentTool {
    client: JiraClient,
}

impl AddCommentTool {
    #[must_use]
    /// # Panics
    /// This function does not panic.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for AddCommentTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: issue_key"))?;

        let comment = args
            .get("comment")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: comment"))?;

        info!("Adding comment to issue: {}", issue_key);

        let created_comment = self.client.add_comment(issue_key, comment).await?;

        let response_text = format!(
            "Comment added successfully to issue {}\nComment ID: {}\nAuthor: {}\nCreated: {}",
            issue_key,
            created_comment.id,
            created_comment.author.display_name,
            created_comment.created
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

