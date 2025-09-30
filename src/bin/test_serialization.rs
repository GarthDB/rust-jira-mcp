use rust_jira_mcp::types::jira::{JiraUser, JiraProject, JiraStatus, JiraPriority, JiraIssueType};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing serialization with real data...");

    // Test JiraUser deserialization with real data
    let user_json = fs::read_to_string("fixtures/raw/myself.json")?;
    let user: JiraUser = serde_json::from_str(&user_json)?;
    println!("✅ JiraUser deserialization successful: {}", user.display_name);

    // Test JiraProject deserialization with real data
    let project_json = fs::read_to_string("fixtures/raw/dna_project.json")?;
    let project: JiraProject = serde_json::from_str(&project_json)?;
    println!("✅ JiraProject deserialization successful: {}", project.name);

    // Test JiraStatus deserialization with real data
    let status_json = r#"{
        "self": "https://jira.corp.adobe.com/rest/api/2/status/10019",
        "description": "The issue is open and ready for the assignee to start work on it.",
        "iconUrl": "https://jira.corp.adobe.com/images/icons/statuses/visible.png",
        "name": "Backlog",
        "id": "10019",
        "statusCategory": {
            "self": "https://jira.corp.adobe.com/rest/api/2/statuscategory/2",
            "id": 2,
            "key": "new",
            "colorName": "default",
            "name": "To Do"
        }
    }"#;

    let status: JiraStatus = serde_json::from_str(status_json)?;
    println!("✅ JiraStatus deserialization successful: {}", status.name);

    // Test JiraPriority deserialization with real data
    let priority_json = r#"{
        "self": "https://jira.corp.adobe.com/rest/api/2/priority/8",
        "iconUrl": "https://jira.corp.adobe.com/images/icons/priorities/normal.png",
        "name": "Normal",
        "id": "8"
    }"#;

    let priority: JiraPriority = serde_json::from_str(priority_json)?;
    println!("✅ JiraPriority deserialization successful: {}", priority.name);

    // Test JiraIssueType deserialization with real data
    let issue_type_json = r#"{
        "self": "https://jira.corp.adobe.com/rest/api/2/issuetype/7",
        "id": "7",
        "description": "Created by Jira Software - do not edit or delete. Issue type for a user story.",
        "iconUrl": "https://jira.corp.adobe.com/secure/viewavatar?size=xsmall&avatarId=18815&avatarType=issuetype",
        "name": "Story",
        "subtask": false,
        "avatarId": 18815
    }"#;

    let issue_type: JiraIssueType = serde_json::from_str(issue_type_json)?;
    println!("✅ JiraIssueType deserialization successful: {}", issue_type.name);

    println!("🎉 All real data serialization tests passed!");
    Ok(())
}
