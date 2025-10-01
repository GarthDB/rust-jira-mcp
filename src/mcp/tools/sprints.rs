use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::jira::JiraSprintCreateRequest;
use crate::types::mcp::{MCPContent, MCPToolResult};
use std::fmt::Write;
use tracing::info;

/// Get a sprint by ID
pub struct GetSprintTool {
    client: JiraClient,
}

impl GetSprintTool {
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
impl crate::mcp::server::MCPToolHandler for GetSprintTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let sprint_id = args
            .get("sprint_id")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: sprint_id")
            })?;

        info!("Getting sprint: {}", sprint_id);

        let sprint =
            self.client
                .get_sprint(i32::try_from(sprint_id).map_err(|_| {
                    crate::error::JiraError::api_error("Sprint ID too large for i32")
                })?)
                .await?;

        let response_text = format!(
            "Sprint: {}\nName: {}\nState: {:?}\nStart Date: {}\nEnd Date: {}\nGoal: {}\nURL: {}/secure/RapidBoard.jspa?rapidView={}&view=planning.nodetail&selectedIssue={}",
            sprint.id,
            sprint.name,
            sprint.state,
            sprint.start_date.as_deref().unwrap_or("Not set"),
            sprint.end_date.as_deref().unwrap_or("Not set"),
            sprint.goal.as_deref().unwrap_or("No goal set"),
            self.client.api_base_url().replace("/rest/api/2", ""),
            sprint.rapid_view_id.unwrap_or(0),
            sprint.id
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Create a new sprint
pub struct CreateSprintTool {
    client: JiraClient,
}

impl CreateSprintTool {
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
impl crate::mcp::server::MCPToolHandler for CreateSprintTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::JiraError::api_error("Missing required parameter: name")
        })?;

        let rapid_view_id = args
            .get("rapid_view_id")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: rapid_view_id")
            })?;

        let start_date = args.get("start_date").and_then(|v| v.as_str());
        let end_date = args.get("end_date").and_then(|v| v.as_str());
        let goal = args.get("goal").and_then(|v| v.as_str());

        info!("Creating sprint: {} for board {}", name, rapid_view_id);

        let sprint_request = JiraSprintCreateRequest {
            name: name.to_string(),
            rapid_view_id: i32::try_from(rapid_view_id).map_err(|_| {
                crate::error::JiraError::api_error("Rapid view ID too large for i32")
            })?,
            start_date: start_date.map(std::string::ToString::to_string),
            end_date: end_date.map(std::string::ToString::to_string),
            goal: goal.map(std::string::ToString::to_string),
        };

        let sprint = self.client.create_sprint(&sprint_request).await?;

        let response_text = format!(
            "Sprint created successfully!\nID: {}\nName: {}\nBoard ID: {}\nURL: {}/secure/RapidBoard.jspa?rapidView={}&view=planning.nodetail&selectedIssue={}",
            sprint.id,
            sprint.name,
            sprint.rapid_view_id,
            self.client.api_base_url().replace("/rest/api/2", ""),
            sprint.rapid_view_id,
            sprint.id
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Add issues to a sprint
pub struct AddIssuesToSprintTool {
    client: JiraClient,
}

impl AddIssuesToSprintTool {
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
impl crate::mcp::server::MCPToolHandler for AddIssuesToSprintTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let sprint_id = args
            .get("sprint_id")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: sprint_id")
            })?;

        let issues = args
            .get("issues")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issues")
            })?;

        let issue_keys: Vec<String> = issues
            .iter()
            .filter_map(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .collect();

        if issue_keys.is_empty() {
            return Err(crate::error::JiraError::api_error(
                "No valid issue keys provided",
            ));
        }

        info!("Adding {} issues to sprint {}", issue_keys.len(), sprint_id);

        let response = self
            .client
            .add_issues_to_sprint(
                i32::try_from(sprint_id).map_err(|_| {
                    crate::error::JiraError::api_error("Sprint ID too large for i32")
                })?,
                &issue_keys,
            )
            .await?;

        let mut response_text = format!(
            "Added {} issues to sprint {}\n",
            issue_keys.len(),
            sprint_id
        );

        if let Some(errors) = response.errors {
            if !errors.is_empty() {
                response_text.push_str("\nErrors encountered:\n");
                for error in errors {
                    writeln!(
                        response_text,
                        "- {}: {}",
                        error.issue_key, error.error_message
                    )
                    .unwrap();
                }
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Get issues in a sprint
pub struct GetSprintIssuesTool {
    client: JiraClient,
}

impl GetSprintIssuesTool {
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
impl crate::mcp::server::MCPToolHandler for GetSprintIssuesTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let sprint_id = args
            .get("sprint_id")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: sprint_id")
            })?;

        let start_at = args
            .get("start_at")
            .or_else(|| args.get("startAt"))
            .and_then(serde_json::Value::as_i64)
            .map(|v| i32::try_from(v).unwrap_or(0));

        let max_results = args
            .get("max_results")
            .or_else(|| args.get("maxResults"))
            .and_then(serde_json::Value::as_i64)
            .map(|v| i32::try_from(v).unwrap_or(0));

        info!("Getting issues for sprint: {}", sprint_id);

        let sprint_issues = self
            .client
            .get_sprint_issues(
                i32::try_from(sprint_id).map_err(|_| {
                    crate::error::JiraError::api_error("Sprint ID too large for i32")
                })?,
                start_at,
                max_results,
            )
            .await?;

        let response_text = format!(
            "Found {} issues in sprint {} (showing {} of {} total)\n\n",
            sprint_issues.issues.len(),
            sprint_id,
            sprint_issues.issues.len(),
            sprint_issues.total
        );

        let mut content = vec![MCPContent::text(response_text)];

        for issue in sprint_issues.issues {
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

            let issue_text = format!(
                "• {} - {}\n  Status: {}\n  Assignee: {}\n  URL: {}/browse/{}\n",
                issue.key,
                summary,
                status,
                assignee,
                self.client.api_base_url().replace("/rest/api/2", ""),
                issue.key
            );
            content.push(MCPContent::text(issue_text));
        }

        Ok(MCPToolResult {
            content,
            is_error: Some(false),
        })
    }
}

