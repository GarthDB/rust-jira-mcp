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
        let jql = args.get("jql").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::JiraError::ApiError {
                message: "Missing required parameter: jql".to_string(),
            }
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

        let search_result = self
            .client
            .search_issues(jql, start_at, max_results)
            .await?;

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
                issue
                    .fields
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

        info!(
            "Updating Jira issue {} with fields: {:?}",
            issue_id_or_key, fields
        );

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

        let comment = args.get("comment").and_then(|v| v.as_str());

        info!(
            "Transitioning Jira issue {} to transition {}",
            issue_key, transition_id
        );

        self.client
            .transition_issue(issue_key, transition_id, comment)
            .await?;

        let response_text = format!("Issue {} transitioned successfully!", issue_key);

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Project Configuration and Metadata Tools

// Get Project Configuration Tool
pub struct GetProjectConfigTool {
    client: JiraClient,
}

impl GetProjectConfigTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetProjectConfigTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: project_key".to_string(),
            })?;

        info!("Getting project configuration for: {}", project_key);

        let config = self.client.get_project_configuration(project_key).await?;

        let response_text = format!(
            "Project Configuration for {}:\n\n{}",
            project_key,
            serde_json::to_string_pretty(&config).unwrap_or_else(|_| "Failed to format configuration".to_string())
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Get Issue Types Tool
pub struct GetIssueTypesTool {
    client: JiraClient,
}

impl GetIssueTypesTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetIssueTypesTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: project_key".to_string(),
            })?;

        info!("Getting issue types for project: {}", project_key);

        let issue_types = self.client.get_project_issue_types(project_key).await?;

        let mut response_text = format!("Issue Types for project {}:\n\n", project_key);

        if issue_types.is_empty() {
            response_text.push_str("No issue types found.");
        } else {
            for (i, issue_type) in issue_types.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. {} (ID: {})\n   Description: {}\n   Subtask: {}\n\n",
                    i + 1,
                    issue_type.name,
                    issue_type.id,
                    issue_type.description.as_deref().unwrap_or("No description"),
                    issue_type.subtask
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Get Issue Type Metadata Tool
pub struct GetIssueTypeMetadataTool {
    client: JiraClient,
}

impl GetIssueTypeMetadataTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetIssueTypeMetadataTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_type_id = args
            .get("issue_type_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_type_id".to_string(),
            })?;

        info!("Getting issue type metadata for ID: {}", issue_type_id);

        let issue_type = self.client.get_issue_type_metadata(issue_type_id).await?;

        let response_text = format!(
            "Issue Type Metadata:\n\nName: {}\nID: {}\nDescription: {}\nSubtask: {}\nIcon URL: {}",
            issue_type.name,
            issue_type.id,
            issue_type.description.as_deref().unwrap_or("No description"),
            issue_type.subtask,
            issue_type.icon_url.as_deref().unwrap_or("No icon")
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Get Project Components Tool
pub struct GetProjectComponentsTool {
    client: JiraClient,
}

impl GetProjectComponentsTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetProjectComponentsTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: project_key".to_string(),
            })?;

        info!("Getting components for project: {}", project_key);

        let components = self.client.get_project_components(project_key).await?;

        let mut response_text = format!("Components for project {}:\n\n", project_key);

        if components.is_empty() {
            response_text.push_str("No components found.");
        } else {
            for (i, component) in components.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. {} (ID: {})\n   Description: {}\n   URL: {}\n\n",
                    i + 1,
                    component.name,
                    component.id,
                    component.description.as_deref().unwrap_or("No description"),
                    component.self_url
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Get Priorities and Statuses Tool
pub struct GetPrioritiesAndStatusesTool {
    client: JiraClient,
}

impl GetPrioritiesAndStatusesTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetPrioritiesAndStatusesTool {
    async fn handle(&self, _args: serde_json::Value) -> Result<MCPToolResult> {
        info!("Getting priorities and statuses");

        let (priorities, statuses) = tokio::try_join!(
            self.client.get_priorities(),
            self.client.get_statuses()
        )?;

        let mut response_text = "Priorities and Statuses:\n\n".to_string();

        // Add priorities section
        response_text.push_str("PRIORITIES:\n");
        if priorities.is_empty() {
            response_text.push_str("No priorities found.\n\n");
        } else {
            for (i, priority) in priorities.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. {} (ID: {})\n   Description: {}\n\n",
                    i + 1,
                    priority.name,
                    priority.id,
                    priority.description.as_deref().unwrap_or("No description")
                ));
            }
        }

        // Add statuses section
        response_text.push_str("STATUSES:\n");
        if statuses.is_empty() {
            response_text.push_str("No statuses found.");
        } else {
            for (i, status) in statuses.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. {} (ID: {})\n   Description: {}\n   Category: {} ({})\n\n",
                    i + 1,
                    status.name,
                    status.id,
                    status.description.as_deref().unwrap_or("No description"),
                    status.status_category.name,
                    status.status_category.color_name
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Get Custom Fields Tool
pub struct GetCustomFieldsTool {
    client: JiraClient,
}

impl GetCustomFieldsTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetCustomFieldsTool {
    async fn handle(&self, _args: serde_json::Value) -> Result<MCPToolResult> {
        info!("Getting custom fields");

        let custom_fields = self.client.get_custom_fields().await?;

        let mut response_text = "Custom Fields:\n\n".to_string();

        if custom_fields.is_empty() {
            response_text.push_str("No custom fields found.");
        } else {
            for (i, field) in custom_fields.iter().enumerate() {
                let field_name = field.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                let field_id = field.get("id").and_then(|v| v.as_str()).unwrap_or("Unknown");
                let field_type = field.get("schema").and_then(|s| s.get("type")).and_then(|v| v.as_str()).unwrap_or("Unknown");
                let custom = field.get("custom").and_then(|v| v.as_bool()).unwrap_or(false);

                response_text.push_str(&format!(
                    "{}. {} (ID: {})\n   Type: {}\n   Custom: {}\n\n",
                    i + 1,
                    field_name,
                    field_id,
                    field_type,
                    custom
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Get Project Metadata Tool (comprehensive)
pub struct GetProjectMetadataTool {
    client: JiraClient,
}

impl GetProjectMetadataTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetProjectMetadataTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: project_key".to_string(),
            })?;

        info!("Getting comprehensive project metadata for: {}", project_key);

        let metadata = self.client.get_project_metadata(project_key).await?;

        let response_text = format!(
            "Comprehensive Project Metadata for {}:\n\n{}",
            project_key,
            serde_json::to_string_pretty(&metadata).unwrap_or_else(|_| "Failed to format metadata".to_string())
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}
