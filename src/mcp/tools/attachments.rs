use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::mcp::{MCPContent, MCPToolResult};
use base64::Engine;
use tracing::info;

/// Get attachments for a Jira issue
pub struct GetIssueAttachmentsTool {
    client: JiraClient,
}

impl GetIssueAttachmentsTool {
    #[must_use]
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
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: issue_key"))?;

        info!("Getting attachments for issue: {}", issue_key);

        let attachments = self.client.get_issue_attachments(issue_key).await?;

        let mut content = vec![MCPContent::text(format!("Found {} attachments for issue {}\n\n", attachments.len(), issue_key))];

        for attachment in attachments {
            let attachment_text = format!(
                "• {} ({}) - {} bytes\n",
                attachment.filename,
                &attachment.mime_type,
                attachment.size
            );
            content.push(MCPContent::text(attachment_text));
        }

        Ok(MCPToolResult {
            content,
            is_error: Some(false),
        })
    }
}

/// Upload an attachment to a Jira issue
pub struct UploadAttachmentTool {
    client: JiraClient,
}

impl UploadAttachmentTool {
    #[must_use]
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
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: issue_key"))?;

        let filename = args
            .get("filename")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: filename"))?;

        let content_base64 = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: content"))?;

        info!("Uploading attachment {} to issue: {}", filename, issue_key);

        // Decode base64 content
        let content = base64::engine::general_purpose::STANDARD
            .decode(content_base64)
            .map_err(|_| crate::error::JiraError::api_error("Invalid base64 content"))?;

        let mime_type = args.get("mime_type").and_then(|v| v.as_str());

        let uploaded_attachments = self.client.upload_attachment(issue_key, filename, &content, mime_type).await?;

        let response_text = format!(
            "Successfully uploaded {} attachment(s) to issue {}",
            uploaded_attachments.len(),
            issue_key
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Delete an attachment
pub struct DeleteAttachmentTool {
    client: JiraClient,
}

impl DeleteAttachmentTool {
    #[must_use]
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
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: attachment_id"))?;

        info!("Deleting attachment: {}", attachment_id);

        self.client.delete_attachment(attachment_id).await?;

        let response_text = format!("Attachment {} deleted successfully", attachment_id);

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

/// Download an attachment
pub struct DownloadAttachmentTool {
    client: JiraClient,
}

impl DownloadAttachmentTool {
    #[must_use]
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
            .ok_or_else(|| crate::error::JiraError::api_error("Missing required parameter: attachment_id"))?;

        info!("Downloading attachment: {}", attachment_id);

        let content = self.client.download_attachment(attachment_id).await?;

        // Encode content as base64 for transmission
        let content_base64 = base64::engine::general_purpose::STANDARD.encode(&content);

        let response_text = format!(
            "Attachment {} downloaded successfully ({} bytes)\nBase64 content: {}",
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
