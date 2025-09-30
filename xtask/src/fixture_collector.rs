use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use tracing::{debug, error, info, warn};

pub struct FixtureCollector {
    request_id: u64,
}

impl FixtureCollector {
    pub fn new() -> Self {
        Self { request_id: 1 }
    }

    pub async fn collect_fixtures(
        &self,
        project: &str,
        output_dir: &Path,
        anonymize: bool,
    ) -> Result<()> {
        info!("📦 Collecting fixtures for project: {}", project);
        info!("📁 Output directory: {:?}", output_dir);

        // Create output directory
        fs::create_dir_all(output_dir)?;

        let mut collected_count = 0;

        // 1. Project configuration
        if let Ok(project_data) = self.collect_project_config(project).await {
            let filename = format!("{}_project_config.json", project.to_lowercase());
            let filepath = output_dir.join(filename);
            self.save_fixture(&filepath, &project_data, anonymize)?;
            collected_count += 1;
        }

        // 2. Search for issues
        if let Ok(search_data) = self.collect_issue_search(project).await {
            let filename = format!("{}_issues_search.json", project.to_lowercase());
            let filepath = output_dir.join(filename);
            self.save_fixture(&filepath, &search_data, anonymize)?;
            collected_count += 1;
        }

        // 3. Get specific issue if available
        if let Ok(issue_data) = self.collect_specific_issue(project).await {
            if let Some(issue_key) = issue_data.get("key").and_then(|k| k.as_str()) {
                let filename = format!("{}_issue_{}.json", project.to_lowercase(), issue_key);
                let filepath = output_dir.join(filename);
                self.save_fixture(&filepath, &issue_data, anonymize)?;
                collected_count += 1;
            }
        }

        // 4. System metadata
        if let Ok(metadata) = self.collect_system_metadata().await {
            let filename = "system_metadata.json";
            let filepath = output_dir.join(filename);
            self.save_fixture(&filepath, &metadata, anonymize)?;
            collected_count += 1;
        }

        info!("✅ Collected {} fixtures", collected_count);
        Ok(())
    }

    pub async fn generate_synthetic_fixtures(
        &self,
        project: &str,
        output_dir: &Path,
        count: usize,
    ) -> Result<()> {
        info!("🎭 Generating synthetic fixtures for project: {}", project);
        info!("📁 Output directory: {:?}", output_dir);
        info!("🔢 Count: {}", count);

        // Create output directory
        fs::create_dir_all(output_dir)?;

        // Generate project info
        let project_info = self.generate_project_info(project);
        let project_file =
            output_dir.join(format!("{}_project_config.json", project.to_lowercase()));
        self.save_fixture(&project_file, &project_info, false)?;

        // Generate issues
        let issues = self.generate_issues(project, count);
        let issues_file = output_dir.join(format!("{}_issues_search.json", project.to_lowercase()));
        self.save_fixture(&issues_file, &issues, false)?;

        // Generate system metadata
        let metadata = self.generate_system_metadata();
        let metadata_file = output_dir.join("system_metadata.json");
        self.save_fixture(&metadata_file, &metadata, false)?;

        info!("✅ Generated synthetic fixtures");
        Ok(())
    }

    async fn collect_project_config(&self, project: &str) -> Result<Value> {
        let params = json!({"project_key": project});
        let response = self
            .send_mcp_request("get_project_config", params.as_object().unwrap().clone())
            .await?;

        if let Some(error) = response.get("error") {
            return Err(anyhow::anyhow!("Failed to get project config: {}", error));
        }

        Ok(response)
    }

    async fn collect_issue_search(&self, project: &str) -> Result<Value> {
        let params = json!({
            "jql": format!("project = {} ORDER BY updated DESC", project),
            "max_results": 10
        });
        let response = self
            .send_mcp_request("search_jira_issues", params.as_object().unwrap().clone())
            .await?;

        if let Some(error) = response.get("error") {
            return Err(anyhow::anyhow!("Failed to search issues: {}", error));
        }

        Ok(response)
    }

