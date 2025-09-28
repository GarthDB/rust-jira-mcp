use rust_jira_mcp::config::JiraConfig;
use rust_jira_mcp::jira::client::JiraClient;
use rust_jira_mcp::types::jira::*;
use serde_json::json;
use std::collections::HashMap;

/// Create a test configuration for integration tests
fn create_test_config(base_url: &str) -> JiraConfig {
    JiraConfig {
        api_base_url: base_url.to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test-token".to_string(),
        default_project: Some("TEST".to_string()),
        max_results: Some(50),
        timeout_seconds: Some(30),
        log_file: None,
        strict_ssl: Some(false), // Disable SSL verification for tests
    }
}

#[tokio::test]
async fn test_get_issue_success() {
    let mut server = mockito::Server::new_async().await;
    let base_url = server.url();
    
    // Mock the issue endpoint
    let mock_response = json!({
        "id": "12345",
        "key": "TEST-123",
        "self": format!("{}/rest/api/2/issue/12345", base_url),
        "fields": {
            "summary": "Test Issue",
            "description": "This is a test issue",
            "status": {
                "name": "To Do",
                "id": "1"
            }
        }
    });
    
    let _mock = server
        .mock("GET", "/rest/api/2/issue/TEST-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();
    
    let config = create_test_config(&format!("{}/rest/api/2", base_url));
    let client = JiraClient::new(config).expect("Failed to create JiraClient");
    
    let result = client.get_issue("TEST-123").await;
    assert!(result.is_ok());
    
    let issue = result.unwrap();
    assert_eq!(issue.key, "TEST-123");
    assert_eq!(issue.id, "12345");
}

#[tokio::test]
async fn test_get_issue_not_found() {
    let mut server = mockito::Server::new_async().await;
    let base_url = server.url();
    
    let mock_response = json!({
        "errorMessages": ["Issue Does Not Exist"],
        "errors": {}
    });
    
    let _mock = server
        .mock("GET", "/rest/api/2/issue/TEST-999")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();
    
    let config = create_test_config(&format!("{}/rest/api/2", base_url));
    let client = JiraClient::new(config).expect("Failed to create JiraClient");
    
    let result = client.get_issue("TEST-999").await;
    assert!(result.is_err());
    
    if let Err(error) = result {
        assert!(error.to_string().contains("Issue Does Not Exist"));
    }
}

#[tokio::test]
async fn test_search_issues_success() {
    let mut server = mockito::Server::new_async().await;
    let base_url = server.url();
    
    let mock_response = json!({
        "expand": "names,schema",
        "startAt": 0,
        "maxResults": 50,
        "total": 2,
        "issues": [
            {
                "id": "12345",
                "key": "TEST-123",
                "self": format!("{}/rest/api/2/issue/12345", base_url),
                "fields": {
                    "summary": "Test Issue 1",
                    "status": {
                        "name": "To Do",
                        "id": "1"
                    }
                }
            },
            {
                "id": "12346",
                "key": "TEST-124",
                "self": format!("{}/rest/api/2/issue/12346", base_url),
                "fields": {
                    "summary": "Test Issue 2",
                    "status": {
                        "name": "In Progress",
                        "id": "2"
                    }
                }
            }
        ]
    });
    
    let _mock = server
        .mock("GET", "/rest/api/2/search")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".to_string(), "project = TEST".to_string()),
            mockito::Matcher::UrlEncoded("maxResults".to_string(), "50".to_string()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();
    
    let config = create_test_config(&format!("{}/rest/api/2", base_url));
    let client = JiraClient::new(config).expect("Failed to create JiraClient");
    
    let result = client.search_issues("project = TEST", None, None).await;
    assert!(result.is_ok());
    
    let search_result = result.unwrap();
    assert_eq!(search_result.total, 2);
    assert_eq!(search_result.issues.len(), 2);
    assert_eq!(search_result.issues[0].key, "TEST-123");
    assert_eq!(search_result.issues[1].key, "TEST-124");
}

#[tokio::test]
async fn test_get_project_success() {
    let mut server = mockito::Server::new_async().await;
    let base_url = server.url();
    
    let mock_response = json!({
        "id": "10000",
        "key": "TEST",
        "name": "Test Project",
        "projectTypeKey": "software",
        "self": format!("{}/rest/api/2/project/10000", base_url)
    });
    
    let _mock = server
        .mock("GET", "/rest/api/2/project/TEST")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();
    
    let config = create_test_config(&format!("{}/rest/api/2", base_url));
    let client = JiraClient::new(config).expect("Failed to create JiraClient");
    
    let result = client.get_project("TEST").await;
    assert!(result.is_ok());
    
    let project = result.unwrap();
    assert_eq!(project.key, "TEST");
    assert_eq!(project.name, "Test Project");
    assert_eq!(project.project_type_key, "software");
}

#[tokio::test]
async fn test_get_projects_success() {
    let mut server = mockito::Server::new_async().await;
    let base_url = server.url();
    
    let mock_response = json!([
        {
            "id": "10000",
            "key": "TEST",
            "name": "Test Project",
            "projectTypeKey": "software",
            "self": format!("{}/rest/api/2/project/10000", base_url)
        },
        {
            "id": "10001",
            "key": "DEMO",
            "name": "Demo Project",
            "projectTypeKey": "business",
            "self": format!("{}/rest/api/2/project/10001", base_url)
        }
    ]);
    
    let _mock = server
        .mock("GET", "/rest/api/2/project")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();
    
    let config = create_test_config(&format!("{}/rest/api/2", base_url));
    let client = JiraClient::new(config).expect("Failed to create JiraClient");
    
    let result = client.get_projects().await;
    assert!(result.is_ok());
    
    let projects = result.unwrap();
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].key, "TEST");
    assert_eq!(projects[1].key, "DEMO");
}

