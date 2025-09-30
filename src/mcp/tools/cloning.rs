use crate::config::JiraConfig;
use crate::error::{JiraError, Result};
use crate::jira::client::JiraClient;
use crate::types::mcp::{MCPContent, MCPToolResult};
use tracing::info;

/// Clone a Jira issue
pub struct CloneIssueTool {
    client: JiraClient,
}

impl CloneIssueTool {
    #[must_use]
    /// # Panics
    /// This function panics if `JiraClient::new` fails.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

/// Parse basic required parameters for issue cloning
fn parse_basic_parameters(args: &serde_json::Value) -> Result<(String, String, String, String)> {
    let original_issue_key = args
        .get("original_issue_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JiraError::api_error("Missing required parameter: original_issue_key"))?;

    let project_key = args
        .get("project_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JiraError::api_error("Missing required parameter: project_key"))?;

    let issue_type = args
        .get("issue_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JiraError::api_error("Missing required parameter: issue_type"))?;

    let summary = args
        .get("summary")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JiraError::api_error("Missing required parameter: summary"))?;

    Ok((
        original_issue_key.to_string(),
        project_key.to_string(),
        issue_type.to_string(),
        summary.to_string(),
    ))
}

/// Parse optional parameters for issue cloning
fn parse_optional_parameters(args: &serde_json::Value) -> (Option<String>, bool, bool, bool, bool) {
    let description = args
        .get("description")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let copy_attachments = args
        .get("copy_attachments")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let copy_work_logs = args
        .get("copy_work_logs")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let copy_watchers = args
        .get("copy_watchers")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let copy_links = args
        .get("copy_links")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    (
        description,
        copy_attachments,
        copy_work_logs,
        copy_watchers,
        copy_links,
    )
}

/// Parse field mapping parameters for issue cloning
fn parse_field_mapping(args: &serde_json::Value) -> Option<crate::types::jira::JiraFieldMapping> {
    args.get("field_mapping")?;

    let copy_fields = args
        .get("copy_fields")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();

    let exclude_fields = args
        .get("exclude_fields")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();

    let custom_field_mapping = args
        .get("custom_field_mapping")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        });

    Some(crate::types::jira::JiraFieldMapping {
        copy_fields,
        exclude_fields,
        custom_field_mapping,
    })
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for CloneIssueTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        // Parse all parameters using helper functions
        let (original_issue_key, project_key, issue_type, summary) = parse_basic_parameters(&args)?;
        let (description, copy_attachments, copy_work_logs, copy_watchers, copy_links) =
            parse_optional_parameters(&args);
        let field_mapping = parse_field_mapping(&args);

        info!(
            "Cloning issue {} to project {} as {} with field mapping: {:?}",
            original_issue_key, project_key, issue_type, field_mapping
        );

        // Create clone request
        let clone_request = crate::types::jira::JiraIssueCloneRequest {
            project_key: project_key.clone(),
            issue_type: issue_type.clone(),
            summary: summary.clone(),
            description,
            field_mapping,
            copy_attachments: Some(copy_attachments),
            copy_comments: Some(true),
            copy_work_logs: Some(copy_work_logs),
            copy_watchers: Some(copy_watchers),
            copy_links: Some(copy_links),
        };

        // Execute cloning
        let clone_response = self
            .client
            .clone_issue(&original_issue_key, &clone_request)
            .await?;

        // Format response
        let response_text = format!(
            "Issue cloned successfully\nOriginal: {}\nCloned: {}\nProject: {}\nType: {}\nSummary: {}",
            clone_response.original_issue_key,
            clone_response.cloned_issue_key,
            project_key,
            issue_type,
            summary
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}