    async fn collect_specific_issue(&self, project: &str) -> Result<Value> {
        // Try to find an existing issue
        let search_params = json!({
            "jql": format!("project = {} ORDER BY updated DESC", project),
            "max_results": 1
        });
        let search_response = self
            .send_mcp_request(
                "search_jira_issues",
                search_params.as_object().unwrap().clone(),
            )
            .await?;

        if let Some(error) = search_response.get("error") {
            return Err(anyhow::anyhow!("Failed to search for issues: {}", error));
        }

        // Extract first issue key
        let issue_key = self.extract_issue_key(&search_response)?;
        if issue_key.is_none() {
            return Err(anyhow::anyhow!("No issues found"));
        }

        let issue_key = issue_key.unwrap();
        let params = json!({"issue_key": issue_key});
        let response = self
            .send_mcp_request("get_jira_issue", params.as_object().unwrap().clone())
            .await?;

        if let Some(error) = response.get("error") {
            return Err(anyhow::anyhow!("Failed to get issue: {}", error));
        }

        Ok(response)
    }

    async fn collect_system_metadata(&self) -> Result<Value> {
        let mut metadata = Map::new();

        // Collect priorities and statuses
        let priorities_response = self
            .send_mcp_request("get_priorities_and_statuses", Map::new())
            .await?;
        if priorities_response.get("error").is_none() {
            metadata.insert("priorities_and_statuses".to_string(), priorities_response);
        }

        // Collect custom fields
        let fields_response = self
            .send_mcp_request("get_custom_fields", Map::new())
            .await?;
        if fields_response.get("error").is_none() {
            metadata.insert("custom_fields".to_string(), fields_response);
        }

        // Collect link types
        let links_response = self
            .send_mcp_request("get_jira_link_types", Map::new())
            .await?;
        if links_response.get("error").is_none() {
            metadata.insert("link_types".to_string(), links_response);
        }

        // Collect labels
        let labels_response = self.send_mcp_request("get_jira_labels", Map::new()).await?;
        if labels_response.get("error").is_none() {
            metadata.insert("labels".to_string(), labels_response);
        }

        Ok(Value::Object(metadata))
    }

    fn generate_project_info(&self, project: &str) -> Value {
        json!({
            "success": true,
            "project": {
                "key": project,
                "name": format!("Mock {} Project", project),
                "description": format!("Mock description for the {} project", project),
                "lead": {
                    "self": "https://jira.example.com/rest/api/2/user?username=mockuser",
                    "accountId": "mock-12345",
                    "emailAddress": "mockuser@example.com",
                    "displayName": "Mock User",
                    "active": true
                },
                "projectTypeKey": "software",
                "url": format!("https://jira.example.com/rest/api/2/project/{}", project),
                "avatarUrls": {
                    "48x48": "https://jira.example.com/secure/projectavatar?pid=10000&avatarId=10000",
                    "24x24": "https://jira.example.com/secure/projectavatar?size=small&pid=10000&avatarId=10000",
                    "16x16": "https://jira.example.com/secure/projectavatar?size=xsmall&pid=10000&avatarId=10000",
                    "32x32": "https://jira.example.com/secure/projectavatar?size=medium&pid=10000&avatarId=10000"
                }
            }
        })
    }

