use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tracing::{debug, error, info, warn};

pub struct TestRunner {
    request_id: u64,
}

impl TestRunner {
    pub fn new() -> Self {
        Self { request_id: 1 }
    }

    pub async fn run_comprehensive_suite(
        &self,
        project: &str,
        cleanup: bool,
        save_results: bool,
    ) -> Result<()> {
        info!(
            "🚀 Running comprehensive test suite for project: {}",
            project
        );

        let mut test_results = Map::new();
        test_results.insert("project".to_string(), Value::String(project.to_string()));
        test_results.insert(
            "timestamp".to_string(),
            Value::String(chrono::Utc::now().to_rfc3339()),
        );
        test_results.insert("operations".to_string(), Value::Object(Map::new()));
        test_results.insert("created_test_data".to_string(), Value::Array(Vec::new()));
        test_results.insert("cleanup_required".to_string(), Value::Array(Vec::new()));

        let mut operations = Map::new();
        let created_data = Vec::new();
        let mut cleanup_required = Vec::new();

        // Phase 1: Read-only operations
        info!("🔍 Phase 1: Read-only operations");
        let read_ops = self.run_read_only_phase(project).await?;
        operations.insert("read_only".to_string(), Value::Object(read_ops));

        // Phase 2: Issue operations
        info!("🔧 Phase 2: Issue operations");
        let (issue_ops, issue_key) = self.run_issue_phase(project).await?;
        operations.insert("issue_operations".to_string(), Value::Object(issue_ops));

        if let Some(key) = issue_key {
            cleanup_required.push(Value::String(key));
        }

        // Phase 3: Write operations (if safe)
        if project != "DNA" {
            info!("✏️ Phase 3: Write operations");
            let (write_ops, write_cleanup) = self.run_write_phase(project).await?;
            operations.insert("write_operations".to_string(), Value::Object(write_ops));
            cleanup_required.extend(write_cleanup);
        } else {
            info!("⚠️ Skipping write operations for DNA project");
        }

        // Phase 4: Cleanup
        if cleanup {
            info!("🧹 Phase 4: Cleanup");
            let cleanup_ops = self.run_cleanup_phase(&cleanup_required).await?;
            operations.insert("cleanup".to_string(), Value::Object(cleanup_ops));
        } else {
            info!("⚠️ Skipping cleanup phase");
        }

        test_results.insert("operations".to_string(), Value::Object(operations));
        test_results.insert("created_test_data".to_string(), Value::Array(created_data));
        test_results.insert(
            "cleanup_required".to_string(),
            Value::Array(cleanup_required),
        );

        if save_results {
            self.save_test_results(&test_results).await?;
        }

        info!("🎉 Comprehensive test suite completed!");
        Ok(())
    }

    pub async fn cleanup_test_data(&self, project: &str, dry_run: bool) -> Result<()> {
        info!("🧹 Cleaning up test data for project: {}", project);

        if dry_run {
            info!("🔍 Dry run mode - showing what would be cleaned up");
        }

        // Search for test issues
        let search_params = json!({
            "jql": format!("project = {} AND labels = MCP-TEST", project),
            "max_results": 100
        });

        let search_response = self
            .send_mcp_request(
                "search_jira_issues",
                search_params.as_object().unwrap().clone(),
            )
            .await?;

        if let Some(error) = search_response.get("error") {
            warn!("⚠️ Search failed: {}", error);
            return Ok(());
        }

        let issues = self.extract_issues_from_search(&search_response)?;
        info!("🔍 Found {} test issues to clean up", issues.len());

        for issue_key in issues {
            if dry_run {
                info!("🗑️ Would delete: {}", issue_key);
            } else {
                info!("🗑️ Deleting: {}", issue_key);
                let delete_response = self
                    .send_mcp_request(
                        "delete_jira_issue",
                        json!({"issue_key": issue_key}).as_object().unwrap().clone(),
                    )
                    .await?;

                if let Some(error) = delete_response.get("error") {
                    warn!("⚠️ Failed to delete {}: {}", issue_key, error);
                } else {
                    info!("✅ Deleted: {}", issue_key);
                }
            }
        }

        if dry_run {
            info!("🔍 Dry run completed - no actual changes made");
        } else {
            info!("✅ Cleanup completed");
        }

        Ok(())
    }

    async fn run_read_only_phase(&self, project: &str) -> Result<Map<String, Value>> {
        let mut results = Map::new();

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

        for (operation, params) in tests {
            let response = self
                .send_mcp_request(operation, params.as_object().unwrap().clone())
                .await?;
            results.insert(operation.to_string(), response);
        }

        Ok(results)
    }

