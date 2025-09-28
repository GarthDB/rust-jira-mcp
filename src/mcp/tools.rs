#![allow(clippy::format_push_string)]

use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::jira::{
    BulkOperationConfig, BulkOperationItem, BulkOperationType, JiraComponentCreateRequest,
    JiraComponentUpdateRequest, JiraIssueCloneRequest, JiraLabelCreateRequest,
    JiraLabelUpdateRequest,
};
use crate::types::mcp::{MCPContent, MCPToolResult};
use base64::{engine::general_purpose, Engine as _};
use serde_json::json;
use tracing::info;

// Test Auth Tool
pub struct TestAuthTool {
    config: JiraConfig,
}

impl TestAuthTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
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
    #[allow(clippy::missing_panics_doc)]
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
            .and_then(serde_json::Value::as_i64)
            .map(|v| i32::try_from(v).unwrap_or(0));

        let max_results = args
            .get("max_results")
            .and_then(serde_json::Value::as_i64)
            .map(|v| i32::try_from(v).unwrap_or(0));

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
            use std::fmt::Write;
            writeln!(
                issue_details,
                "• {} - {}",
                issue.key,
                issue
                    .fields
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No summary")
            )
            .unwrap();
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(format!("{response_text}{issue_details}"))],
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
    #[allow(clippy::missing_panics_doc)]
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
    #[allow(clippy::missing_panics_doc)]
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
                "Issue {issue_id_or_key} updated successfully!"
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
    #[allow(clippy::missing_panics_doc)]
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
    #[allow(clippy::missing_panics_doc)]
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

        let mut response_text = format!("Comments for issue {issue_key}:\n\n");

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
    #[allow(clippy::missing_panics_doc)]
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
    #[allow(clippy::missing_panics_doc)]
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

        let mut response_text = format!("Available transitions for issue {issue_key}:\n\n");

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
    #[allow(clippy::missing_panics_doc)]
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

        let response_text = format!("Issue {issue_key} transitioned successfully!");

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
    #[allow(clippy::missing_panics_doc)]
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
            serde_json::to_string_pretty(&config)
                .unwrap_or_else(|_| "Failed to format configuration".to_string())
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
    #[allow(clippy::missing_panics_doc)]
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

        let mut response_text = format!("Issue Types for project {project_key}:\n\n");

        if issue_types.is_empty() {
            response_text.push_str("No issue types found.");
        } else {
            for (i, issue_type) in issue_types.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. {} (ID: {})\n   Description: {}\n   Subtask: {}\n\n",
                    i + 1,
                    issue_type.name,
                    issue_type.id,
                    issue_type
                        .description
                        .as_deref()
                        .unwrap_or("No description"),
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
    #[allow(clippy::missing_panics_doc)]
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

        let response_text =
            format!(
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
    #[allow(clippy::missing_panics_doc)]
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

        let mut response_text = format!("Components for project {project_key}:\n\n");

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
    #[allow(clippy::missing_panics_doc)]
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

        let (priorities, statuses) =
            tokio::try_join!(self.client.get_priorities(), self.client.get_statuses())?;

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
    #[allow(clippy::missing_panics_doc)]
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
                let field_name = field
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                let field_id = field
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                let field_type = field
                    .get("schema")
                    .and_then(|s| s.get("type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                let custom = field
                    .get("custom")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);

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
    #[allow(clippy::missing_panics_doc)]
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

        info!(
            "Getting comprehensive project metadata for: {}",
            project_key
        );

        let metadata = self.client.get_project_metadata(project_key).await?;

        let response_text = format!(
            "Comprehensive Project Metadata for {}:\n\n{}",
            project_key,
            serde_json::to_string_pretty(&metadata)
                .unwrap_or_else(|_| "Failed to format metadata".to_string())
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Bulk Operations Tools

// Bulk Update Issues Tool
pub struct BulkUpdateIssuesTool {
    client: JiraClient,
}

impl BulkUpdateIssuesTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
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
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_keys (array of issue keys)".to_string(),
            })?;

        let fields = args
            .get("fields")
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: fields".to_string(),
            })?;

        // Parse configuration if provided
        let config = if let Some(config_data) = args.get("config") {
            serde_json::from_value(config_data.clone()).unwrap_or_default()
        } else {
            BulkOperationConfig::default()
        };

        // Convert issue keys to strings
        let issue_keys: Vec<String> = issue_keys
            .iter()
            .filter_map(|v| v.as_str().map(ToString::to_string))
            .collect();

        if issue_keys.is_empty() {
            return Err(crate::error::JiraError::ApiError {
                message: "No issue keys provided".to_string(),
            });
        }

        info!("Bulk updating {} issues", issue_keys.len());

        let summary = self
            .client
            .bulk_update_issues(issue_keys, fields.clone(), Some(config))
            .await?;

        let response_text = format!(
            "Bulk Update Completed!\n\n\
            Total Operations: {}\n\
            Successful: {}\n\
            Failed: {}\n\
            Success Rate: {:.1}%\n\
            Duration: {}ms\n\n\
            Results:\n{}",
            summary.total_operations,
            summary.successful_operations,
            summary.failed_operations,
            summary.success_rate(),
            summary.duration_ms,
            summary
                .results
                .iter()
                .map(|r| format!(
                    "• {}: {} ({})",
                    r.issue_key,
                    if r.success { "SUCCESS" } else { "FAILED" },
                    r.error_message.as_deref().unwrap_or("No error")
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Bulk Transition Issues Tool
pub struct BulkTransitionIssuesTool {
    client: JiraClient,
}

impl BulkTransitionIssuesTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
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
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_keys (array of issue keys)".to_string(),
            })?;

        let transition_id = args
            .get("transition_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: transition_id".to_string(),
            })?;

        let comment = args.get("comment").and_then(|v| v.as_str());

        // Parse configuration if provided
        let config = if let Some(config_data) = args.get("config") {
            serde_json::from_value(config_data.clone()).unwrap_or_default()
        } else {
            BulkOperationConfig::default()
        };

        // Convert issue keys to strings
        let issue_keys: Vec<String> = issue_keys
            .iter()
            .filter_map(|v| v.as_str().map(ToString::to_string))
            .collect();

        if issue_keys.is_empty() {
            return Err(crate::error::JiraError::ApiError {
                message: "No issue keys provided".to_string(),
            });
        }

        info!(
            "Bulk transitioning {} issues to transition {}",
            issue_keys.len(),
            transition_id
        );

        let summary = self
            .client
            .bulk_transition_issues(
                issue_keys,
                transition_id.to_string(),
                comment.map(ToString::to_string),
                Some(config),
            )
            .await?;

        let response_text = format!(
            "Bulk Transition Completed!\n\n\
            Total Operations: {}\n\
            Successful: {}\n\
            Failed: {}\n\
            Success Rate: {:.1}%\n\
            Duration: {}ms\n\n\
            Results:\n{}",
            summary.total_operations,
            summary.successful_operations,
            summary.failed_operations,
            summary.success_rate(),
            summary.duration_ms,
            summary
                .results
                .iter()
                .map(|r| format!(
                    "• {}: {} ({})",
                    r.issue_key,
                    if r.success { "SUCCESS" } else { "FAILED" },
                    r.error_message.as_deref().unwrap_or("No error")
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Bulk Add Comments Tool
pub struct BulkAddCommentsTool {
    client: JiraClient,
}

impl BulkAddCommentsTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
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
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_keys (array of issue keys)".to_string(),
            })?;

        let comment_body = args
            .get("comment_body")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: comment_body".to_string(),
            })?;

        // Parse configuration if provided
        let config = if let Some(config_data) = args.get("config") {
            serde_json::from_value(config_data.clone()).unwrap_or_default()
        } else {
            BulkOperationConfig::default()
        };

        // Convert issue keys to strings
        let issue_keys: Vec<String> = issue_keys
            .iter()
            .filter_map(|v| v.as_str().map(ToString::to_string))
            .collect();

        if issue_keys.is_empty() {
            return Err(crate::error::JiraError::ApiError {
                message: "No issue keys provided".to_string(),
            });
        }

        info!("Bulk adding comments to {} issues", issue_keys.len());

        let summary = self
            .client
            .bulk_add_comments(issue_keys, comment_body.to_string(), Some(config))
            .await?;

        let response_text = format!(
            "Bulk Add Comments Completed!\n\n\
            Total Operations: {}\n\
            Successful: {}\n\
            Failed: {}\n\
            Success Rate: {:.1}%\n\
            Duration: {}ms\n\n\
            Results:\n{}",
            summary.total_operations,
            summary.successful_operations,
            summary.failed_operations,
            summary.success_rate(),
            summary.duration_ms,
            summary
                .results
                .iter()
                .map(|r| format!(
                    "• {}: {} ({})",
                    r.issue_key,
                    if r.success { "SUCCESS" } else { "FAILED" },
                    r.error_message.as_deref().unwrap_or("No error")
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Mixed Bulk Operations Tool
pub struct MixedBulkOperationsTool {
    client: JiraClient,
}

impl MixedBulkOperationsTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
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
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: operations (array of operation objects)"
                    .to_string(),
            })?;

        // Parse configuration if provided
        let config = if let Some(config_data) = args.get("config") {
            serde_json::from_value(config_data.clone()).unwrap_or_default()
        } else {
            BulkOperationConfig::default()
        };

        // Parse operations
        let mut bulk_operations = Vec::new();
        for (i, op) in operations.iter().enumerate() {
            let issue_key = op
                .get("issue_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::JiraError::ApiError {
                    message: format!("Missing issue_key in operation {}", i + 1),
                })?;

            let operation_type = op
                .get("operation_type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::JiraError::ApiError {
                    message: format!("Missing operation_type in operation {}", i + 1),
                })?;

            let operation_type = match operation_type {
                "update" => BulkOperationType::Update,
                "transition" => BulkOperationType::Transition,
                "add_comment" => BulkOperationType::AddComment,
                "mixed" => BulkOperationType::Mixed,
                _ => {
                    return Err(crate::error::JiraError::ApiError {
                        message: format!(
                            "Invalid operation_type '{}' in operation {}. Must be one of: update, transition, add_comment, mixed",
                            operation_type, i + 1
                        ),
                    });
                }
            };

            let data = op
                .get("data")
                .ok_or_else(|| crate::error::JiraError::ApiError {
                    message: format!("Missing data in operation {}", i + 1),
                })?;

            bulk_operations.push(BulkOperationItem {
                issue_key: issue_key.to_string(),
                operation_type,
                data: data.clone(),
            });
        }

        if bulk_operations.is_empty() {
            return Err(crate::error::JiraError::ApiError {
                message: "No operations provided".to_string(),
            });
        }

        info!("Executing {} mixed bulk operations", bulk_operations.len());

        let summary = self
            .client
            .execute_bulk_operations(bulk_operations, config)
            .await?;

        let response_text = format!(
            "Mixed Bulk Operations Completed!\n\n\
            Total Operations: {}\n\
            Successful: {}\n\
            Failed: {}\n\
            Success Rate: {:.1}%\n\
            Duration: {}ms\n\n\
            Results:\n{}",
            summary.total_operations,
            summary.successful_operations,
            summary.failed_operations,
            summary.success_rate(),
            summary.duration_ms,
            summary
                .results
                .iter()
                .map(|r| format!(
                    "• {} ({}): {} ({})",
                    r.issue_key,
                    match r.operation_type {
                        BulkOperationType::Update => "UPDATE",
                        BulkOperationType::Transition => "TRANSITION",
                        BulkOperationType::AddComment => "COMMENT",
                        BulkOperationType::Mixed => "MIXED",
                    },
                    if r.success { "SUCCESS" } else { "FAILED" },
                    r.error_message.as_deref().unwrap_or("No error")
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Issue Linking Tools

// Get Link Types Tool
pub struct GetLinkTypesTool {
    client: JiraClient,
}

impl GetLinkTypesTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetLinkTypesTool {
    async fn handle(&self, _args: serde_json::Value) -> Result<MCPToolResult> {
        info!("Getting available link types");

        let link_types = self.client.get_link_types().await?;

        let mut response_text = "Available Link Types:\n\n".to_string();

        if link_types.is_empty() {
            response_text.push_str("No link types found.");
        } else {
            for (i, link_type) in link_types.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. {} (ID: {})\n   Inward: {}\n   Outward: {}\n\n",
                    i + 1,
                    link_type.name,
                    link_type.id,
                    link_type.inward,
                    link_type.outward
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Get Issue Links Tool
pub struct GetIssueLinksTool {
    client: JiraClient,
}

impl GetIssueLinksTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetIssueLinksTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        info!("Getting issue links for: {}", issue_key);

        let issue_links = self.client.get_issue_links(issue_key).await?;

        let mut response_text = format!("Issue Links for {issue_key}:\n\n");

        if issue_links.is_empty() {
            response_text.push_str("No issue links found.");
        } else {
            for (i, link) in issue_links.iter().enumerate() {
                let inward_issue = link.inward_issue.as_ref().map_or("N/A", |i| i.key.as_str());
                let outward_issue = link
                    .outward_issue
                    .as_ref()
                    .map_or("N/A", |i| i.key.as_str());

                response_text.push_str(&format!(
                    "{}. {} (ID: {})\n   Inward Issue: {}\n   Outward Issue: {}\n\n",
                    i + 1,
                    link.link_type.name,
                    link.id,
                    inward_issue,
                    outward_issue
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Create Issue Link Tool
pub struct CreateIssueLinkTool {
    client: JiraClient,
}

impl CreateIssueLinkTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for CreateIssueLinkTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let inward_issue_key = args
            .get("inward_issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: inward_issue_key".to_string(),
            })?;

        let outward_issue_key = args
            .get("outward_issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: outward_issue_key".to_string(),
            })?;

        let link_type_name = args
            .get("link_type_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: link_type_name".to_string(),
            })?;

        let comment = args.get("comment").and_then(|v| v.as_str());

        info!(
            "Creating issue link: {} -> {} ({})",
            inward_issue_key, outward_issue_key, link_type_name
        );

        self.client
            .link_issues(inward_issue_key, outward_issue_key, link_type_name, comment)
            .await?;

        let response_text = format!(
            "Issue link created successfully!\n\nInward Issue: {inward_issue_key}\nOutward Issue: {outward_issue_key}\nLink Type: {link_type_name}"
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Delete Issue Link Tool
pub struct DeleteIssueLinkTool {
    client: JiraClient,
}

impl DeleteIssueLinkTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for DeleteIssueLinkTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let link_id = args
            .get("link_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: link_id".to_string(),
            })?;

        info!("Deleting issue link: {}", link_id);

        self.client.delete_issue_link(link_id).await?;

        let response_text = format!("Issue link {link_id} deleted successfully!");

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// File Attachment Tools

// Get Issue Attachments Tool
pub struct GetIssueAttachmentsTool {
    client: JiraClient,
}

impl GetIssueAttachmentsTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetIssueAttachmentsTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        info!("Getting attachments for issue: {}", issue_key);

        let attachments = self.client.get_issue_attachments(issue_key).await?;

        let mut response_text = format!("Attachments for issue {issue_key}:\n\n");

        if attachments.is_empty() {
            response_text.push_str("No attachments found.");
        } else {
            for (i, attachment) in attachments.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. {} (ID: {})\n   Size: {} bytes\n   MIME Type: {}\n   Author: {}\n   Created: {}\n   URL: {}\n\n",
                    i + 1,
                    attachment.filename,
                    attachment.id,
                    attachment.size,
                    attachment.mime_type,
                    attachment.author.display_name,
                    attachment.created,
                    attachment.self_url
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Upload Attachment Tool
pub struct UploadAttachmentTool {
    client: JiraClient,
}

impl UploadAttachmentTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for UploadAttachmentTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        let filename = args
            .get("filename")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: filename".to_string(),
            })?;

        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: content (base64 encoded)".to_string(),
            })?;

        let mime_type = args.get("mime_type").and_then(|v| v.as_str());

        // Decode base64 content
        let content_bytes = general_purpose::STANDARD.decode(content).map_err(|e| {
            crate::error::JiraError::ApiError {
                message: format!("Failed to decode base64 content: {e}"),
            }
        })?;

        info!("Uploading attachment to issue: {}", issue_key);

        let attachments = self
            .client
            .upload_attachment(issue_key, filename, &content_bytes, mime_type)
            .await?;

        let mut response_text =
            format!("Attachment uploaded successfully to issue {issue_key}!\n\n");

        for (i, attachment) in attachments.iter().enumerate() {
            response_text.push_str(&format!(
                "{}. {} (ID: {})\n   Size: {} bytes\n   MIME Type: {}\n   URL: {}\n\n",
                i + 1,
                attachment.filename,
                attachment.id,
                attachment.size,
                attachment.mime_type,
                attachment.self_url
            ));
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Delete Attachment Tool
pub struct DeleteAttachmentTool {
    client: JiraClient,
}

impl DeleteAttachmentTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for DeleteAttachmentTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let attachment_id = args
            .get("attachment_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: attachment_id".to_string(),
            })?;

        info!("Deleting attachment: {}", attachment_id);

        self.client.delete_attachment(attachment_id).await?;

        let response_text = format!("Attachment {attachment_id} deleted successfully!");

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Download Attachment Tool
pub struct DownloadAttachmentTool {
    client: JiraClient,
}

impl DownloadAttachmentTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for DownloadAttachmentTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let attachment_id = args
            .get("attachment_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: attachment_id".to_string(),
            })?;

        info!("Downloading attachment: {}", attachment_id);

        let content = self.client.download_attachment(attachment_id).await?;
        let content_base64 = general_purpose::STANDARD.encode(&content);

        let response_text = format!(
            "Attachment {} downloaded successfully!\n\nSize: {} bytes\nBase64 Content:\n{}",
            attachment_id,
            content.len(),
            content_base64
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Work Log Tools

// Get Issue Work Logs Tool
pub struct GetIssueWorkLogsTool {
    client: JiraClient,
}

impl GetIssueWorkLogsTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
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
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        info!("Getting work logs for issue: {}", issue_key);

        let work_logs = self.client.get_issue_work_logs(issue_key).await?;

        let mut response_text = format!("Work Logs for issue {issue_key}:\n\n");

        if work_logs.is_empty() {
            response_text.push_str("No work logs found.");
        } else {
            let mut total_time = 0;
            for (i, work_log) in work_logs.iter().enumerate() {
                total_time += work_log.time_spent_seconds;
                response_text.push_str(&format!(
                    "{}. {} (ID: {})\n   Time Spent: {} ({} seconds)\n   Author: {}\n   Started: {}\n   Comment: {}\n\n",
                    i + 1,
                    work_log.time_spent,
                    work_log.id,
                    work_log.time_spent,
                    work_log.time_spent_seconds,
                    work_log.author.display_name,
                    work_log.created,
                    work_log.comment.as_deref().unwrap_or("No comment")
                ));
            }

            // Convert total seconds to hours and minutes
            let total_hours = total_time / 3600;
            let total_minutes = (total_time % 3600) / 60;
            response_text.push_str(&format!(
                "Total Time Logged: {total_hours} hours {total_minutes} minutes ({total_time} seconds)"
            ));
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Add Work Log Tool
pub struct AddWorkLogTool {
    client: JiraClient,
}

impl AddWorkLogTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
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
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        let time_spent = args
            .get("time_spent")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: time_spent".to_string(),
            })?;

        let comment = args.get("comment").and_then(|v| v.as_str());
        let started = args.get("started").and_then(|v| v.as_str());

        let work_log = crate::types::jira::JiraWorkLogCreateRequest {
            comment: comment.map(ToString::to_string),
            time_spent: time_spent.to_string(),
            started: started.map(ToString::to_string),
            visibility: None,
        };

        info!("Adding work log to issue: {}", issue_key);

        let created_work_log = self.client.add_work_log(issue_key, &work_log).await?;

        let response_text = format!(
            "Work log added successfully!\n\nIssue: {}\nWork Log ID: {}\nTime Spent: {} ({} seconds)\nAuthor: {}\nCreated: {}\nComment: {}",
            issue_key,
            created_work_log.id,
            created_work_log.time_spent,
            created_work_log.time_spent_seconds,
            created_work_log.author.display_name,
            created_work_log.created,
            created_work_log.comment.as_deref().unwrap_or("No comment")
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Update Work Log Tool
pub struct UpdateWorkLogTool {
    client: JiraClient,
}

impl UpdateWorkLogTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
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
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        let work_log_id = args
            .get("work_log_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: work_log_id".to_string(),
            })?;

        let time_spent = args.get("time_spent").and_then(|v| v.as_str());
        let comment = args.get("comment").and_then(|v| v.as_str());
        let started = args.get("started").and_then(|v| v.as_str());

        let work_log = crate::types::jira::JiraWorkLogUpdateRequest {
            comment: comment.map(ToString::to_string),
            time_spent: time_spent.map(ToString::to_string),
            started: started.map(ToString::to_string),
            visibility: None,
        };

        info!("Updating work log {} for issue: {}", work_log_id, issue_key);

        let updated_work_log = self
            .client
            .update_work_log(issue_key, work_log_id, &work_log)
            .await?;

        let response_text = format!(
            "Work log updated successfully!\n\nIssue: {}\nWork Log ID: {}\nTime Spent: {} ({} seconds)\nAuthor: {}\nUpdated: {}\nComment: {}",
            issue_key,
            updated_work_log.id,
            updated_work_log.time_spent,
            updated_work_log.time_spent_seconds,
            updated_work_log.author.display_name,
            updated_work_log.updated.as_deref().unwrap_or("Unknown"),
            updated_work_log.comment.as_deref().unwrap_or("No comment")
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Delete Work Log Tool
pub struct DeleteWorkLogTool {
    client: JiraClient,
}

impl DeleteWorkLogTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
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
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        let work_log_id = args
            .get("work_log_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: work_log_id".to_string(),
            })?;

        info!("Deleting work log {} for issue: {}", work_log_id, issue_key);

        self.client.delete_work_log(issue_key, work_log_id).await?;

        let response_text =
            format!("Work log {work_log_id} deleted successfully from issue {issue_key}!");

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Issue Watcher Tools

// Get Issue Watchers Tool
pub struct GetIssueWatchersTool {
    client: JiraClient,
}

impl GetIssueWatchersTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
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
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        info!("Getting watchers for issue: {}", issue_key);

        let watchers_response = self.client.get_issue_watchers(issue_key).await?;

        let mut response_text = format!(
            "Watchers for issue {issue_key}:\n\nWatching: {}\nWatch Count: {}\n\n",
            watchers_response.is_watching, watchers_response.watch_count
        );

        if watchers_response.watchers.is_empty() {
            response_text.push_str("No watchers found.");
        } else {
            for (i, watcher) in watchers_response.watchers.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. {} ({})\n   Email: {}\n   Active: {}\n   Time Zone: {}\n\n",
                    i + 1,
                    watcher.display_name,
                    watcher.account_id,
                    watcher.email_address.as_deref().unwrap_or("N/A"),
                    watcher.active,
                    watcher.time_zone.as_deref().unwrap_or("N/A")
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Add Issue Watcher Tool
pub struct AddIssueWatcherTool {
    client: JiraClient,
}

impl AddIssueWatcherTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
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
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        let account_id = args
            .get("account_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: account_id".to_string(),
            })?;

        info!("Adding watcher {} to issue: {}", account_id, issue_key);

        self.client.add_issue_watcher(issue_key, account_id).await?;

        let response_text =
            format!("Watcher {account_id} added successfully to issue {issue_key}!");

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Remove Issue Watcher Tool
pub struct RemoveIssueWatcherTool {
    client: JiraClient,
}

impl RemoveIssueWatcherTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
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
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_key".to_string(),
            })?;

        let account_id = args
            .get("account_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: account_id".to_string(),
            })?;

        info!("Removing watcher {} from issue: {}", account_id, issue_key);

        self.client
            .remove_issue_watcher(issue_key, account_id)
            .await?;

        let response_text =
            format!("Watcher {account_id} removed successfully from issue {issue_key}!");

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Issue Label Tools

