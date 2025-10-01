use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::mcp::{MCPContent, MCPToolResult};
use serde_json::json;
use tracing::info;

/// Search for Jira issues using JQL
pub struct SearchIssuesTool {
    client: JiraClient,
}

impl SearchIssuesTool {
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
impl crate::mcp::server::MCPToolHandler for SearchIssuesTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let jql = args
            .get("jql")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: jql"))?;

        let start_at = args
            .get("start_at")
            .or_else(|| args.get("startAt")) // Support both snake_case and camelCase
            .and_then(serde_json::Value::as_i64)
            .map(|v| i32::try_from(v).unwrap_or(0));

        let max_results = args
            .get("max_results")
            .or_else(|| args.get("maxResults")) // Support both snake_case and camelCase
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

        let mut content = vec![MCPContent::text(response_text)];

        for issue in search_result.issues {
            let issue_text = format!(
                "• {} - {} ({})\n",
                issue.key,
                issue
                    .fields
                    .get("summary")
                    .and_then(|s| s.as_str())
                    .unwrap_or("No summary"),
                issue
                    .fields
                    .get("status")
                    .and_then(|s| s.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("Unknown status")
            );
            content.push(MCPContent::text(issue_text));
        }

        Ok(MCPToolResult {
            content,
            is_error: Some(false),
        })
    }
}

/// Create a new Jira issue
pub struct CreateIssueTool {
    client: JiraClient,
}

impl CreateIssueTool {
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
impl crate::mcp::server::MCPToolHandler for CreateIssueTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: project_key")
            })?;

        let issue_type = args
            .get("issue_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issue_type")
            })?;

        let summary = args
            .get("summary")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: summary")
            })?;

        let description = args.get("description").and_then(|v| v.as_str());

        info!("Creating Jira issue: {} - {}", project_key, summary);

        let issue_data = json!({
            "fields": {
                "project": {"key": project_key},
                "summary": summary,
                "issuetype": {"name": issue_type},
                "description": description.unwrap_or("")
            }
        });

        let created_issue = self.client.create_issue(&issue_data).await?;

        let response_text = format!(
            "Issue created successfully: {} - {}\nURL: {}/browse/{}",
            created_issue.key,
            summary,
            self.client.api_base_url().replace("/rest/api/2", ""),
            created_issue.key
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Update an existing Jira issue
pub struct UpdateIssueTool {
    client: JiraClient,
}

impl UpdateIssueTool {
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
impl crate::mcp::server::MCPToolHandler for UpdateIssueTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issue_key")
            })?;

        info!("Updating Jira issue: {}", issue_key);

        let mut fields = serde_json::Map::new();

        if let Some(summary) = args.get("summary").and_then(|v| v.as_str()) {
            fields.insert("summary".to_string(), json!(summary));
        }

        if let Some(description) = args.get("description").and_then(|v| v.as_str()) {
            fields.insert("description".to_string(), json!(description));
        }

        if let Some(assignee) = args.get("assignee").and_then(|v| v.as_str()) {
            fields.insert("assignee".to_string(), json!({"name": assignee}));
        }

        if fields.is_empty() {
            return Err(crate::error::JiraError::api_error("No fields to update"));
        }

        let update_data = json!({
            "fields": fields
        });

        self.client.update_issue(issue_key, &update_data).await?;

        let response_text = format!("Issue {issue_key} updated successfully");

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Get a specific Jira issue
pub struct GetIssueTool {
    client: JiraClient,
}

impl GetIssueTool {
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
impl crate::mcp::server::MCPToolHandler for GetIssueTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issue_key")
            })?;

        info!("Getting Jira issue: {}", issue_key);

        let issue = self.client.get_issue(issue_key).await?;

        let summary = issue
            .fields
            .get("summary")
            .and_then(|s| s.as_str())
            .unwrap_or("No summary");
        let status = issue
            .fields
            .get("status")
            .and_then(|s| s.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("Unknown status");
        let assignee = issue
            .fields
            .get("assignee")
            .and_then(|a| a.get("displayName"))
            .and_then(|n| n.as_str())
            .unwrap_or("Unassigned");

        let response_text = format!(
            "Issue: {}\nSummary: {}\nStatus: {}\nAssignee: {}\nURL: {}/browse/{}",
            issue.key,
            summary,
            status,
            assignee,
            self.client.api_base_url().replace("/rest/api/2", ""),
            issue.key
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Link two Jira issues
pub struct LinkIssuesTool {
    client: JiraClient,
}

impl LinkIssuesTool {
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
impl crate::mcp::server::MCPToolHandler for LinkIssuesTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let inward_issue_key = args
            .get("inward_issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: inward_issue_key")
            })?;

        let outward_issue_key = args
            .get("outward_issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: outward_issue_key")
            })?;

        let link_type_name = args
            .get("link_type_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: link_type_name")
            })?;

        let comment = args.get("comment").and_then(|v| v.as_str());

        info!(
            "Linking issues: {} {} {}",
            inward_issue_key, link_type_name, outward_issue_key
        );

        self.client
            .link_issues(inward_issue_key, outward_issue_key, link_type_name, comment)
            .await?;

        let response_text = format!(
            "Successfully linked {} {} {}{}",
            inward_issue_key,
            link_type_name,
            outward_issue_key,
            if comment.is_some() {
                " with comment"
            } else {
                ""
            }
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}
