#!/usr/bin/env python3
"""
Mock data generator for Jira MCP tests.
Creates realistic test data without hitting the live API.
"""

import json
import random
from datetime import datetime, timedelta
from typing import Dict, Any, List

def generate_mock_user() -> Dict[str, Any]:
    """Generate a mock Jira user."""
    user_id = f"test_user_{random.randint(1000, 9999)}"
    return {
        "self": f"https://jira.corp.adobe.com/rest/api/2/user?username={user_id}",
        "name": user_id,
        "key": user_id,
        "emailAddress": "test@example.com",
        "avatarUrls": {
            "48x48": f"https://jira.corp.adobe.com/secure/useravatar?ownerId={user_id}&avatarId=12345",
            "24x24": f"https://jira.corp.adobe.com/secure/useravatar?size=small&ownerId={user_id}&avatarId=12345",
            "16x16": f"https://jira.corp.adobe.com/secure/useravatar?size=xsmall&ownerId={user_id}&avatarId=12345",
            "32x32": f"https://jira.corp.adobe.com/secure/useravatar?size=medium&ownerId={user_id}&avatarId=12345"
        },
        "displayName": f"Test User {random.randint(1, 100)}",
        "active": True,
        "timeZone": "America/Los_Angeles"
    }

def generate_mock_issue(issue_key: str, project_key: str = "DNA") -> Dict[str, Any]:
    """Generate a mock Jira issue."""
    statuses = ["Backlog", "In Progress", "Done", "To Do", "In Review"]
    priorities = ["Low", "Normal", "High", "Critical"]
    issue_types = ["Story", "Bug", "Task", "Epic"]
    
    created_date = datetime.now() - timedelta(days=random.randint(1, 365))
    updated_date = created_date + timedelta(days=random.randint(0, 30))
    
    return {
        "expand": "renderedFields,names,schema,operations,editmeta,changelog,versionedRepresentations",
        "id": str(random.randint(10000000, 99999999)),
        "self": f"https://jira.corp.adobe.com/rest/api/2/issue/{random.randint(10000000, 99999999)}",
        "key": issue_key,
        "fields": {
            "summary": f"Test {issue_types[random.randint(0, len(issue_types)-1)]} for {project_key} project",
            "description": f"This is a test issue for the {project_key} project. It contains mock data for testing purposes.",
            "issuetype": {
                "self": "https://jira.corp.adobe.com/rest/api/2/issuetype/7",
                "id": "7",
                "description": "Created by Jira Software - do not edit or delete. Issue type for a user story.",
                "iconUrl": "https://jira.corp.adobe.com/secure/viewavatar?size=xsmall&avatarId=18815&avatarType=issuetype",
                "name": issue_types[random.randint(0, len(issue_types)-1)],
                "subtask": False,
                "avatarId": 18815
            },
            "project": {
                "self": f"https://jira.corp.adobe.com/rest/api/2/project/{random.randint(10000, 99999)}",
                "id": str(random.randint(10000, 99999)),
                "key": project_key,
                "name": f"Test {project_key} Project",
                "projectTypeKey": "software",
                "avatarUrls": {
                    "48x48": f"https://jira.corp.adobe.com/secure/projectavatar?pid={random.randint(10000, 99999)}&avatarId=147103",
                    "24x24": f"https://jira.corp.adobe.com/secure/projectavatar?size=small&pid={random.randint(10000, 99999)}&avatarId=147103",
                    "16x16": f"https://jira.corp.adobe.com/secure/projectavatar?size=xsmall&pid={random.randint(10000, 99999)}&avatarId=147103",
                    "32x32": f"https://jira.corp.adobe.com/secure/projectavatar?size=medium&pid={random.randint(10000, 99999)}&avatarId=147103"
                }
            },
            "priority": {
                "self": "https://jira.corp.adobe.com/rest/api/2/priority/8",
                "iconUrl": "https://jira.corp.adobe.com/images/icons/priorities/normal.png",
                "name": priorities[random.randint(0, len(priorities)-1)],
                "id": "8"
            },
            "status": {
                "self": "https://jira.corp.adobe.com/rest/api/2/status/10019",
                "description": "The issue is open and ready for the assignee to start work on it.",
                "iconUrl": "https://jira.corp.adobe.com/images/icons/statuses/visible.png",
                "name": statuses[random.randint(0, len(statuses)-1)],
                "id": "10019",
                "statusCategory": {
                    "self": "https://jira.corp.adobe.com/rest/api/2/statuscategory/2",
                    "id": 2,
                    "key": "new",
                    "colorName": "default",
                    "name": "To Do"
                }
            },
            "assignee": generate_mock_user() if random.random() > 0.3 else None,
            "reporter": generate_mock_user(),
            "creator": generate_mock_user(),
            "created": created_date.isoformat() + "+0000",
            "updated": updated_date.isoformat() + "+0000",
            "labels": [f"test-label-{i}" for i in range(random.randint(0, 3))],
            "components": [],
            "fixVersions": [],
            "versions": [],
            "issuelinks": [],
            "subtasks": [],
            "watches": {
                "self": f"https://jira.corp.adobe.com/rest/api/2/issue/{issue_key}/watchers",
                "watchCount": random.randint(0, 5),
                "isWatching": False
            },
            "votes": {
                "self": f"https://jira.corp.adobe.com/rest/api/2/issue/{issue_key}/votes",
                "votes": random.randint(0, 10),
                "hasVoted": False
            },
            "worklog": {
                "startAt": 0,
                "maxResults": 20,
                "total": random.randint(0, 5),
                "worklogs": []
            },
            "comment": {
                "comments": [],
                "maxResults": 0,
                "total": random.randint(0, 3),
                "startAt": 0
            },
            "progress": {
                "progress": random.randint(0, 100),
                "total": 100
            },
            "aggregateprogress": {
                "progress": random.randint(0, 100),
                "total": 100
            },
            "timeestimate": random.randint(0, 40) * 3600 if random.random() > 0.5 else None,
            "timeoriginalestimate": random.randint(0, 40) * 3600 if random.random() > 0.5 else None,
            "timespent": random.randint(0, 20) * 3600 if random.random() > 0.5 else None,
            "aggregatetimeestimate": random.randint(0, 40) * 3600 if random.random() > 0.5 else None,
            "aggregatetimespent": random.randint(0, 20) * 3600 if random.random() > 0.5 else None,
            "workratio": random.randint(-1, 100),
            "resolution": None,
            "resolutiondate": None,
            "duedate": None,
            "environment": None,
            "attachment": [],
            "issuelinks": [],
            "votes": {
                "self": f"https://jira.corp.adobe.com/rest/api/2/issue/{issue_key}/votes",
                "votes": random.randint(0, 10),
                "hasVoted": False
            }
        }
    }

