use crate::config::JiraConfig;
use crate::jira::client::JiraClient;
use mockito::{Mock, Server};
use serde_json::json;

/// Builder for creating mock Jira API responses
pub struct JiraMockBuilder {
    server: mockito::ServerGuard,
    base_url: String,
    zephyr_base_url: String,
}

impl JiraMockBuilder {
    /// Create a new mock builder
    pub async fn new() -> Self {
        let server = Server::new_async().await;
        let base_url = server.url();
        let zephyr_base_url = base_url.replace("/rest/api/2", "/rest/zephyr/latest");
        Self {
            server,
            base_url,
            zephyr_base_url,
        }
    }

    /// Get the base URL for the mock server
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the Zephyr base URL for the mock server
    pub fn zephyr_base_url(&self) -> &str {
        &self.zephyr_base_url
    }

    /// Create a Jira client configured to use this mock server
    pub fn create_client(&self) -> JiraClient {
        let config = JiraConfig {
            api_base_url: format!("{}/rest/api/2", self.base_url),
            email: "test@example.com".to_string(),
            personal_access_token: "test-token".to_string(),
            default_project: Some("TEST".to_string()),
            max_results: Some(50),
            timeout_seconds: Some(30),
            log_file: None,
            strict_ssl: Some(false), // Disable SSL verification for tests
        };
        JiraClient::new(config).expect("Failed to create JiraClient")
    }

    // --- Jira API Mocks ---

    /// Mock a successful issue retrieval
    pub async fn mock_get_issue(&mut self, issue_key: &str) -> Mock {
        let response = json!({
            "id": "12345",
            "key": issue_key,
            "self": format!("{}/rest/api/2/issue/12345", self.base_url),
            "fields": {
                "summary": "Test Issue Summary",
                "description": "This is a test issue description",
                "status": {
                    "id": "1",
                    "name": "To Do",
                    "description": "Issue is in To Do status",
                    "statusCategory": {
                        "id": 2,
                        "key": "new",
                        "colorName": "blue-gray",
                        "name": "To Do"
                    }
                },
                "priority": {
                    "id": "3",
                    "name": "Medium",
                    "iconUrl": "https://test-jira.example.com/images/icons/priorities/medium.svg"
                },
                "issuetype": {
                    "id": "10001",
                    "name": "Story",
                    "description": "A user story",
                    "iconUrl": "https://test-jira.example.com/images/icons/issuetypes/story.svg",
                    "subtask": false
                },
                "project": {
                    "id": "10000",
                    "key": "TEST",
                    "name": "Test Project",
                    "projectTypeKey": "software",
                    "avatarUrls": {}
                },
                "assignee": {
                    "accountId": "test-user-123",
                    "displayName": "Test User",
                    "emailAddress": "test.user@example.com",
                    "avatarUrls": {}
                },
                "reporter": {
                    "accountId": "test-reporter-123",
                    "displayName": "Test Reporter",
                    "emailAddress": "test.reporter@example.com",
                    "avatarUrls": {}
                },
                "created": "2024-01-01T10:00:00.000+0000",
                "updated": "2024-01-01T10:00:00.000+0000",
                "labels": ["test", "example"],
                "components": [],
                "fixVersions": []
            }
        });

        let path = format!("/rest/api/2/issue/{}", issue_key);
        self.server
            .mock("GET", &*path)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create()
    }

    /// Mock a 404 Not Found response for an issue
    pub async fn mock_issue_not_found(&mut self, issue_key: &str) -> Mock {
        let response = json!({
            "errorMessages": ["Issue Does Not Exist"],
            "errors": {}
        });

        let path = format!("/rest/api/2/issue/{}", issue_key);
        self.server
            .mock("GET", &*path)
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create()
    }

