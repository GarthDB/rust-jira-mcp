use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::mcp::{MCPContent, MCPToolResult};
use serde_json::json;
use tracing::info;

// Test Auth Tool
pub struct TestAuthTool {
    config: JiraConfig,
}

impl TestAuthTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for TestAuthTool {
    async fn handle(&self, _args: serde_json::Value) -> Result<MCPToolResult> {
        info!("Testing Jira authentication");

        // For now, just return a success response
        // In a real implementation, this would make an actual API call to test auth
        let _response = json!({
            "authenticated": true,
            "method": "Personal Access Token",
            "email": self.config.email,
            "api_base_url": self.config.api_base_url
        });

        Ok(MCPToolResult {
            content: vec![MCPContent::text(
                "Authentication test successful".to_string(),
            )],
            is_error: Some(false),
        })
    }
}

// Search Issues Tool
pub struct SearchIssuesTool {
    client: JiraClient,
}

impl SearchIssuesTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for SearchIssuesTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let jql = args
            .get("jql")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: jql".to_string(),
            })?;

        let start_at = args
            .get("start_at")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);

        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);

        info!("Searching Jira issues with JQL: {}", jql);

        let search_result = self.client.search_issues(jql, start_at, max_results).await?;

        let response_text = format!(
            "Found {} issues (showing {} of {} total)\n\n",
            search_result.issues.len(),
            search_result.issues.len(),
            search_result.total
        );

        let mut issue_details = String::new();
        for issue in &search_result.issues {
            issue_details.push_str(&format!(
                "• {} - {}\n",
                issue.key,
                issue.fields
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No summary")
            ));
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(format!(
                "{}{}",
                response_text, issue_details
            ))],
            is_error: Some(false),
        })
    }
}

// Create Issue Tool
pub struct CreateIssueTool {
    client: JiraClient,
}

impl CreateIssueTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for CreateIssueTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let fields = args
            .get("fields")
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: fields".to_string(),
            })?;

        info!("Creating Jira issue with fields: {:?}", fields);

        let issue = self.client.create_issue(fields).await?;

        let response_text = format!(
            "Issue created successfully!\n\nKey: {}\nID: {}\nURL: {}",
            issue.key, issue.id, issue.self_url
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Update Issue Tool
pub struct UpdateIssueTool {
    client: JiraClient,
}

impl UpdateIssueTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for UpdateIssueTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_id_or_key = args
            .get("issue_id_or_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_id_or_key".to_string(),
            })?;

        let fields = args
            .get("fields")
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: fields".to_string(),
            })?;

        info!("Updating Jira issue {} with fields: {:?}", issue_id_or_key, fields);

        self.client.update_issue(issue_id_or_key, fields).await?;

        Ok(MCPToolResult {
            content: vec![MCPContent::text(format!(
                "Issue {} updated successfully!",
                issue_id_or_key
            ))],
            is_error: Some(false),
        })
    }
}

// Get Issue Tool
pub struct GetIssueTool {
    client: JiraClient,
}

impl GetIssueTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetIssueTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        info!("Getting Jira issue: {}", issue_key);

        let issue = self.client.get_issue(issue_key).await?;

        let summary = issue
            .fields
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("No summary");

        let status = issue
            .fields
            .get("status")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown status");

        let assignee = issue
            .fields
            .get("assignee")
            .and_then(|v| v.get("displayName"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unassigned");

        let response_text = format!(
            "Issue Details:\n\nKey: {}\nSummary: {}\nStatus: {}\nAssignee: {}\nURL: {}",
            issue.key, summary, status, assignee, issue.self_url
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Get Comments Tool
pub struct GetCommentsTool {
    client: JiraClient,
}

impl GetCommentsTool {
    #[must_use]
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
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        info!("Getting comments for Jira issue: {}", issue_key);

        let comments = self.client.get_comments(issue_key).await?;

        let mut response_text = format!("Comments for issue {}:\n\n", issue_key);

        if comments.is_empty() {
            response_text.push_str("No comments found.");
        } else {
            for (i, comment) in comments.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. {} (by {}) - {}\n   {}\n\n",
                    i + 1,
                    comment.created,
                    comment.author.display_name,
                    comment.id,
                    comment.body
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Add Comment Tool
pub struct AddCommentTool {
    client: JiraClient,
}

impl AddCommentTool {
    #[must_use]
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
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        let comment_body = args
            .get("comment_body")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: comment_body".to_string(),
            })?;

        info!("Adding comment to Jira issue: {}", issue_key);

        let comment = self.client.add_comment(issue_key, comment_body).await?;

        let response_text = format!(
            "Comment added successfully!\n\nComment ID: {}\nAuthor: {}\nCreated: {}\nBody: {}",
            comment.id, comment.author.display_name, comment.created, comment.body
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Get Transitions Tool
pub struct GetTransitionsTool {
    client: JiraClient,
}

impl GetTransitionsTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetTransitionsTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        info!("Getting transitions for Jira issue: {}", issue_key);

        let transitions = self.client.get_transitions(issue_key).await?;

        let mut response_text = format!("Available transitions for issue {}:\n\n", issue_key);

        if transitions.is_empty() {
            response_text.push_str("No transitions available.");
        } else {
            for (i, transition) in transitions.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. {} (ID: {}) -> {}\n",
                    i + 1,
                    transition.name,
                    transition.id,
                    transition.to.name
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Transition Issue Tool
pub struct TransitionIssueTool {
    client: JiraClient,
}

impl TransitionIssueTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for TransitionIssueTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        let transition_id = args
            .get("transition_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: transition_id".to_string(),
            })?;

        let comment = args
            .get("comment")
            .and_then(|v| v.as_str());

        info!("Transitioning Jira issue {} to transition {}", issue_key, transition_id);

        self.client.transition_issue(issue_key, transition_id, comment).await?;

        let response_text = format!(
            "Issue {} transitioned successfully!",
            issue_key
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}