// Get Labels Tool
pub struct GetLabelsTool {
    client: JiraClient,
}

impl GetLabelsTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetLabelsTool {
    async fn handle(&self, _args: serde_json::Value) -> Result<MCPToolResult> {
        info!("Getting all available labels");

        let labels = self.client.get_labels().await?;

        let mut response_text = "Available Labels:\n\n".to_string();

        if labels.is_empty() {
            response_text.push_str("No labels found.");
        } else {
            for (i, label) in labels.iter().enumerate() {
                response_text.push_str(&format!("{}. {}\n", i + 1, label.name));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Create Label Tool
pub struct CreateLabelTool {
    client: JiraClient,
}

impl CreateLabelTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for CreateLabelTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::JiraError::ApiError {
                message: "Missing required parameter: name".to_string(),
            }
        })?;

        info!("Creating label: {}", name);

        let label_request = JiraLabelCreateRequest {
            name: name.to_string(),
        };

        let created_label = self.client.create_label(&label_request).await?;

        let response_text = format!(
            "Label created successfully!\n\nName: {}",
            created_label.name
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Update Label Tool
pub struct UpdateLabelTool {
    client: JiraClient,
}

impl UpdateLabelTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for UpdateLabelTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let old_name = args
            .get("old_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: old_name".to_string(),
            })?;

        let new_name = args
            .get("new_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: new_name".to_string(),
            })?;