#[tokio::test]
async fn test_create_issue_success() {
    let mut server = mockito::Server::new_async().await;
    let base_url = server.url();
    
    let issue_data = json!({
        "fields": {
            "project": {
                "key": "TEST"
            },
            "summary": "Test Issue",
            "description": "This is a test issue",
            "issuetype": {
                "name": "Bug"
            }
        }
    });
    
    let mock_response = json!({
        "id": "12345",
        "key": "TEST-123",
        "self": format!("{}/rest/api/2/issue/12345", base_url)
    });
    
    let _mock = server
        .mock("POST", "/rest/api/2/issue")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();
    
    let config = create_test_config(&format!("{}/rest/api/2", base_url));
    let client = JiraClient::new(config).expect("Failed to create JiraClient");
    
    let result = client.create_issue(&issue_data).await;
    assert!(result.is_ok());
    
    let issue = result.unwrap();
    assert_eq!(issue.key, "TEST-123");
    assert_eq!(issue.id, "12345");
}

#[tokio::test]
async fn test_add_comment_success() {
    let mut server = mockito::Server::new_async().await;
    let base_url = server.url();
    
    let mock_response = json!({
        "id": "12345",
        "body": "This is a test comment",
        "author": {
            "accountId": "12345:abcdef",
            "displayName": "Test User",
            "emailAddress": "test@example.com",
            "active": true,
            "timeZone": "UTC"
        },
        "created": "2023-01-01T00:00:00.000Z"
    });
    
    let _mock = server
        .mock("POST", "/rest/api/2/issue/TEST-123/comment")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();
    
    let config = create_test_config(&format!("{}/rest/api/2", base_url));
    let client = JiraClient::new(config).expect("Failed to create JiraClient");
    
    let result = client.add_comment("TEST-123", "This is a test comment").await;
    assert!(result.is_ok());
    
    let comment = result.unwrap();
    assert_eq!(comment.body, "This is a test comment");
    assert_eq!(comment.author.display_name, "Test User");
}

#[tokio::test]
async fn test_get_comments_success() {
    let mut server = mockito::Server::new_async().await;
    let base_url = server.url();
    
    let mock_response = json!({
        "comments": [
            {
                "id": "12345",
                "body": "First comment",
                "author": {
                    "accountId": "12345:abcdef",
                    "displayName": "Test User",
                    "emailAddress": "test@example.com",
                    "active": true,
                    "timeZone": "UTC"
                },
                "created": "2023-01-01T00:00:00.000Z"
            },
            {
                "id": "12346",
                "body": "Second comment",
                "author": {
                    "accountId": "12345:abcdef",
                    "displayName": "Test User",
                    "emailAddress": "test@example.com",
                    "active": true,
                    "timeZone": "UTC"
                },
                "created": "2023-01-01T01:00:00.000Z"
            }
        ]
    });
    
    let _mock = server
        .mock("GET", "/rest/api/2/issue/TEST-123/comment")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();
    
    let config = create_test_config(&format!("{}/rest/api/2", base_url));
    let client = JiraClient::new(config).expect("Failed to create JiraClient");
    
    let result = client.get_comments("TEST-123").await;
    assert!(result.is_ok());
    
    let comments = result.unwrap();
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].body, "First comment");
    assert_eq!(comments[1].body, "Second comment");
}

