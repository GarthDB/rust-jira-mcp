use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::mcp::{MCPContent, MCPToolResult};
use tracing::info;

/// Get project configuration details
pub struct GetProjectConfigTool {
    client: JiraClient,
}

impl GetProjectConfigTool {
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
impl crate::mcp::server::MCPToolHandler for GetProjectConfigTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: project_key"))?;

        info!("Getting project configuration for: {}", project_key);

        let config = self.client.get_project_configuration(project_key).await?;

        let response_text = format!(
            "Project configuration for {}:\n{}",
            project_key,
            serde_json::to_string_pretty(&config).unwrap_or_else(|_| "Failed to format configuration".to_string())
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Get comprehensive project metadata
pub struct GetProjectMetadataTool {
    client: JiraClient,
}

impl GetProjectMetadataTool {
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
impl crate::mcp::server::MCPToolHandler for GetProjectMetadataTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: project_key"))?;

        info!("Getting project metadata for: {}", project_key);

        let metadata = self.client.get_project_metadata(project_key).await?;

        let response_text = format!(
            "Project metadata for {}:\n{}",
            project_key,
            serde_json::to_string_pretty(&metadata).unwrap_or_else(|_| "Failed to format metadata".to_string())
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Get priorities and statuses
pub struct GetPrioritiesAndStatusesTool {
    client: JiraClient,
}

impl GetPrioritiesAndStatusesTool {
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
impl crate::mcp::server::MCPToolHandler for GetPrioritiesAndStatusesTool {
    async fn handle(&self, _args: serde_json::Value) -> Result<MCPToolResult> {
        info!("Getting priorities and statuses");

        let priorities = self.client.get_priorities().await?;
        let statuses = self.client.get_statuses().await?;

        let mut content = vec![MCPContent::text("Priorities:\n".to_string())];

        for priority in priorities {
            let priority_text = format!("• {} - {}\n", priority.name, priority.description.as_deref().unwrap_or("No description"));
            content.push(MCPContent::text(priority_text));
        }

        content.push(MCPContent::text("\nStatuses:\n".to_string()));

        for status in statuses {
            let status_text = format!("• {} - {}\n", status.name, status.description.as_deref().unwrap_or("No description"));
            content.push(MCPContent::text(status_text));
        }

        Ok(MCPToolResult {
            content,
            is_error: Some(false),
        })
    }
}

/// Get custom fields
pub struct GetCustomFieldsTool {
    client: JiraClient,
}

impl GetCustomFieldsTool {
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
impl crate::mcp::server::MCPToolHandler for GetCustomFieldsTool {
    async fn handle(&self, _args: serde_json::Value) -> Result<MCPToolResult> {
        info!("Getting custom fields");

        let custom_fields = self.client.get_custom_fields().await?;

        let mut content = vec![MCPContent::text(format!("Found {} custom fields:\n\n", custom_fields.len()))];

        for field in custom_fields {
            let field_name = field.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
            let field_id = field.get("id").and_then(|i| i.as_str()).unwrap_or("Unknown");
            let field_type = field.get("schema").and_then(|s| s.get("type")).and_then(|t| t.as_str()).unwrap_or("Unknown");

            let field_text = format!("• {} ({}): {}\n", field_name, field_id, field_type);
            content.push(MCPContent::text(field_text));
        }

        Ok(MCPToolResult {
            content,
            is_error: Some(false),
        })
    }
}

/// Get issue type metadata
pub struct GetIssueTypeMetadataTool {
    client: JiraClient,
}

impl GetIssueTypeMetadataTool {
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
impl crate::mcp::server::MCPToolHandler for GetIssueTypeMetadataTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_type_id = args
            .get("issue_type_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: issue_type_id"))?;

        info!("Getting issue type metadata for: {}", issue_type_id);

        let issue_type = self.client.get_issue_type_metadata(issue_type_id).await?;

        let response_text = format!(
            "Issue type metadata for {}:\n{}",
            issue_type_id,
            serde_json::to_string_pretty(&issue_type).unwrap_or_else(|_| "Failed to format issue type".to_string())
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Get project components
pub struct GetProjectComponentsTool {
    client: JiraClient,
}

impl GetProjectComponentsTool {
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
impl crate::mcp::server::MCPToolHandler for GetProjectComponentsTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: project_key"))?;

        info!("Getting project components for: {}", project_key);

        let components = self.client.get_project_components(project_key).await?;

        let mut content = vec![MCPContent::text(format!("Found {} components for project {}:\n\n", components.len(), project_key))];

        for component in components {
            let component_text = format!("• {} - {}\n", component.name, component.description.unwrap_or_default());
            content.push(MCPContent::text(component_text));
        }

        Ok(MCPToolResult {
            content,
            is_error: Some(false),
        })
    }
}

/// Get issue types for a project
pub struct GetIssueTypesTool {
    client: JiraClient,
}

impl GetIssueTypesTool {
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
impl crate::mcp::server::MCPToolHandler for GetIssueTypesTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: project_key"))?;

        info!("Getting issue types for project: {}", project_key);

        let issue_types = self.client.get_project_issue_types(project_key).await?;

        let mut content = vec![MCPContent::text(format!("Found {} issue types for project {}:\n\n", issue_types.len(), project_key))];

        for issue_type in issue_types {
            let issue_type_text = format!("• {} - {}\n", issue_type.name, issue_type.description.unwrap_or_default());
            content.push(MCPContent::text(issue_type_text));
        }

        Ok(MCPToolResult {
            content,
            is_error: Some(false),
        })
    }
}

/// Get link types
pub struct GetLinkTypesTool {
    client: JiraClient,
}

impl GetLinkTypesTool {
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
impl crate::mcp::server::MCPToolHandler for GetLinkTypesTool {
    async fn handle(&self, _args: serde_json::Value) -> Result<MCPToolResult> {
        info!("Getting link types");

        let link_types = self.client.get_link_types().await?;

        let mut content = vec![MCPContent::text(format!("Found {} link types:\n\n", link_types.len()))];

        for link_type in link_types {
            let link_type_text = format!("• {} - {}\n", link_type.name, link_type.inward);
            content.push(MCPContent::text(link_type_text));
        }

        Ok(MCPToolResult {
            content,
            is_error: Some(false),
        })
    }
}
