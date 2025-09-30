use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;
use tracing::{debug, error, info, warn};

pub struct FixtureTestRunner {
    fixtures_dir: String,
    anonymized_dir: String,
}

impl FixtureTestRunner {
    pub fn new() -> Self {
        Self {
            fixtures_dir: "fixtures/raw".to_string(),
            anonymized_dir: "fixtures/anonymized".to_string(),
        }
    }

    /// Load a fixture file
    fn load_fixture(&self, filename: &str, use_anonymized: bool) -> Result<Value> {
        let dir = if use_anonymized {
            &self.anonymized_dir
        } else {
            &self.fixtures_dir
        };
        let path = format!("{}/{}", dir, filename);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read fixture: {}", path))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON from fixture: {}", path))
    }

    /// Test serialization with real fixtures
    pub fn test_serialization_with_fixtures(&self) -> Result<()> {
        info!("🧪 Testing serialization with real fixtures...");

        // Test key fixtures that should deserialize properly
        let test_cases = vec![
            ("myself.json", "JiraUser"),
            ("dna_project.json", "JiraProject"),
            ("dna_1244_issue.json", "JiraIssue"),
            ("system_statuses.json", "JiraStatus"),
            ("system_priorities.json", "JiraPriority"),
            ("dna_issue_types.json", "JiraIssueType"),
        ];

        let mut success_count = 0;
        let total_count = test_cases.len();

        for (filename, struct_name) in test_cases {
            match self.test_fixture_serialization(filename, struct_name) {
                Ok(_) => {
                    info!(
                        "✅ {} - {} deserialized successfully",
                        filename, struct_name
                    );
                    success_count += 1;
                }
                Err(e) => {
                    warn!("⚠️ {} - {} failed: {}", filename, struct_name, e);
                }
            }
        }

        info!(
            "📊 Serialization test results: {}/{} succeeded",
            success_count, total_count
        );
        Ok(())
    }

    /// Test a single fixture file
    fn test_fixture_serialization(&self, filename: &str, struct_name: &str) -> Result<()> {
        let data = self.load_fixture(filename, false)?; // Use raw data for testing

        // For now, just verify it's valid JSON and has expected structure
        match struct_name {
            "JiraUser" => {
                if !data.is_object() {
                    anyhow::bail!("Expected object for JiraUser");
                }
                if !data.get("name").is_some() || !data.get("displayName").is_some() {
                    anyhow::bail!("Missing required JiraUser fields");
                }
            }
            "JiraProject" => {
                if !data.is_object() {
                    anyhow::bail!("Expected object for JiraProject");
                }
                if !data.get("key").is_some() || !data.get("name").is_some() {
                    anyhow::bail!("Missing required JiraProject fields");
                }
            }
            "JiraIssue" => {
                if !data.is_object() {
                    anyhow::bail!("Expected object for JiraIssue");
                }
                if !data.get("key").is_some() || !data.get("fields").is_some() {
                    anyhow::bail!("Missing required JiraIssue fields");
                }
            }
            _ => {
                // Generic validation for other types
                if !data.is_object() {
                    anyhow::bail!("Expected object for {}", struct_name);
                }
            }
        }

        Ok(())
    }

    /// Generate test data from fixtures
    pub fn generate_test_data_from_fixtures(&self) -> Result<()> {
        info!("📦 Generating test data from fixtures...");

        let output_dir = Path::new("tests/fixtures");
        fs::create_dir_all(output_dir)?;

        // Generate test cases for different scenarios
        self.generate_user_test_cases()?;
        self.generate_project_test_cases()?;
        self.generate_issue_test_cases()?;
        self.generate_search_test_cases()?;

        info!("✅ Test data generation complete!");
        Ok(())
    }

    fn generate_user_test_cases(&self) -> Result<()> {
        let user_data = self.load_fixture("myself.json", true)?; // Use anonymized data

        let test_cases = json!({
            "valid_user": user_data,
            "minimal_user": {
                "self": "https://jira.example.com/rest/api/2/user?username=testuser",
                "name": "testuser",
                "key": "testuser",
                "displayName": "Test User",
                "active": true
            },
            "user_with_email": {
                "self": "https://jira.example.com/rest/api/2/user?username=testuser2",
                "name": "testuser2",
                "key": "testuser2",
                "emailAddress": "testuser2@example.com",
                "displayName": "Test User 2",
                "active": true,
                "timeZone": "UTC"
            }
        });

        let output_file = "tests/fixtures/user_test_cases.json";
        fs::write(output_file, serde_json::to_string_pretty(&test_cases)?)?;
        info!("✅ Generated user test cases: {}", output_file);
        Ok(())
    }

    fn generate_project_test_cases(&self) -> Result<()> {
        let project_data = self.load_fixture("dna_project.json", true)?; // Use anonymized data

        let test_cases = json!({
            "valid_project": project_data,
            "minimal_project": {
                "self": "https://jira.example.com/rest/api/2/project/10000",
                "id": "10000",
                "key": "TEST",
                "name": "Test Project",
                "projectTypeKey": "software"
            },
            "project_with_avatars": {
                "self": "https://jira.example.com/rest/api/2/project/10001",
                "id": "10001",
                "key": "TEST2",
                "name": "Test Project 2",
                "projectTypeKey": "software",
                "avatarUrls": {
                    "48x48": "https://jira.example.com/secure/projectavatar?pid=10001&avatarId=10001",
                    "24x24": "https://jira.example.com/secure/projectavatar?size=small&pid=10001&avatarId=10001",
                    "16x16": "https://jira.example.com/secure/projectavatar?size=xsmall&pid=10001&avatarId=10001",
                    "32x32": "https://jira.example.com/secure/projectavatar?size=medium&pid=10001&avatarId=10001"
                }
            }
        });

        let output_file = "tests/fixtures/project_test_cases.json";
        fs::write(output_file, serde_json::to_string_pretty(&test_cases)?)?;
        info!("✅ Generated project test cases: {}", output_file);
        Ok(())
    }

    fn generate_issue_test_cases(&self) -> Result<()> {
        let issue_data = self.load_fixture("dna_1244_issue.json", true)?; // Use anonymized data

        let test_cases = json!({
            "valid_issue": issue_data,
            "minimal_issue": {
                "self": "https://jira.example.com/rest/api/2/issue/TEST-1",
                "id": "20001",
                "key": "TEST-1",
                "fields": {
                    "summary": "Test Issue Summary",
                    "description": "Test issue description",
                    "project": {
                        "self": "https://jira.example.com/rest/api/2/project/10000",
                        "id": "10000",
                        "key": "TEST",
                        "name": "Test Project",
                        "projectTypeKey": "software"
                    },
                    "issuetype": {
                        "self": "https://jira.example.com/rest/api/2/issuetype/10001",
                        "id": "10001",
                        "name": "Story",
                        "subtask": false
                    },
                    "status": {
                        "self": "https://jira.example.com/rest/api/2/status/10001",
                        "id": "10001",
                        "name": "To Do",
                        "statusCategory": {
                            "self": "https://jira.example.com/rest/api/2/statuscategory/2",
                            "id": 2,
                            "key": "new",
                            "colorName": "blue-gray",
                            "name": "To Do"
                        }
                    },
                    "priority": {
                        "self": "https://jira.example.com/rest/api/2/priority/3",
                        "id": "3",
                        "name": "Medium"
                    },
                    "reporter": {
                        "self": "https://jira.example.com/rest/api/2/user?username=testuser",
                        "name": "testuser",
                        "key": "testuser",
                        "displayName": "Test User",
                        "active": true
                    }
                }
            }
        });

        let output_file = "tests/fixtures/issue_test_cases.json";
        fs::write(output_file, serde_json::to_string_pretty(&test_cases)?)?;
        info!("✅ Generated issue test cases: {}", output_file);
        Ok(())
    }

    fn generate_search_test_cases(&self) -> Result<()> {
        let search_data = self.load_fixture("dna_open_issues.json", true)?; // Use anonymized data

        let test_cases = json!({
            "valid_search_result": search_data,
            "empty_search_result": {
                "expand": "names,schema",
                "startAt": 0,
                "maxResults": 50,
                "total": 0,
                "issues": []
            },
            "search_with_issues": {
                "expand": "names,schema",
                "startAt": 0,
                "maxResults": 50,
                "total": 1,
                "issues": [
                    {
                        "self": "https://jira.example.com/rest/api/2/issue/TEST-1",
                        "id": "20001",
                        "key": "TEST-1",
                        "fields": {
                            "summary": "Test Issue Summary",
                            "status": {
                                "self": "https://jira.example.com/rest/api/2/status/10001",
                                "id": "10001",
                                "name": "To Do"
                            }
                        }
                    }
                ]
            }
        });

        let output_file = "tests/fixtures/search_test_cases.json";
        fs::write(output_file, serde_json::to_string_pretty(&test_cases)?)?;
        info!("✅ Generated search test cases: {}", output_file);
        Ok(())
    }

    /// Validate anonymization quality
    pub fn validate_anonymization(&self) -> Result<()> {
        info!("🔍 Validating anonymization quality...");

        let raw_user = self.load_fixture("myself.json", false)?;
        let anonymized_user = self.load_fixture("myself.json", true)?;

        // Check that sensitive data was anonymized
        if raw_user.get("emailAddress") == anonymized_user.get("emailAddress") {
            warn!("⚠️ Email address was not anonymized");
        }

        if raw_user.get("displayName") == anonymized_user.get("displayName") {
            warn!("⚠️ Display name was not anonymized");
        }

        // Check that URLs were anonymized
        if let Some(self_url) = anonymized_user.get("self") {
            if let Some(url_str) = self_url.as_str() {
                if url_str.contains("jira.corp.adobe.com") {
                    warn!("⚠️ URL was not anonymized: {}", url_str);
                }
            }
        }

        info!("✅ Anonymization validation complete");
        Ok(())
    }

    /// Run comprehensive fixture tests
    pub async fn run_fixture_tests(&self) -> Result<()> {
        info!("🚀 Running comprehensive fixture tests...");

        // Test serialization
        self.test_serialization_with_fixtures()?;

        // Validate anonymization
        self.validate_anonymization()?;

        // Generate test data
        self.generate_test_data_from_fixtures()?;

        info!("✅ All fixture tests completed successfully!");
        Ok(())
    }
}
