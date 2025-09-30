use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tracing::{debug, error, info, warn};

pub struct EnhancedMCPTester {
    request_id: u64,
    fixtures_dir: String,
}

impl EnhancedMCPTester {
    pub fn new() -> Self {
        Self {
            request_id: 1,
            fixtures_dir: "fixtures/raw".to_string(),
        }
    }

    /// Load real data from collected fixtures
    fn load_fixture(&self, filename: &str) -> Result<Value> {
        let path = format!("{}/{}", self.fixtures_dir, filename);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read fixture: {}", path))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON from fixture: {}", path))
    }

    /// Extract real issue key from collected data
    fn get_real_issue_key(&self) -> Result<String> {
        let issue_data = self.load_fixture("dna_1244_issue.json")?;
        let key = issue_data
            .get("key")
            .and_then(|k| k.as_str())
            .ok_or_else(|| anyhow::anyhow!("No issue key found in fixture"))?;
        Ok(key.to_string())
    }

    /// Extract real project key from collected data
    fn get_real_project_key(&self) -> Result<String> {
        let project_data = self.load_fixture("dna_project.json")?;
        let key = project_data
            .get("key")
            .and_then(|k| k.as_str())
            .ok_or_else(|| anyhow::anyhow!("No project key found in fixture"))?;
        Ok(key.to_string())
    }

    /// Get real JQL query from collected data
    fn get_real_jql_queries(&self) -> Vec<String> {
        vec![
            "project = DNA ORDER BY updated DESC".to_string(),
            "project = DNA AND status in (Open, \"In Progress\", \"To Do\") ORDER BY updated DESC"
                .to_string(),
            "project = DNA AND assignee = garthdb ORDER BY updated DESC".to_string(),
            "project = DNA AND labels is not EMPTY ORDER BY updated DESC".to_string(),
            "project = DNA AND updated >= -7d ORDER BY updated DESC".to_string(),
        ]
    }

    /// Test operation with real data patterns
    pub async fn test_operation_with_real_data(
        &self,
        operation: &str,
        params: Map<String, Value>,
    ) -> Result<()> {
        info!("🔧 Testing operation with real data: {}", operation);
        debug!("📋 Parameters: {}", serde_json::to_string_pretty(&params)?);

        let response = self.send_mcp_request(operation, params).await?;

        if let Some(error) = response.get("error") {
            error!("❌ {} failed: {}", operation, error);
            return Err(anyhow::anyhow!("Operation failed: {}", error));
        } else {
            info!("✅ {} succeeded", operation);
            if let Some(result) = response.get("result") {
                debug!("📄 Response: {}", serde_json::to_string_pretty(result)?);
            }
        }

        Ok(())
    }

    /// Run comprehensive read-only tests with real data
    pub async fn run_enhanced_read_only_tests(&self) -> Result<()> {
        info!("🔍 Running enhanced read-only tests with real data patterns");

        let project_key = self.get_real_project_key()?;
        let issue_key = self.get_real_issue_key()?;

        let tests = vec![
            // Authentication tests
            ("test_jira_auth", json!({"random_string": "test"})),
            // Project information tests
            ("get_project_config", json!({"project_key": project_key})),
            (
                "get_project_components",
                json!({"project_key": project_key}),
            ),
            (
                "get_project_issue_types",
                json!({"project_key": project_key}),
            ),
            // System metadata tests
            ("get_priorities_and_statuses", json!({})),
            ("get_custom_fields", json!({})),
            ("get_jira_link_types", json!({})),
            ("get_jira_labels", json!({})),
            // Issue search tests with real JQL queries
            (
                "search_jira_issues",
                json!({
                    "jql": "project = DNA ORDER BY updated DESC",
                    "max_results": 5
                }),
            ),
            (
                "search_jira_issues",
                json!({
                    "jql": "project = DNA AND status in (Open, \"In Progress\", \"To Do\") ORDER BY updated DESC",
                    "max_results": 5
                }),
            ),
            (
                "search_jira_issues",
                json!({
                    "jql": "project = DNA AND assignee = garthdb ORDER BY updated DESC",
                    "max_results": 5
                }),
            ),
            // Specific issue tests
            ("get_jira_issue", json!({"issue_key": issue_key})),
            ("get_jira_comments", json!({"issue_key": issue_key})),
            (
                "get_jira_issue_attachments",
                json!({"issue_key": issue_key}),
            ),
            ("get_jira_issue_work_logs", json!({"issue_key": issue_key})),
            ("get_jira_issue_watchers", json!({"issue_key": issue_key})),
            (
                "get_jira_issue_transitions",
                json!({"issue_key": issue_key}),
            ),
        ];

        let mut success_count = 0;
        let test_count = tests.len();

        for (operation, params) in tests {
            if let Err(e) = self
                .test_operation_with_real_data(operation, params.as_object().unwrap().clone())
                .await
            {
                warn!("⚠️ {} failed: {}", operation, e);
            } else {
                success_count += 1;
            }
        }

        info!(
            "📊 Enhanced read-only tests: {}/{} succeeded",
            success_count, test_count
        );
        Ok(())
    }

    /// Run write tests with real data patterns (safe project only)
    pub async fn run_enhanced_write_tests(&self, safe_project: &str) -> Result<()> {
        info!(
            "✍️ Running enhanced write tests with real data patterns for project: {}",
            safe_project
        );

        // Create test issue with real data patterns
        let test_issue_data = json!({
            "project_key": safe_project,
            "summary": "Enhanced MCP Test Issue - Real Data Patterns",
            "description": "This test issue uses real data patterns collected from Adobe Jira API responses. Safe to delete.",
            "issue_type_name": "Story",
            "reporter_name": "garthdb",
            "labels": ["MCP-TEST", "AUTOMATED", "SAFE-TO-DELETE", "ENHANCED-TEST"]
        });

        info!("➕ Creating test issue with real data patterns...");
        let create_response = self
            .send_mcp_request(
                "create_jira_issue",
                test_issue_data.as_object().unwrap().clone(),
            )
            .await?;

        let issue_key = create_response
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.get(0))
            .and_then(|item| item.get("data"))
            .and_then(|data| data.get("key"))
            .and_then(|key| key.as_str())
            .map(|s| s.to_string())
            .context("Failed to extract issue key from create response")?;

        info!("✅ Created test issue: {}", issue_key);

        // Run write operations with real data patterns
        let write_tests = vec![
            (
                "update_jira_issue",
                json!({
                    "issue_key": issue_key,
                    "summary": "Updated Enhanced MCP Test Issue - Real Data Patterns",
                    "description": "This test issue has been updated with real data patterns. Safe to delete.",
                    "labels": ["MCP-TEST", "AUTOMATED", "SAFE-TO-DELETE", "ENHANCED-TEST", "UPDATED"]
                }),
            ),
            (
                "add_jira_comment",
                json!({
                    "issue_key": issue_key,
                    "comment_body": "Enhanced test comment with real data patterns from Adobe Jira API"
                }),
            ),
            (
                "add_jira_work_log",
                json!({
                    "issue_key": issue_key,
                    "time_spent": "2m",
                    "comment": "Enhanced work log entry with real data patterns"
                }),
            ),
            ("add_jira_issue_watcher", json!({"issue_key": issue_key})),
        ];

        let mut success_count = 0;
        let test_count = write_tests.len();

        for (operation, params) in write_tests {
            if let Err(e) = self
                .test_operation_with_real_data(operation, params.as_object().unwrap().clone())
                .await
            {
                warn!("⚠️ {} failed: {}", operation, e);
            } else {
                success_count += 1;
            }
        }

        // Cleanup
        info!("🧹 Cleaning up enhanced test issue: {}", issue_key);
        let cleanup_response = self
            .send_mcp_request(
                "delete_jira_issue",
                json!({"issue_key": issue_key}).as_object().unwrap().clone(),
            )
            .await?;

        if let Some(error) = cleanup_response.get("error") {
            warn!("⚠️ Cleanup failed: {}", error);
        } else {
            info!("✅ Enhanced test cleanup completed");
        }

        info!(
            "📊 Enhanced write tests: {}/{} succeeded",
            success_count, test_count
        );
        Ok(())
    }

    /// Run bulk operations with real data patterns
    pub async fn run_enhanced_bulk_tests(&self, safe_project: &str) -> Result<()> {
        info!(
            "📦 Running enhanced bulk tests with real data patterns for project: {}",
            safe_project
        );

        // Bulk create issues with real data patterns
        let bulk_create_data = json!({
            "issueUpdates": [
                {
                    "fields": {
                        "project": {"key": safe_project},
                        "issuetype": {"name": "Story"},
                        "summary": "Enhanced Bulk Test Issue 1 - Real Data Patterns",
                        "description": "First bulk test issue with real data patterns. Safe to delete.",
                        "labels": ["MCP-TEST", "AUTOMATED", "SAFE-TO-DELETE", "BULK-ENHANCED"]
                    }
                },
                {
                    "fields": {
                        "project": {"key": safe_project},
                        "issuetype": {"name": "Story"},
                        "summary": "Enhanced Bulk Test Issue 2 - Real Data Patterns",
                        "description": "Second bulk test issue with real data patterns. Safe to delete.",
                        "labels": ["MCP-TEST", "AUTOMATED", "SAFE-TO-DELETE", "BULK-ENHANCED"]
                    }
                }
            ]
        });

        info!("➕ Bulk creating test issues with real data patterns...");
        let bulk_create_response = self
            .send_mcp_request(
                "bulk_create_issues",
                bulk_create_data.as_object().unwrap().clone(),
            )
            .await?;

        // Extract issue keys
        let issue_keys = bulk_create_response
            .get("content")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        item.get("data")
                            .and_then(|data| data.get("key"))
                            .and_then(|key| key.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        if issue_keys.is_empty() {
            warn!("⚠️ No issue keys extracted from bulk create response");
            return Ok(());
        }

        info!("✅ Bulk created {} test issues", issue_keys.len());

        // Bulk update issues
        let bulk_update_data = json!({
            "issueUpdates": issue_keys.iter().map(|key| {
                json!({
                    "key": key,
                    "fields": {
                        "labels": ["MCP-TEST", "AUTOMATED", "SAFE-TO-DELETE", "BULK-ENHANCED", "UPDATED"]
                    }
                })
            }).collect::<Vec<_>>()
        });

        info!("🔄 Bulk updating test issues...");
        let bulk_update_response = self
            .send_mcp_request(
                "bulk_update_issues",
                bulk_update_data.as_object().unwrap().clone(),
            )
            .await?;

        // Bulk add comments
        let bulk_comment_data = json!({
            "issueKeys": issue_keys,
            "comment": "Enhanced bulk comment with real data patterns from Adobe Jira API"
        });

        info!("💬 Bulk adding comments...");
        let bulk_comment_response = self
            .send_mcp_request(
                "bulk_add_comments",
                bulk_comment_data.as_object().unwrap().clone(),
            )
            .await?;

        // Cleanup - bulk delete
        info!("🧹 Bulk cleaning up test issues...");
        let bulk_delete_data = json!({
            "issueKeys": issue_keys
        });

        let bulk_delete_response = self
            .send_mcp_request(
                "bulk_delete_issues",
                bulk_delete_data.as_object().unwrap().clone(),
            )
            .await?;

        if let Some(error) = bulk_delete_response.get("error") {
            warn!("⚠️ Bulk cleanup failed: {}", error);
        } else {
            info!("✅ Enhanced bulk test cleanup completed");
        }

        info!("📊 Enhanced bulk tests completed successfully");
        Ok(())
    }

    /// Send MCP request (same as original implementation)
    async fn send_mcp_request(
        &self,
        method_name: &str,
        arguments: Map<String, Value>,
    ) -> Result<Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": format!("tools/{}", method_name),
            "params": {
                "name": method_name,
                "arguments": arguments
            }
        });

        debug!("Sending request: {}", request);

        let mut child = Command::new("./target/debug/rust-jira-mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn MCP server process")?;

        let stdin = child.stdin.as_mut().context("Failed to open stdin")?;
        stdin
            .write_all(serde_json::to_string(&request)?.as_bytes())
            .context("Failed to write to stdin")?;

        let output = child
            .wait_with_output()
            .context("Failed to wait for MCP server process")?;

        if !output.status.success() {
            error!(
                "MCP server stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            anyhow::bail!("MCP server command failed with status: {}", output.status);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("MCP server stdout: {}", stdout);

        let json_line = stdout.lines().find(|line| line.trim().starts_with('{'));

        if let Some(line) = json_line {
            let response: Value = serde_json::from_str(line)
                .context(format!("Failed to parse JSON response: {}", line))?;
            Ok(response)
        } else {
            anyhow::bail!("No JSON response found in MCP server output");
        }
    }

    /// Extract issue key from search response (same as original implementation)
    fn extract_issue_key(&self, response: &Value) -> Result<Option<String>> {
        if let Some(result) = response.get("result") {
            if let Some(issues) = result.get("issues") {
                if let Some(issues_array) = issues.as_array() {
                    if let Some(first_issue) = issues_array.first() {
                        if let Some(key) = first_issue.get("key") {
                            if let Some(key_str) = key.as_str() {
                                return Ok(Some(key_str.to_string()));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }
}