#[tokio::test]
async fn test_get_transitions_success() {
    let mut server = mockito::Server::new_async().await;
    let base_url = server.url();
    
    let mock_response = json!({
        "transitions": [
            {
                "id": "11",
                "name": "To Do",
                "to": {
                    "id": "1",
                    "name": "To Do",
                    "description": "The issue is open and ready for the assignee to start work on it",
                    "iconUrl": "https://test-jira.example.com/images/icons/statuses/open.png",
                    "statusCategory": {
                        "id": 1,
                        "key": "new",
                        "colorName": "blue-gray",
                        "name": "To Do"
                    }
                },
                "properties": {
                    "screen": "HasScreen",
                    "scope": "Global",
                    "availability": "Available",
                    "conditionality": "Conditional"
                }
            },
            {
                "id": "21",
                "name": "In Progress",
                "to": {
                    "id": "2",
                    "name": "In Progress",
                    "description": "This issue is being actively worked on at the moment by the assignee",
                    "iconUrl": "https://test-jira.example.com/images/icons/statuses/inprogress.png",
                    "statusCategory": {
                        "id": 2,
                        "key": "indeterminate",
                        "colorName": "yellow",
                        "name": "In Progress"
                    }
                },
                "properties": {
                    "screen": "NoScreen",
                    "scope": "Local",
                    "availability": "Unavailable",
                    "conditionality": "Unconditional"
                }
            }
        ]
    });
    
    let _mock = server
        .mock("GET", "/rest/api/2/issue/TEST-123/transitions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();
    
    let config = create_test_config(&format!("{}/rest/api/2", base_url));
    let client = JiraClient::new(config).expect("Failed to create JiraClient");
    
    let result = client.get_transitions("TEST-123").await;
    assert!(result.is_ok());
    
    let transitions = result.unwrap();
    assert_eq!(transitions.len(), 2);
    assert_eq!(transitions[0].name, "To Do");
    assert_eq!(transitions[1].name, "In Progress");
}

#[tokio::test]
async fn test_transition_issue_success() {
    let mut server = mockito::Server::new_async().await;
    let base_url = server.url();
    
    let _mock = server
        .mock("POST", "/rest/api/2/issue/TEST-123/transitions")
        .with_status(204)
        .create();
    
    let config = create_test_config(&format!("{}/rest/api/2", base_url));
    let client = JiraClient::new(config).expect("Failed to create JiraClient");
    
    let result = client.transition_issue("TEST-123", "21", Some("Moving to In Progress")).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_retry_logic_on_server_error() {
    let mut server = mockito::Server::new_async().await;
    let base_url = server.url();
    
    // First request returns 500, second returns 200
    let mock_response = json!({
        "id": "12345",
        "key": "TEST-123",
        "self": format!("{}/rest/api/2/issue/12345", base_url),
        "fields": {
            "summary": "Test Issue"
        }
    });
    
    let _mock = server
        .mock("GET", "/rest/api/2/issue/TEST-123")
        .with_status(500)
        .times(1)
        .create();
    
    let _mock2 = server
        .mock("GET", "/rest/api/2/issue/TEST-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .times(1)
        .create();
    
    let config = create_test_config(&format!("{}/rest/api/2", base_url));
    let client = JiraClient::new(config).expect("Failed to create JiraClient");
    
    let result = client.get_issue("TEST-123").await;
    assert!(result.is_ok());
    
    let issue = result.unwrap();
    assert_eq!(issue.key, "TEST-123");
}

#[tokio::test]
async fn test_authentication_error() {
    let mut server = mockito::Server::new_async().await;
    let base_url = server.url();
    
    let mock_response = json!({
        "errorMessages": ["You do not have permission to view this issue"],
        "errors": {}
    });
    
    let _mock = server
        .mock("GET", "/rest/api/2/issue/TEST-123")
        .with_status(403)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();
    
    let config = create_test_config(&format!("{}/rest/api/2", base_url));
    let client = JiraClient::new(config).expect("Failed to create JiraClient");
    
    let result = client.get_issue("TEST-123").await;
    assert!(result.is_err());
    
    if let Err(error) = result {
        assert!(error.to_string().contains("You do not have permission"));
    }
}

#[tokio::test]
async fn test_rate_limiting() {
    let mut server = mockito::Server::new_async().await;
    let base_url = server.url();
    
    let mock_response = json!({
        "id": "12345",
        "key": "TEST-123",
        "self": format!("{}/rest/api/2/issue/12345", base_url),
        "fields": {
            "summary": "Test Issue"
        }
    });
    
    let _mock = server
        .mock("GET", "/rest/api/2/issue/TEST-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .times(2)
        .create();
    
    let config = create_test_config(&format!("{}/rest/api/2", base_url));
    let client = JiraClient::new(config).expect("Failed to create JiraClient");
    
    // Make two requests quickly - the second should be rate limited
    let start = std::time::Instant::now();
    let result1 = client.get_issue("TEST-123").await;
    let result2 = client.get_issue("TEST-123").await;
    let duration = start.elapsed();
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    
    // The second request should have taken at least 100ms due to rate limiting
    assert!(duration.as_millis() >= 100);
}