/// Start a sprint (set state to active)
pub struct StartSprintTool {
    client: JiraClient,
}

impl StartSprintTool {
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
impl crate::mcp::server::MCPToolHandler for StartSprintTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let sprint_id = args
            .get("sprint_id")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: sprint_id")
            })?;

        info!("Starting sprint: {}", sprint_id);

        let sprint =
            self.client
                .start_sprint(i32::try_from(sprint_id).map_err(|_| {
                    crate::error::JiraError::api_error("Sprint ID too large for i32")
                })?)
                .await?;

        let response_text = format!(
            "Sprint started successfully!\nID: {}\nName: {}\nState: {:?}\nURL: {}/secure/RapidBoard.jspa?rapidView={}&view=planning.nodetail&selectedIssue={}",
            sprint.id,
            sprint.name,
            sprint.state,
            self.client.api_base_url().replace("/rest/api/2", ""),
            sprint.rapid_view_id.unwrap_or(0),
            sprint.id
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Close a sprint (set state to closed)
pub struct CloseSprintTool {
    client: JiraClient,
}

impl CloseSprintTool {
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
impl crate::mcp::server::MCPToolHandler for CloseSprintTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let sprint_id = args
            .get("sprint_id")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: sprint_id")
            })?;

        info!("Closing sprint: {}", sprint_id);

        let sprint =
            self.client
                .close_sprint(i32::try_from(sprint_id).map_err(|_| {
                    crate::error::JiraError::api_error("Sprint ID too large for i32")
                })?)
                .await?;

        let response_text = format!(
            "Sprint closed successfully!\nID: {}\nName: {}\nState: {:?}\nURL: {}/secure/RapidBoard.jspa?rapidView={}&view=planning.nodetail&selectedIssue={}",
            sprint.id,
            sprint.name,
            sprint.state,
            self.client.api_base_url().replace("/rest/api/2", ""),
            sprint.rapid_view_id.unwrap_or(0),
            sprint.id
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Get sprints for a board
pub struct GetBoardSprintsTool {
    client: JiraClient,
}

impl GetBoardSprintsTool {
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
impl crate::mcp::server::MCPToolHandler for GetBoardSprintsTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let rapid_view_id = args
            .get("rapid_view_id")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: rapid_view_id")
            })?;

        info!("Getting sprints for board: {}", rapid_view_id);

        let sprints = self
            .client
            .get_board_sprints(i32::try_from(rapid_view_id).map_err(|_| {
                crate::error::JiraError::api_error("Rapid view ID too large for i32")
            })?)
            .await?;

        let response_text = format!(
            "Found {} sprints for board {}\n\n",
            sprints.len(),
            rapid_view_id
        );

        let mut content = vec![MCPContent::text(response_text)];

        for sprint in sprints {
            let sprint_text = format!(
                "• {} - {}\n  State: {:?}\n  Start: {}\n  End: {}\n  Goal: {}\n  URL: {}/secure/RapidBoard.jspa?rapidView={}&view=planning.nodetail&selectedIssue={}\n",
                sprint.id,
                sprint.name,
                sprint.state,
                sprint.start_date.as_deref().unwrap_or("Not set"),
                sprint.end_date.as_deref().unwrap_or("Not set"),
                sprint.goal.as_deref().unwrap_or("No goal set"),
                self.client.api_base_url().replace("/rest/api/2", ""),
                rapid_view_id,
                sprint.id
            );
            content.push(MCPContent::text(sprint_text));
        }

        Ok(MCPToolResult {
            content,
            is_error: Some(false),
        })
    }
}