def generate_mock_search_result(project_key: str = "DNA", num_issues: int = 5) -> Dict[str, Any]:
    """Generate a mock search result."""
    issues = []
    for i in range(num_issues):
        issue_key = f"{project_key}-{random.randint(1000, 9999)}"
        issues.append(generate_mock_issue(issue_key, project_key))
    
    return {
        "expand": "schema,names",
        "startAt": 0,
        "maxResults": num_issues,
        "total": random.randint(num_issues, num_issues * 10),
        "issues": issues
    }

def generate_mock_project(project_key: str = "DNA") -> Dict[str, Any]:
    """Generate a mock Jira project."""
    return {
        "expand": "description,lead,url,projectKeys",
        "self": f"https://jira.corp.adobe.com/rest/api/2/project/{random.randint(10000, 99999)}",
        "id": str(random.randint(10000, 99999)),
        "key": project_key,
        "description": f"Test project for {project_key} - contains mock data for testing",
        "lead": generate_mock_user(),
        "components": [
            {
                "self": f"https://jira.corp.adobe.com/rest/api/2/component/{random.randint(100000, 999999)}",
                "id": str(random.randint(100000, 999999)),
                "name": f"Test Component {i+1}",
                "isAssigneeTypeValid": False
            }
            for i in range(random.randint(2, 5))
        ],
        "issueTypes": [
            {
                "self": "https://jira.corp.adobe.com/rest/api/2/issuetype/1",
                "id": "1",
                "description": "A problem which impairs or prevents the functions of the product.",
                "iconUrl": "https://jira.corp.adobe.com/secure/viewavatar?size=xsmall&avatarId=18803&avatarType=issuetype",
                "name": "Bug",
                "subtask": False,
                "avatarId": 18803
            },
            {
                "self": "https://jira.corp.adobe.com/rest/api/2/issuetype/7",
                "id": "7",
                "description": "Created by Jira Software - do not edit or delete. Issue type for a user story.",
                "iconUrl": "https://jira.corp.adobe.com/secure/viewavatar?size=xsmall&avatarId=18815&avatarType=issuetype",
                "name": "Story",
                "subtask": False,
                "avatarId": 18815
            }
        ],
        "url": f"https://test-{project_key.lower()}.corp.adobe.com",
        "assigneeType": "UNASSIGNED",
        "versions": [],
        "projectTypeKey": "software",
        "avatarUrls": {
            "48x48": f"https://jira.corp.adobe.com/secure/projectavatar?pid={random.randint(10000, 99999)}&avatarId=147103",
            "24x24": f"https://jira.corp.adobe.com/secure/projectavatar?size=small&pid={random.randint(10000, 99999)}&avatarId=147103",
            "16x16": f"https://jira.corp.adobe.com/secure/projectavatar?size=xsmall&pid={random.randint(10000, 99999)}&avatarId=147103",
            "32x32": f"https://jira.corp.adobe.com/secure/projectavatar?size=medium&pid={random.randint(10000, 99999)}&avatarId=147103"
        }
    }

