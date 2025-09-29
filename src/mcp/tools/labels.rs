use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::mcp::{MCPContent, MCPToolResult};
use tracing::info;

/// Get all labels
pub struct GetLabelsTool {
    client: JiraClient,
}

impl GetLabelsTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetLabelsTool {
    async fn handle(&self, _args: serde_json::Value) -> Result<MCPToolResult> {
        info!("Getting all labels");

        let labels = self.client.get_labels().await?;

        let mut content = vec![MCPContent::text(format!("Found {} labels:\n\n", labels.len()))];

        for label in labels {
            let label_text = format!("• {}\n", label.name);
            content.push(MCPContent::text(label_text));
        }

        Ok(MCPToolResult {
            content,
            is_error: Some(false),
        })
    }
}

/// Create a new label
pub struct CreateLabelTool {
    client: JiraClient,
}

impl CreateLabelTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for CreateLabelTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: name"))?;

        info!("Creating label: {}", name);

        let label_request = crate::types::jira::JiraLabelCreateRequest {
            name: name.to_string(),
        };

        let created_label = self.client.create_label(&label_request).await?;

        let response_text = format!(
            "Label created successfully: {}",
            created_label.name
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Update a label
pub struct UpdateLabelTool {
    client: JiraClient,
}

impl UpdateLabelTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for UpdateLabelTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: name"))?;

        let new_name = args
            .get("new_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: new_name"))?;

        info!("Updating label: {} to {}", name, new_name);

        let update_request = crate::types::jira::JiraLabelUpdateRequest {
            name: new_name.to_string(),
        };

        self.client.update_label(name, &update_request).await?;

        let response_text = format!(
            "Label updated successfully: {} -> {}",
            name,
            new_name
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Delete a label
pub struct DeleteLabelTool {
    client: JiraClient,
}

impl DeleteLabelTool {
    #[must_use]
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for DeleteLabelTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: name"))?;

        info!("Deleting label: {}", name);

        self.client.delete_label(name).await?;

        let response_text = format!(
            "Label deleted successfully: {}",
            name
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