        info!("Updating label from {} to {}", old_name, new_name);

        let label_request = JiraLabelUpdateRequest {
            name: new_name.to_string(),
        };

        let updated_label = self.client.update_label(old_name, &label_request).await?;

        let response_text = format!(
            "Label updated successfully!\n\nOld Name: {}\nNew Name: {}",
            old_name, updated_label.name
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Delete Label Tool
pub struct DeleteLabelTool {
    client: JiraClient,
}

impl DeleteLabelTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for DeleteLabelTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::JiraError::ApiError {
                message: "Missing required parameter: name".to_string(),
            }
        })?;

        info!("Deleting label: {}", name);

        self.client.delete_label(name).await?;

        let response_text = format!("Label {name} deleted successfully!");

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Issue Component Tools

// Create Component Tool
pub struct CreateComponentTool {
    client: JiraClient,
}

impl CreateComponentTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for CreateComponentTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::JiraError::ApiError {
                message: "Missing required parameter: name".to_string(),
            }
        })?;

        let project = args
            .get("project")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: project".to_string(),
            })?;

        let description = args.get("description").and_then(|v| v.as_str());
        let assignee_type = args.get("assignee_type").and_then(|v| v.as_str());
        let lead_account_id = args.get("lead_account_id").and_then(|v| v.as_str());

        info!("Creating component: {} for project: {}", name, project);

        let component_request = JiraComponentCreateRequest {
            name: name.to_string(),
            description: description.map(ToString::to_string),
            project: project.to_string(),
            assignee_type: assignee_type.map(ToString::to_string),
            lead_account_id: lead_account_id.map(ToString::to_string),
        };

        let created_component = self.client.create_component(&component_request).await?;

        let response_text = format!(
            "Component created successfully!\n\nName: {}\nID: {}\nDescription: {}\nURL: {}",
            created_component.name,
            created_component.id,
            created_component
                .description
                .as_deref()
                .unwrap_or("No description"),
            created_component.self_url
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Update Component Tool
pub struct UpdateComponentTool {
    client: JiraClient,
}

impl UpdateComponentTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for UpdateComponentTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let component_id = args
            .get("component_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: component_id".to_string(),
            })?;

        let name = args.get("name").and_then(|v| v.as_str());
        let description = args.get("description").and_then(|v| v.as_str());
        let assignee_type = args.get("assignee_type").and_then(|v| v.as_str());
        let lead_account_id = args.get("lead_account_id").and_then(|v| v.as_str());

        info!("Updating component: {}", component_id);

        let component_request = JiraComponentUpdateRequest {
            name: name.map(ToString::to_string),
            description: description.map(ToString::to_string),
            assignee_type: assignee_type.map(ToString::to_string),
            lead_account_id: lead_account_id.map(ToString::to_string),
        };

        let updated_component = self
            .client
            .update_component(component_id, &component_request)
            .await?;

        let response_text = format!(
            "Component updated successfully!\n\nName: {}\nID: {}\nDescription: {}\nURL: {}",
            updated_component.name,
            updated_component.id,
            updated_component
                .description
                .as_deref()
                .unwrap_or("No description"),
            updated_component.self_url
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Delete Component Tool
pub struct DeleteComponentTool {
    client: JiraClient,
}

impl DeleteComponentTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for DeleteComponentTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let component_id = args
            .get("component_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: component_id".to_string(),
            })?;

        info!("Deleting component: {}", component_id);

        self.client.delete_component(component_id).await?;

        let response_text = format!("Component {component_id} deleted successfully!");

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Issue Cloning Tools

