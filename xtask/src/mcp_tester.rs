use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::io::Write;
use std::process::{Command, Stdio};
use tracing::{debug, error, info, warn};

pub struct MCPTester {
    request_id: u64,
}

impl MCPTester {
    pub fn new() -> Self {
        Self { request_id: 1 }
    }

    pub async fn test_operation(&self, operation: &str, params: Map<String, Value>) -> Result<()> {
        info!("🔧 Testing operation: {}", operation);
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

    pub async fn run_test_suite(&self, project: &str, suite: &str) -> Result<()> {
        info!("🚀 Running {} test suite for project: {}", suite, project);

        match suite {
            "read-only" => self.run_read_only_tests(project).await,
            "issues" => self.run_issue_tests(project).await,
            "write" => self.run_write_tests(project).await,
            "all" => self.run_all_tests(project).await,
            _ => {
                error!("Unknown test suite: {}", suite);
                Err(anyhow::anyhow!("Unknown test suite: {}", suite))
            }
        }
    }

    async fn run_read_only_tests(&self, project: &str) -> Result<()> {
        info!("🔍 Running read-only tests for {}", project);

        let tests = vec![
            ("test_jira_auth", json!({"random_string": "test"})),
            ("get_project_config", json!({"project_key": project})),
            ("get_project_components", json!({"project_key": project})),
            ("get_project_issue_types", json!({"project_key": project})),
            ("get_priorities_and_statuses", json!({})),
            ("get_custom_fields", json!({})),
            ("get_jira_link_types", json!({})),
            ("get_jira_labels", json!({})),
        ];

        let mut success_count = 0;
        let test_count = tests.len();
        for (operation, params) in tests {
            if let Err(e) = self
                .test_operation(operation, params.as_object().unwrap().clone())
                .await
            {
                warn!("⚠️ {} failed: {}", operation, e);
            } else {
                success_count += 1;
            }
        }

        info!(
            "📊 Read-only tests: {}/{} succeeded",
            success_count, test_count
        );
        Ok(())
    }

    async fn run_issue_tests(&self, project: &str) -> Result<()> {
        info!("🔧 Running issue tests for {}", project);

        // First, search for existing issues
        let search_params = json!({
            "jql": format!("project = {} ORDER BY updated DESC", project),
            "max_results": 5
        });

        let search_response = self
            .send_mcp_request(
                "search_jira_issues",
                search_params.as_object().unwrap().clone(),
            )
            .await?;

        if let Some(error) = search_response.get("error") {
            warn!("⚠️ Search failed: {}, skipping issue tests", error);
            return Ok(());
        }

        // Extract issue key from search results
        let issue_key = self.extract_issue_key(&search_response)?;
        if issue_key.is_none() {
            warn!("⚠️ No issues found for testing");
            return Ok(());
        }

        let issue_key = issue_key.unwrap();
        info!("🎯 Using issue: {}", issue_key);

        let tests = vec![
            ("get_jira_issue", json!({"issue_key": issue_key})),
            ("get_jira_comments", json!({"issue_key": issue_key})),
            (
                "get_jira_issue_attachments",
                json!({"issue_key": issue_key}),
            ),
            ("get_jira_issue_work_logs", json!({"issue_key": issue_key})),
            ("get_jira_issue_watchers", json!({"issue_key": issue_key})),
        ];

        let mut success_count = 0;
        let test_count = tests.len();
        for (operation, params) in tests {
            if let Err(e) = self
                .test_operation(operation, params.as_object().unwrap().clone())
                .await
            {
                warn!("⚠️ {} failed: {}", operation, e);
            } else {
                success_count += 1;
            }
        }

        info!("📊 Issue tests: {}/{} succeeded", success_count, test_count);
        Ok(())
    }

    async fn run_write_tests(&self, project: &str) -> Result<()> {
        if project == "DNA" {
            warn!("⚠️ Skipping write tests for DNA project to avoid production data");
            return Ok(());
        }

        info!("✏️ Running write tests for {}", project);
        info!("⚠️ WARNING: This will create test data!");

        // Create test issue
        let test_issue_data = json!({
            "project_key": project,
            "issue_type": "Task",
            "summary": format!("MCP-TEST-{} - Automated Test", chrono::Utc::now().format("%Y%m%d-%H%M%S")),
            "description": "This is an automated test issue. Safe to delete.",
            "labels": ["MCP-TEST", "AUTOMATED", "SAFE-TO-DELETE"]
        });

        let create_response = self
            .send_mcp_request(
                "create_jira_issue",
                test_issue_data.as_object().unwrap().clone(),
            )
            .await?;

        if let Some(error) = create_response.get("error") {
            error!("❌ Failed to create test issue: {}", error);
            return Err(anyhow::anyhow!("Failed to create test issue: {}", error));
        }

        let issue_key = self.extract_issue_key(&create_response)?;
        if issue_key.is_none() {
            error!("❌ Could not extract issue key from create response");
            return Err(anyhow::anyhow!("Could not extract issue key"));
        }

        let issue_key = issue_key.unwrap();
        info!("✅ Created test issue: {}", issue_key);

        // Test write operations
        let tests = vec![
            (
                "update_jira_issue",
                json!({
                    "issue_key": issue_key,
                    "fields": {"description": "Updated test description"}
                }),
            ),
            (
                "add_jira_comment",
                json!({
                    "issue_key": issue_key,
                    "comment": format!("Test comment at {}", chrono::Utc::now().to_rfc3339())
                }),
            ),
            (
                "add_jira_work_log",
                json!({
                    "issue_key": issue_key,
                    "time_spent": "1m",
                    "comment": "Test work log"
                }),
            ),
            ("add_jira_issue_watcher", json!({"issue_key": issue_key})),
        ];

        let mut success_count = 0;
        let test_count = tests.len();
        for (operation, params) in tests {
            if let Err(e) = self
                .test_operation(operation, params.as_object().unwrap().clone())
                .await
            {
                warn!("⚠️ {} failed: {}", operation, e);
            } else {
                success_count += 1;
            }
        }

        // Cleanup
        info!("🧹 Cleaning up test issue: {}", issue_key);
        let cleanup_response = self
            .send_mcp_request(
                "delete_jira_issue",
                json!({"issue_key": issue_key}).as_object().unwrap().clone(),
            )
            .await?;

        if let Some(error) = cleanup_response.get("error") {
            warn!("⚠️ Cleanup failed: {}", error);
        } else {
            info!("✅ Cleanup completed");
        }

        info!("📊 Write tests: {}/{} succeeded", success_count, test_count);
        Ok(())
    }

    async fn run_all_tests(&self, project: &str) -> Result<()> {
        info!("🚀 Running all test suites for {}", project);

        self.run_read_only_tests(project).await?;
        self.run_issue_tests(project).await?;

        if project != "DNA" {
            self.run_write_tests(project).await?;
        } else {
            info!("⚠️ Skipping write tests for DNA project");
        }

        Ok(())
    }

    async fn send_mcp_request(&self, method: &str, params: Map<String, Value>) -> Result<Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": "tools/call",
            "params": {
                "name": method,
                "arguments": params
            }
        });

        let mut cmd = Command::new("./target/release/rust-jira-mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start MCP server")?;

        let mut stdin = cmd.stdin.take().unwrap();
        let request_json = serde_json::to_string(&request)? + "\n";
        stdin.write_all(request_json.as_bytes())?;
        drop(stdin);

        let output = cmd.wait_with_output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("MCP server error: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.trim().split('\n').collect();

        for line in lines {
            if line.trim().starts_with('{') {
                return Ok(serde_json::from_str(line)?);
            }
        }

        Err(anyhow::anyhow!("No JSON response found in output"))
    }

    fn extract_issue_key(&self, response: &Value) -> Result<Option<String>> {
        if let Some(result) = response.get("result") {
            if let Some(content) = result.get("content") {
                if let Some(content_array) = content.as_array() {
                    for item in content_array {
                        if let Some(text) = item.get("text") {
                            if let Ok(data) =
                                serde_json::from_str::<Value>(text.as_str().unwrap_or(""))
                            {
                                if let Some(key) = data.get("key") {
                                    return Ok(Some(key.as_str().unwrap_or("").to_string()));
                                }
                                if let Some(issues) = data.get("issues") {
                                    if let Some(issues_array) = issues.as_array() {
                                        if let Some(first_issue) = issues_array.first() {
                                            if let Some(key) = first_issue.get("key") {
                                                return Ok(Some(
                                                    key.as_str().unwrap_or("").to_string(),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }
}