    async fn run_issue_phase(&self, project: &str) -> Result<(Map<String, Value>, Option<String>)> {
        let mut results = Map::new();

        // Search for issues
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
        results.insert("search_jira_issues".to_string(), search_response.clone());

        // Extract issue key
        let issue_key = self.extract_issue_key(&search_response)?;

        if let Some(key) = &issue_key {
            let tests = vec![
                ("get_jira_issue", json!({"issue_key": key})),
                ("get_jira_comments", json!({"issue_key": key})),
                ("get_jira_issue_attachments", json!({"issue_key": key})),
                ("get_jira_issue_work_logs", json!({"issue_key": key})),
                ("get_jira_issue_watchers", json!({"issue_key": key})),
            ];

            for (operation, params) in tests {
                let response = self
                    .send_mcp_request(operation, params.as_object().unwrap().clone())
                    .await?;
                results.insert(operation.to_string(), response);
            }
        }

        Ok((results, issue_key))
    }

    async fn run_write_phase(&self, project: &str) -> Result<(Map<String, Value>, Vec<Value>)> {
        let mut results = Map::new();
        let mut cleanup_required = Vec::new();

        // Create test issue
        let test_issue_data = json!({
            "project_key": project,
            "issue_type": "Task",
            "summary": format!("MCP-TEST-{} - Comprehensive Test", chrono::Utc::now().format("%Y%m%d-%H%M%S")),
            "description": "Comprehensive test issue created by xtask. Safe to delete.",
            "labels": ["MCP-TEST", "COMPREHENSIVE", "SAFE-TO-DELETE"]
        });

        let create_response = self
            .send_mcp_request(
                "create_jira_issue",
                test_issue_data.as_object().unwrap().clone(),
            )
            .await?;
        results.insert("create_jira_issue".to_string(), create_response);

        let issue_key = self.extract_issue_key(&results["create_jira_issue"])?;
        if let Some(key) = &issue_key {
            cleanup_required.push(Value::String(key.clone()));

            // Test write operations
            let tests = vec![
                (
                    "update_jira_issue",
                    json!({
                        "issue_key": key,
                        "fields": {"description": "Updated comprehensive test description"}
                    }),
                ),
                (
                    "add_jira_comment",
                    json!({
                        "issue_key": key,
                        "comment": format!("Comprehensive test comment at {}", chrono::Utc::now().to_rfc3339())
                    }),
                ),
                (
                    "add_jira_work_log",
                    json!({
                        "issue_key": key,
                        "time_spent": "1m",
                        "comment": "Comprehensive test work log"
                    }),
                ),
                ("add_jira_issue_watcher", json!({"issue_key": key})),
            ];

            for (operation, params) in tests {
                let response = self
                    .send_mcp_request(operation, params.as_object().unwrap().clone())
                    .await?;
                results.insert(operation.to_string(), response);
            }
        }

        Ok((results, cleanup_required))
    }

    async fn run_cleanup_phase(&self, cleanup_items: &[Value]) -> Result<Map<String, Value>> {
        let mut results = Map::new();

        for item in cleanup_items {
            if let Some(issue_key) = item.as_str() {
                let delete_response = self
                    .send_mcp_request(
                        "delete_jira_issue",
                        json!({"issue_key": issue_key}).as_object().unwrap().clone(),
                    )
                    .await?;
                results.insert(format!("delete_{}", issue_key), delete_response);
            }
        }

        Ok(results)
    }

    async fn save_test_results(&self, results: &Map<String, Value>) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!(
            "tests/fixtures/comprehensive_test_results_{}.json",
            timestamp
        );

        fs::create_dir_all("tests/fixtures")?;
        let json_str = serde_json::to_string_pretty(&Value::Object(results.clone()))?;
        fs::write(&filename, json_str)?;

        info!("💾 Test results saved to: {}", filename);
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

    fn extract_issues_from_search(&self, response: &Value) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        if let Some(result) = response.get("result") {
            if let Some(content) = result.get("content") {
                if let Some(content_array) = content.as_array() {
                    for item in content_array {
                        if let Some(text) = item.get("text") {
                            if let Ok(data) =
                                serde_json::from_str::<Value>(text.as_str().unwrap_or(""))
                            {
                                if let Some(issues_array) =
                                    data.get("issues").and_then(|i| i.as_array())
                                {
                                    for issue in issues_array {
                                        if let Some(key) = issue.get("key").and_then(|k| k.as_str())
                                        {
                                            issues.push(key.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(issues)
    }
}