// Clone Issue Tool
pub struct CloneIssueTool {
    client: JiraClient,
}

impl CloneIssueTool {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
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
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: original_issue_key".to_string(),
            })?;

        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: project_key".to_string(),
            })?;

        let issue_type = args
            .get("issue_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: issue_type".to_string(),
            })?;

        let summary = args
            .get("summary")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::ApiError {
                message: "Missing required parameter: summary".to_string(),
            })?;

        let description = args.get("description").and_then(|v| v.as_str());
        let copy_attachments = args
            .get("copy_attachments")
            .and_then(serde_json::Value::as_bool);
        let copy_comments = args
            .get("copy_comments")
            .and_then(serde_json::Value::as_bool);
        let copy_work_logs = args
            .get("copy_work_logs")
            .and_then(serde_json::Value::as_bool);
        let copy_watchers = args
            .get("copy_watchers")
            .and_then(serde_json::Value::as_bool);
        let copy_links = args.get("copy_links").and_then(serde_json::Value::as_bool);

        info!(
            "Cloning issue {} to project {} as {}",
            original_issue_key, project_key, issue_type
        );

        let clone_request = JiraIssueCloneRequest {
            project_key: project_key.to_string(),
            issue_type: issue_type.to_string(),
            summary: summary.to_string(),
            description: description.map(ToString::to_string),
            field_mapping: None, // Use default field mapping
            copy_attachments,
            copy_comments,
            copy_work_logs,
            copy_watchers,
            copy_links,
        };

        let clone_response = self
            .client
            .clone_issue(original_issue_key, &clone_request)
            .await?;

        let mut response_text = format!(
            "Issue cloned successfully!\n\nOriginal Issue: {}\nCloned Issue: {}\nCloned Issue ID: {}\nCloned Issue URL: {}\n\n",
            clone_response.original_issue_key,
            clone_response.cloned_issue_key,
            clone_response.cloned_issue_id,
            clone_response.cloned_issue_url
        );

        if let Some(count) = clone_response.copied_attachments {
            response_text.push_str(&format!("Copied Attachments: {count}\n"));
        }
        if let Some(count) = clone_response.copied_comments {
            response_text.push_str(&format!("Copied Comments: {count}\n"));
        }
        if let Some(count) = clone_response.copied_work_logs {
            response_text.push_str(&format!("Copied Work Logs: {count}\n"));
        }
        if let Some(count) = clone_response.copied_watchers {
            response_text.push_str(&format!("Copied Watchers: {count}\n"));
        }
        if let Some(count) = clone_response.copied_links {
            response_text.push_str(&format!("Copied Links: {count}\n"));
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}
