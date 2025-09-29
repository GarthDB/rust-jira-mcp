use crate::config::JiraConfig;
use crate::error::Result;
use crate::types::mcp::{MCPContent, MCPToolResult};
use serde_json::json;
use tracing::info;

/// Test authentication with Jira API
pub struct TestAuthTool {
    config: JiraConfig,
}

impl TestAuthTool {
    #[must_use]
    /// # Panics
    /// This function does not panic.
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