    fn generate_issues(&self, project: &str, count: usize) -> Value {
        let mut issues = Vec::new();

        for i in 1..=count {
            let issue_key = format!("{}-{}", project, 1000 + i);
            let issue = json!({
                "expand": "renderedFields,names,schema,operations,editmeta,changelog,versionedRepresentations",
                "id": format!("{}", 10000000 + i),
                "self": format!("https://jira.example.com/rest/api/2/issue/{}", issue_key),
                "key": issue_key,
                "fields": {
                    "summary": format!("Mock Issue Summary for {}", issue_key),
                    "description": format!("This is a mock description for issue {}", issue_key),
                    "issuetype": {
                        "self": "https://jira.example.com/rest/api/2/issuetype/7",
                        "id": "7",
                        "description": "A standard user story",
                        "iconUrl": "https://jira.example.com/secure/viewavatar?size=xsmall&avatarId=18815&avatarType=issuetype",
                        "name": "Story",
                        "subtask": false,
                        "avatarId": 18815
                    },
                    "project": {
                        "self": format!("https://jira.example.com/rest/api/2/project/{}", project),
                        "id": "10000",
                        "key": project,
                        "name": format!("Mock {} Project", project),
                        "projectTypeKey": "software"
                    },
                    "status": {
                        "self": "https://jira.example.com/rest/api/2/status/3",
                        "description": "Status for In Progress",
                        "iconUrl": "https://jira.example.com/images/icons/statuses/visible.png",
                        "name": "In Progress",
                        "id": "3",
                        "statusCategory": {
                            "self": "https://jira.example.com/rest/api/2/statuscategory/indeterminate",
                            "id": 3,
                            "key": "indeterminate",
                            "colorName": "yellow",
                            "name": "In Progress"
                        }
                    },
                    "priority": {
                        "self": "https://jira.example.com/rest/api/2/priority/8",
                        "iconUrl": "https://jira.example.com/images/icons/priorities/normal.png",
                        "name": "Normal",
                        "id": "8"
                    },
                    "reporter": {
                        "self": "https://jira.example.com/rest/api/2/user?username=mockreporter",
                        "accountId": "mock-reporter-123",
                        "emailAddress": "mockreporter@example.com",
                        "displayName": "Mock Reporter",
                        "active": true
                    },
                    "assignee": {
                        "self": "https://jira.example.com/rest/api/2/user?username=mockassignee",
                        "accountId": "mock-assignee-123",
                        "emailAddress": "mockassignee@example.com",
                        "displayName": "Mock Assignee",
                        "active": true
                    },
                    "created": chrono::Utc::now().to_rfc3339(),
                    "updated": chrono::Utc::now().to_rfc3339(),
                    "labels": ["mock", "test", "synthetic"],
                    "components": [],
                    "fixVersions": [],
                    "issuelinks": [],
                    "watches": {
                        "self": format!("https://jira.example.com/rest/api/2/issue/{}/watchers", issue_key),
                        "watchCount": 0,
                        "isWatching": false
                    },
                    "votes": {
                        "self": format!("https://jira.example.com/rest/api/2/issue/{}/votes", issue_key),
                        "votes": 0,
                        "hasVoted": false
                    }
                }
            });
            issues.push(issue);
        }

        json!({
            "expand": "schema,names",
            "startAt": 0,
            "maxResults": count,
            "total": count,
            "issues": issues
        })
    }

    fn generate_system_metadata(&self) -> Value {
        json!({
            "priorities_and_statuses": {
                "priorities": [
                    {"id": "1", "name": "Highest"},
                    {"id": "8", "name": "Normal"},
                    {"id": "5", "name": "Low"}
                ],
                "statuses": [
                    {"id": "10019", "name": "Backlog", "statusCategory": {"key": "new", "name": "To Do"}},
                    {"id": "3", "name": "In Progress", "statusCategory": {"key": "indeterminate", "name": "In Progress"}},
                    {"id": "10033", "name": "Done", "statusCategory": {"key": "done", "name": "Done"}}
                ]
            },
            "custom_fields": [],
            "link_types": [
                {"id": "10000", "name": "Relates", "inward": "relates to", "outward": "relates to"},
                {"id": "10001", "name": "Blocks", "inward": "is blocked by", "outward": "blocks"}
            ],
            "labels": ["mock", "test", "synthetic", "automated"]
        })
    }

    fn save_fixture(&self, filepath: &Path, data: &Value, anonymize: bool) -> Result<()> {
        let mut data = data.clone();

        if anonymize {
            data = self.anonymize_data(data);
        }

        let json_str = serde_json::to_string_pretty(&data)?;
        fs::write(filepath, json_str)?;

        info!("💾 Saved fixture: {:?}", filepath);
        Ok(())
    }

    fn anonymize_data(&self, mut data: Value) -> Value {
        match &mut data {
            Value::Object(map) => {
                for (key, value) in map.iter_mut() {
                    match key.as_str() {
                        "emailAddress" => *value = Value::String("anon@example.com".to_string()),
                        "displayName" => *value = Value::String("Anonymous User".to_string()),
                        "name" if value.is_string() && value.as_str() != Some("garthdb") => {
                            *value = Value::String("anonuser".to_string());
                        }
                        "self" if value.is_string() => {
                            if let Some(s) = value.as_str() {
                                *value = Value::String(
                                    s.replace("jira.corp.adobe.com", "jira.example.com"),
                                );
                            }
                        }
                        _ => {
                            *value = self.anonymize_data(value.clone());
                        }
                    }
                }
            }
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    *item = self.anonymize_data(item.clone());
                }
            }
            _ => {}
        }
        data
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
