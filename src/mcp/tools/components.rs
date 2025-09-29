use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::mcp::{MCPContent, MCPToolResult};
use tracing::info;

/// Create a new component
pub struct CreateComponentTool {
    client: JiraClient,
}

impl CreateComponentTool {
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
impl crate::mcp::server::MCPToolHandler for CreateComponentTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::JiraError::api_error("Missing required parameter: name")
        })?;

        let project = args
            .get("project")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: project")
            })?;

        let description = args.get("description").and_then(|v| v.as_str());
        let lead_account_id = args.get("lead_account_id").and_then(|v| v.as_str());

        info!("Creating component: {} for project: {}", name, project);

        let component_request = crate::types::jira::JiraComponentCreateRequest {
            name: name.to_string(),
            description: description.map(ToString::to_string),
            project: project.to_string(),
            lead_account_id: lead_account_id.map(ToString::to_string),
            assignee_type: Some("PROJECT_LEAD".to_string()),
        };

        let created_component = self.client.create_component(&component_request).await?;

        let response_text = format!(
            "Component created successfully: {} for project {}",
            created_component.name, project
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Update a component
pub struct UpdateComponentTool {
    client: JiraClient,
}

impl UpdateComponentTool {
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
impl crate::mcp::server::MCPToolHandler for UpdateComponentTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let component_id = args
            .get("component_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: component_id")
            })?;

        let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::JiraError::api_error("Missing required parameter: name")
        })?;

        let description = args.get("description").and_then(|v| v.as_str());
        let lead_account_id = args.get("lead_account_id").and_then(|v| v.as_str());

        info!("Updating component: {}", component_id);

        let update_request = crate::types::jira::JiraComponentUpdateRequest {
            name: Some(name.to_string()),
            description: description.map(ToString::to_string),
            lead_account_id: lead_account_id.map(ToString::to_string),
            assignee_type: Some("PROJECT_LEAD".to_string()),
        };

        self.client
            .update_component(component_id, &update_request)
            .await?;

        let response_text = format!("Component updated successfully: {component_id}");

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Delete a component
pub struct DeleteComponentTool {
    client: JiraClient,
}

impl DeleteComponentTool {
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
impl crate::mcp::server::MCPToolHandler for DeleteComponentTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let component_id = args
            .get("component_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: component_id")
            })?;

        info!("Deleting component: {}", component_id);

        self.client.delete_component(component_id).await?;

        let response_text = format!("Component deleted successfully: {component_id}");

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}