def generate_mock_test_data() -> Dict[str, Any]:
    """Generate complete mock test data."""
    return {
        "collection_timestamp": datetime.now().isoformat(),
        "description": "Mock test data generated for Jira MCP testing",
        "operations": {
            "authentication": {
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "content": [
                        {
                            "text": "Authentication test successful",
                            "type": "text"
                        }
                    ],
                    "is_error": False
                }
            },
            "project_config": {
                "jsonrpc": "2.0",
                "id": 2,
                "result": {
                    "content": [
                        {
                            "text": json.dumps(generate_mock_project("DNA"), indent=2),
                            "type": "text"
                        }
                    ],
                    "is_error": False
                }
            },
            "get_issue": {
                "jsonrpc": "2.0",
                "id": 3,
                "result": {
                    "content": [
                        {
                            "text": json.dumps(generate_mock_issue("DNA-1244", "DNA"), indent=2),
                            "type": "text"
                        }
                    ],
                    "is_error": False
                }
            },
            "search_issues": {
                "jsonrpc": "2.0",
                "id": 4,
                "result": {
                    "content": [
                        {
                            "text": json.dumps(generate_mock_search_result("DNA", 5), indent=2),
                            "type": "text"
                        }
                    ],
                    "is_error": False
                }
            },
            "project_components": {
                "jsonrpc": "2.0",
                "id": 5,
                "result": {
                    "content": [
                        {
                            "text": json.dumps([
                                {
                                    "self": f"https://jira.corp.adobe.com/rest/api/2/component/{random.randint(100000, 999999)}",
                                    "id": str(random.randint(100000, 999999)),
                                    "name": f"Test Component {i+1}",
                                    "isAssigneeTypeValid": False
                                }
                                for i in range(random.randint(2, 5))
                            ], indent=2),
                            "type": "text"
                        }
                    ],
                    "is_error": False
                }
            },
            "issue_types": {
                "jsonrpc": "2.0",
                "id": 6,
                "result": {
                    "content": [
                        {
                            "text": json.dumps([
                                {
                                    "self": "https://jira.corp.adobe.com/rest/api/2/issuetype/1",
                                    "id": "1",
                                    "description": "A problem which impairs or prevents the functions of the product.",
                                    "iconUrl": "https://jira.corp.adobe.com/secure/viewavatar?size=xsmall&avatarId=18803&avatarType=issuetype",
                                    "name": "Bug",
                                    "subtask": False,
                                    "avatarId": 18803
                                },
                                {
                                    "self": "https://jira.corp.adobe.com/rest/api/2/issuetype/7",
                                    "id": "7",
                                    "description": "Created by Jira Software - do not edit or delete. Issue type for a user story.",
                                    "iconUrl": "https://jira.corp.adobe.com/secure/viewavatar?size=xsmall&avatarId=18815&avatarType=issuetype",
                                    "name": "Story",
                                    "subtask": False,
                                    "avatarId": 18815
                                }
                            ], indent=2),
                            "type": "text"
                        }
                    ],
                    "is_error": False
                }
            }
        }
    }

def main():
    """Generate and save mock test data."""
    print("🎭 Generating mock test data...")
    
    mock_data = generate_mock_test_data()
    
    # Save to fixtures directory
    import os
    os.makedirs("tests/fixtures", exist_ok=True)
    
    filename = "tests/fixtures/jira_mock_data.json"
    with open(filename, 'w') as f:
        json.dump(mock_data, f, indent=2)
    
    print(f"✅ Mock test data generated and saved to: {filename}")
    print(f"📊 Generated {len(mock_data['operations'])} operation responses")
    
    # Also create a simple version for basic tests
    simple_filename = "tests/fixtures/simple_mock_data.json"
    simple_data = {
        "project": generate_mock_project("DNA"),
        "issue": generate_mock_issue("DNA-1244", "DNA"),
        "search_result": generate_mock_search_result("DNA", 3)
    }
    
    with open(simple_filename, 'w') as f:
        json.dump(simple_data, f, indent=2)
    
    print(f"📁 Also saved simple version as: {simple_filename}")

if __name__ == "__main__":
    main()
