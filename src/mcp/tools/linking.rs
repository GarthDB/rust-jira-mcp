use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::mcp::{MCPContent, MCPToolResult};
use tracing::info;

/// Get issue links for a specific issue
pub struct GetIssueLinksTool {
    client: JiraClient,
}

impl GetIssueLinksTool {
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
impl crate::mcp::server::MCPToolHandler for GetIssueLinksTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let issue_key = args
            .get("issue_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issue_key")
            })?;

        info!("Getting issue links for: {}", issue_key);

        let links = self.client.get_issue_links(issue_key).await?;

        let mut content = vec![MCPContent::text(format!(
            "Found {} links for issue {}\n\n",
            links.len(),
            issue_key
        ))];

        for link in links {
            let link_text = format!("• {} - {}\n", link.id, link.link_type.name);
            content.push(MCPContent::text(link_text));
        }

        Ok(MCPToolResult {
            content,
            is_error: Some(false),
        })
    }
}

/// Create a link between two issues
pub struct CreateIssueLinkTool {
    client: JiraClient,
}

impl CreateIssueLinkTool {
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
impl crate::mcp::server::MCPToolHandler for CreateIssueLinkTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let link_type = args
            .get("link_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: link_type")
            })?;

        let inward_issue = args
            .get("inward_issue")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: inward_issue")
            })?;

        let outward_issue = args
            .get("outward_issue")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: outward_issue")
            })?;

        let comment = args.get("comment").and_then(|v| v.as_str());

        info!(
            "Creating link between {} and {} with type {}",
            inward_issue, outward_issue, link_type
        );

        let link_request = crate::types::jira::JiraIssueLinkCreateRequest {
            link_type: crate::types::jira::JiraIssueLinkType {
                name: link_type.to_string(),
            },
            inward_issue: Some(crate::types::jira::JiraIssueLinkTarget {
                key: inward_issue.to_string(),
            }),
            outward_issue: Some(crate::types::jira::JiraIssueLinkTarget {
                key: outward_issue.to_string(),
            }),
            comment: comment.map(|c| crate::types::jira::JiraIssueLinkComment {
                body: c.to_string(),
                visibility: None,
            }),
        };

        self.client.create_issue_link(&link_request).await?;

        let response_text = format!(
            "Link created successfully between {inward_issue} and {outward_issue} with type {link_type}"
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Delete an issue link
pub struct DeleteIssueLinkTool {
    client: JiraClient,
}

impl DeleteIssueLinkTool {
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
impl crate::mcp::server::MCPToolHandler for DeleteIssueLinkTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let link_id = args
            .get("link_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: link_id")
            })?;

        info!("Deleting issue link: {}", link_id);

        self.client.delete_issue_link(link_id).await?;

        let response_text = format!("Issue link deleted successfully: {link_id}");

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}