    /// Mock a successful issue search
    pub async fn mock_search_issues(&mut self, jql: &str) -> Mock {
        let response = json!({
            "expand": "names,schema",
            "startAt": 0,
            "maxResults": 50,
            "total": 1,
            "issues": [{
                "id": "12345",
                "key": "TEST-123",
                "self": format!("{}/rest/api/2/issue/12345", self.base_url),
                "fields": {
                    "summary": "Test Issue Summary",
                    "status": {
                        "id": "1",
                        "name": "To Do",
                        "statusCategory": {
                            "id": 2,
                            "key": "new",
                            "colorName": "blue-gray",
                            "name": "To Do"
                        }
                    }
                }
            }]
        });

        self.server
            .mock("GET", "/rest/api/2/search")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "jql".to_string(),
                jql.to_string(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create()
    }

    /// Mock a successful issue creation
    pub async fn mock_create_issue(&mut self) -> Mock {
        let response = json!({
            "id": "12346",
            "key": "TEST-124",
            "self": format!("{}/rest/api/2/issue/12346", self.base_url)
        });

        self.server
            .mock("POST", "/rest/api/2/issue")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create()
    }

    /// Mock a successful issue update
    pub async fn mock_update_issue(&mut self, issue_key: &str) -> Mock {
        let path = format!("/rest/api/2/issue/{}", issue_key);
        self.server.mock("PUT", &*path).with_status(204).create()
    }

    /// Mock a successful comment creation
    pub async fn mock_add_comment(&mut self, issue_key: &str) -> Mock {
        let response = json!({
            "id": "10001",
            "body": "Test comment",
            "author": {
                "accountId": "test-user-123",
                "displayName": "Test User",
                "emailAddress": "test.user@example.com",
                "avatarUrls": {}
            },
            "created": "2024-01-01T10:00:00.000+0000",
            "updated": "2024-01-01T10:00:00.000+0000"
        });

        let path = format!("/rest/api/2/issue/{}/comment", issue_key);
        self.server
            .mock("POST", &*path)
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create()
    }

    /// Mock a successful retrieval of comments
    pub async fn mock_get_comments(&mut self, issue_key: &str) -> Mock {
        let response = json!({
            "comments": [
                {
                    "id": "10001",
                    "body": "Test comment 1",
                    "author": { "accountId": "test-user-123", "displayName": "Test User" },
                    "created": "2024-01-01T10:00:00.000+0000"
                },
                {
                    "id": "10002",
                    "body": "Test comment 2",
                    "author": { "accountId": "test-user-123", "displayName": "Test User" },
                    "created": "2024-01-01T10:05:00.000+0000"
                }
            ]
        });

        let path = format!("/rest/api/2/issue/{}/comment", issue_key);
        self.server
            .mock("GET", &*path)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create()
    }

    /// Mock a successful retrieval of transitions
    pub async fn mock_get_transitions(&mut self, issue_key: &str) -> Mock {
        let response = json!({
            "transitions": [
                {
                    "id": "1",
                    "name": "To Do",
                    "to": { "id": "1", "name": "To Do", "statusCategory": { "key": "new" } }
                },
                {
                    "id": "2",
                    "name": "In Progress",
                    "to": { "id": "3", "name": "In Progress", "statusCategory": { "key": "indeterminate" } }
                }
            ]
        });

        let path = format!("/rest/api/2/issue/{}/transitions", issue_key);
        self.server
            .mock("GET", &*path)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create()
    }

    /// Mock a successful issue transition
    pub async fn mock_transition_issue(&mut self, issue_key: &str) -> Mock {
        let path = format!("/rest/api/2/issue/{}/transitions", issue_key);
        self.server.mock("POST", &*path).with_status(204).create()
    }

    /// Mock a server error (500)
    pub async fn mock_server_error(&mut self, endpoint: &str) -> Mock {
        let response = json!({
            "errorMessages": ["Internal Server Error"],
            "errors": {}
        });

        self.server
            .mock("GET", endpoint)
            .with_status(500)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create()
    }

    /// Mock a timeout error
    pub async fn mock_timeout(&mut self, endpoint: &str) -> Mock {
        self.server
            .mock("GET", endpoint)
            .with_status(408)
            .with_header("content-type", "application/json")
            .with_body("Request Timeout")
            .create()
    }

    /// Mock authentication failure
    pub async fn mock_auth_failure(&mut self, endpoint: &str) -> Mock {
        let response = json!({
            "errorMessages": ["You do not have permission to view this issue"],
            "errors": {}
        });

        self.server
            .mock("GET", endpoint)
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create()
    }
}

impl Default for JiraMockBuilder {
    fn default() -> Self {
        // This is a placeholder - in practice, you'd need to handle the async nature
        // For now, we'll panic if this is called synchronously
        panic!("JiraMockBuilder::default() cannot be called synchronously. Use JiraMockBuilder::new().await instead.")
    }
}

/// Helper function to create a mock server and client
pub async fn create_mock_client() -> (JiraMockBuilder, JiraClient) {
    let mock_builder = JiraMockBuilder::new().await;
    let client = mock_builder.create_client();
    (mock_builder, client)
}

/// Helper function to create a mock server with a successful issue response
pub async fn create_mock_with_issue(issue_key: &str) -> (JiraMockBuilder, JiraClient, Mock) {
    let mut mock_builder = JiraMockBuilder::new().await;
    let client = mock_builder.create_client();
    let mock = mock_builder.mock_get_issue(issue_key).await;
    (mock_builder, client, mock)
}
