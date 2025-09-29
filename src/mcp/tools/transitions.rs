use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::mcp::{MCPContent, MCPToolResult};
use tracing::info;

/// Get available transitions for a Jira issue
pub struct GetTransitionsTool {
    client: JiraClient,
}

impl GetTransitionsTool {
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
impl crate::mcp::server::MCPToolHandler for GetTransitionsTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issue_key")
            })?;

        info!("Getting transitions for issue: {}", issue_key);

        let transitions = self.client.get_transitions(issue_key).await?;

        let mut content = vec![MCPContent::text(format!(
            "Found {} transitions for issue {}\n\n",
            transitions.len(),
            issue_key
        ))];

        for transition in transitions {
            let transition_text = format!(
                "• {} - {} (ID: {})\n",
                transition.name, transition.to.name, transition.id
            );
            content.push(MCPContent::text(transition_text));
        }

        Ok(MCPToolResult {
            content,
            is_error: Some(false),
        })
    }
}

/// Transition a Jira issue to a new status
pub struct TransitionIssueTool {
    client: JiraClient,
}

impl TransitionIssueTool {
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
impl crate::mcp::server::MCPToolHandler for TransitionIssueTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issue_key")
            })?;

        let transition_id = args
            .get("transition_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: transition_id")
            })?;

        let comment = args.get("comment").and_then(|v| v.as_str());

        info!(
            "Transitioning issue {} to transition {}",
            issue_key, transition_id
        );

        self.client
            .transition_issue(issue_key, transition_id, comment)
            .await?;

        let response_text = format!(
            "Issue {} transitioned successfully to transition {}",
            issue_key, transition_id
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}
