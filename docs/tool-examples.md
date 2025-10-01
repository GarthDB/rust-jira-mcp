# Tool Usage Examples

This document provides comprehensive examples for all MCP tools available in the Rust Jira MCP Server.

## Table of Contents

- [Authentication Tools](#authentication-tools)
- [Issue Management Tools](#issue-management-tools)
- [Project Configuration Tools](#project-configuration-tools)
- [Bulk Operations Tools](#bulk-operations-tools)
- [Zephyr Test Management Tools](#zephyr-test-management-tools)
- [Advanced Usage Patterns](#advanced-usage-patterns)

## Authentication Tools

### test_jira_auth

Test your Jira API authentication and connection.

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "test_jira_auth",
    "arguments": {}
  }
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "✅ Authentication successful!\n\nConnection Details:\n- Jira URL: https://your-company.atlassian.net\n- User: your.email@company.com\n- API Version: 2\n- Response Time: 245ms"
    }
  ]
}
```

**Use Cases:**
- Verify credentials before running other operations
- Health check for monitoring
- Debugging connection issues

## Issue Management Tools

### search_jira_issues

Search for issues using JQL (Jira Query Language).

**Basic Search:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "search_jira_issues",
    "arguments": {
      "jql": "project = PROJ AND status = Open",
      "max_results": 10
    }
  }
}
```

**Advanced Search with Fields:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "search_jira_issues",
    "arguments": {
      "jql": "project = PROJ AND assignee = currentUser() AND updated >= -7d",
      "max_results": 50,
      "fields": ["summary", "status", "assignee", "priority", "created", "updated"]
    }
  }
}
```

**Complex JQL Examples:**
```json
// Find issues assigned to me in the last week
{
  "method": "tools/call",
  "params": {
    "name": "search_jira_issues",
    "arguments": {
      "jql": "assignee = currentUser() AND updated >= -7d ORDER BY updated DESC",
      "max_results": 25
    }
  }
}

// Find high priority bugs
{
  "method": "tools/call",
  "params": {
    "name": "search_jira_issues",
    "arguments": {
      "jql": "project = PROJ AND issuetype = Bug AND priority = High ORDER BY created DESC",
      "max_results": 20
    }
  }
}

// Find issues with specific components
{
  "method": "tools/call",
  "params": {
    "name": "search_jira_issues",
    "arguments": {
      "jql": "project = PROJ AND component = 'Backend' AND status != Done",
      "max_results": 30
    }
  }
}
```

### create_jira_issue

Create new issues in Jira.

**Basic Issue Creation:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "create_jira_issue",
    "arguments": {
      "project_key": "PROJ",
      "issue_type": "Task",
      "summary": "Implement user authentication",
      "description": "Add OAuth2 authentication to the user login system."
    }
  }
}
```

**Advanced Issue Creation:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "create_jira_issue",
    "arguments": {
      "project_key": "PROJ",
      "issue_type": "Story",
      "summary": "As a user, I want to reset my password",
      "description": "## User Story\n\nAs a registered user, I want to be able to reset my password when I forget it, so that I can regain access to my account.\n\n## Acceptance Criteria\n\n- [ ] User can request password reset via email\n- [ ] Reset link expires after 24 hours\n- [ ] User can set new password via reset link\n- [ ] User is logged out of all sessions after password reset",
      "fields": {
        "priority": "Medium",
        "assignee": "john.doe@company.com",
        "labels": ["authentication", "security", "user-management"],
        "components": ["Backend", "Frontend"],
        "fixVersions": ["v2.1.0"]
      }
    }
  }
}
```

**Bug Report Creation:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "create_jira_issue",
    "arguments": {
      "project_key": "PROJ",
      "issue_type": "Bug",
      "summary": "Login button not responding on mobile devices",
      "description": "## Bug Description\n\nThe login button on the mobile app is not responding when tapped.\n\n## Steps to Reproduce\n\n1. Open the mobile app\n2. Navigate to the login screen\n3. Tap the 'Login' button\n4. Observe that nothing happens\n\n## Expected Behavior\n\nUser should be redirected to the authentication page.\n\n## Environment\n\n- Device: iPhone 12, Android 11\n- App Version: 2.0.1\n- Browser: Safari, Chrome Mobile",
      "fields": {
        "priority": "High",
        "labels": ["mobile", "ui", "critical"],
        "components": ["Mobile App"]
      }
    }
  }
}
```

### update_jira_issue

Update existing issues.

**Basic Update:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "update_jira_issue",
    "arguments": {
      "issue_key": "PROJ-123",
      "fields": {
        "summary": "Updated issue summary",
        "description": "Updated description with more details."
      }
    }
  }
}
```

**Status and Assignee Update:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "update_jira_issue",
    "arguments": {
      "issue_key": "PROJ-123",
      "fields": {
        "assignee": "jane.doe@company.com",
        "priority": "High",
        "labels": ["urgent", "backend"]
      }
    }
  }
}
```

### get_jira_issue

Get detailed information about a specific issue.

**Basic Issue Retrieval:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_jira_issue",
    "arguments": {
      "issue_key": "PROJ-123"
    }
  }
}
```

**Issue with Specific Fields:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_jira_issue",
    "arguments": {
      "issue_key": "PROJ-123",
      "fields": ["summary", "status", "assignee", "priority", "description", "comments"]
    }
  }
}
```

### get_jira_comments

Retrieve comments for an issue.

**Get All Comments:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_jira_comments",
    "arguments": {
      "issue_key": "PROJ-123"
    }
  }
}
```

**Get Comments with Pagination:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_jira_comments",
    "arguments": {
      "issue_key": "PROJ-123",
      "max_results": 20,
      "start_at": 0
    }
  }
}
```

### add_jira_comment

Add comments to issues.

**Basic Comment:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "add_jira_comment",
    "arguments": {
      "issue_key": "PROJ-123",
      "body": "This issue has been reviewed and approved for implementation."
    }
  }
}
```

**Rich Text Comment:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "add_jira_comment",
    "arguments": {
      "issue_key": "PROJ-123",
      "body": "## Code Review Results\n\n✅ **Approved**\n\n### Changes Made:\n- Fixed authentication logic\n- Added input validation\n- Updated error handling\n\n### Next Steps:\n1. Deploy to staging\n2. Run integration tests\n3. Schedule production deployment"
    }
  }
}
```

### get_jira_transitions

Get available transitions for an issue.

**Get All Transitions:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_jira_transitions",
    "arguments": {
      "issue_key": "PROJ-123"
    }
  }
}
```

### transition_jira_issue

Transition an issue to a new status.

**Basic Transition:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "transition_jira_issue",
    "arguments": {
      "issue_key": "PROJ-123",
      "transition_id": "31"
    }
  }
}
```

**Transition with Comment:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "transition_jira_issue",
    "arguments": {
      "issue_key": "PROJ-123",
      "transition_id": "31",
      "comment": "Moving to In Progress - development has started."
    }
  }
}
```

**Transition with Fields:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "transition_jira_issue",
    "arguments": {
      "issue_key": "PROJ-123",
      "transition_id": "31",
      "fields": {
        "assignee": "john.doe@company.com",
        "resolution": "Fixed"
      }
    }
  }
}
```

## Project Configuration Tools

### get_project_config

Get detailed project configuration.

**Basic Project Configuration:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_project_config",
    "arguments": {
      "project_key": "PROJ"
    }
  }
}
```

### get_project_issue_types

Get available issue types for a project.

**Get All Issue Types:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_project_issue_types",
    "arguments": {
      "project_key": "PROJ"
    }
  }
}
```

### get_issue_type_metadata

Get detailed information about a specific issue type.

**Get Issue Type Details:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_issue_type_metadata",
    "arguments": {
      "project_key": "PROJ",
      "issue_type_id": "10001"
    }
  }
}
```

### get_project_components

Get components associated with a project.

**Get All Components:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_project_components",
    "arguments": {
      "project_key": "PROJ"
    }
  }
}
```

### get_priorities_and_statuses

Get all available priorities and statuses.

**Get All Priorities and Statuses:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_priorities_and_statuses",
    "arguments": {}
  }
}
```

### get_custom_fields

Get custom field definitions.

**Get All Custom Fields:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_custom_fields",
    "arguments": {}
  }
}
```

### get_project_metadata

Get comprehensive project metadata in a single call.

**Get Complete Project Information:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_project_metadata",
    "arguments": {
      "project_key": "PROJ"
    }
  }
}
```

## Bulk Operations Tools

### bulk_create_issues

Create multiple issues at once.

**Create Multiple Issues:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "bulk_create_issues",
    "arguments": {
      "issues": [
        {
          "project_key": "PROJ",
          "issue_type": "Task",
          "summary": "Task 1: Implement user authentication",
          "description": "Add OAuth2 authentication to the system."
        },
        {
          "project_key": "PROJ",
          "issue_type": "Task",
          "summary": "Task 2: Add password reset functionality",
          "description": "Implement password reset via email."
        },
        {
          "project_key": "PROJ",
          "issue_type": "Bug",
          "summary": "Bug 1: Fix login button on mobile",
          "description": "Login button not responding on mobile devices."
        }
      ]
    }
  }
}
```

### bulk_update_issues

Update multiple issues at once.

**Update Multiple Issues:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "bulk_update_issues",
    "arguments": {
      "updates": [
        {
          "issue_key": "PROJ-123",
          "fields": {
            "priority": "High",
            "assignee": "john.doe@company.com"
          }
        },
        {
          "issue_key": "PROJ-124",
          "fields": {
            "priority": "Medium",
            "labels": ["backend", "api"]
          }
        }
      ]
    }
  }
}
```

### bulk_transition_issues

Transition multiple issues at once.

**Transition Multiple Issues:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "bulk_transition_issues",
    "arguments": {
      "transitions": [
        {
          "issue_key": "PROJ-123",
          "transition_id": "31",
          "comment": "Moving to In Progress"
        },
        {
          "issue_key": "PROJ-124",
          "transition_id": "31",
          "comment": "Starting development"
        }
      ]
    }
  }
}
```

### bulk_add_comments

Add comments to multiple issues.

**Add Comments to Multiple Issues:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "bulk_add_comments",
    "arguments": {
      "comments": [
        {
          "issue_key": "PROJ-123",
          "body": "Code review completed - approved for merge."
        },
        {
          "issue_key": "PROJ-124",
          "body": "Testing completed - ready for production."
        }
      ]
    }
  }
}
```

## Zephyr Test Management Tools

### get_test_cycles

Get test cycles for a project.

**Get All Test Cycles:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_test_cycles",
    "arguments": {
      "project_key": "PROJ"
    }
  }
}
```

**Get Test Cycles with Filters:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_test_cycles",
    "arguments": {
      "project_key": "PROJ",
      "version": "v2.1.0",
      "status": "ACTIVE"
    }
  }
}
```

### create_test_cycle

Create a new test cycle.

**Create Test Cycle:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "create_test_cycle",
    "arguments": {
      "project_key": "PROJ",
      "name": "Sprint 15 Testing",
      "description": "Test cycle for Sprint 15 features",
      "version": "v2.1.0"
    }
  }
}
```

### get_test_executions

Get test executions for a test cycle.

**Get Test Executions:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_test_executions",
    "arguments": {
      "test_cycle_id": "12345"
    }
  }
}
```

### update_test_execution

Update test execution status.

**Update Test Execution:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "update_test_execution",
    "arguments": {
      "execution_id": "67890",
      "status": "PASS",
      "comment": "All test cases passed successfully."
    }
  }
}
```

## Advanced Usage Patterns

### Workflow Automation

**Complete Issue Lifecycle:**
```json
// 1. Create issue
{
  "method": "tools/call",
  "params": {
    "name": "create_jira_issue",
    "arguments": {
      "project_key": "PROJ",
      "issue_type": "Story",
      "summary": "Implement new feature",
      "description": "Add new feature to the application."
    }
  }
}

// 2. Assign and transition
{
  "method": "tools/call",
  "params": {
    "name": "update_jira_issue",
    "arguments": {
      "issue_key": "PROJ-123",
      "fields": {
        "assignee": "developer@company.com"
      }
    }
  }
}

// 3. Transition to In Progress
{
  "method": "tools/call",
  "params": {
    "name": "transition_jira_issue",
    "arguments": {
      "issue_key": "PROJ-123",
      "transition_id": "31",
      "comment": "Starting development work."
    }
  }
}

// 4. Add progress comment
{
  "method": "tools/call",
  "params": {
    "name": "add_jira_comment",
    "arguments": {
      "issue_key": "PROJ-123",
      "body": "Development completed. Ready for testing."
    }
  }
}

// 5. Transition to Done
{
  "method": "tools/call",
  "params": {
    "name": "transition_jira_issue",
    "arguments": {
      "issue_key": "PROJ-123",
      "transition_id": "41",
      "comment": "Feature implemented and tested."
    }
  }
}
```

### Project Analysis

**Comprehensive Project Analysis:**
```json
// 1. Get project metadata
{
  "method": "tools/call",
  "params": {
    "name": "get_project_metadata",
    "arguments": {
      "project_key": "PROJ"
    }
  }
}

// 2. Get all priorities and statuses
{
  "method": "tools/call",
  "params": {
    "name": "get_priorities_and_statuses",
    "arguments": {}
  }
}

// 3. Search for issues by status
{
  "method": "tools/call",
  "params": {
    "name": "search_jira_issues",
    "arguments": {
      "jql": "project = PROJ ORDER BY status",
      "max_results": 100
    }
  }
}
```

### Bulk Operations for Sprint Management

**Sprint Setup:**
```json
// Create multiple sprint tasks
{
  "method": "tools/call",
  "params": {
    "name": "bulk_create_issues",
    "arguments": {
      "issues": [
        {
          "project_key": "PROJ",
          "issue_type": "Story",
          "summary": "Sprint 15: User Authentication",
          "description": "Implement user authentication system"
        },
        {
          "project_key": "PROJ",
          "issue_type": "Task",
          "summary": "Sprint 15: Database Schema",
          "description": "Design and implement user database schema"
        },
        {
          "project_key": "PROJ",
          "issue_type": "Task",
          "summary": "Sprint 15: API Endpoints",
          "description": "Create authentication API endpoints"
        }
      ]
    }
  }
}

// Assign all tasks to team members
{
  "method": "tools/call",
  "params": {
    "name": "bulk_update_issues",
    "arguments": {
      "updates": [
        {
          "issue_key": "PROJ-123",
          "fields": {
            "assignee": "backend-dev@company.com",
            "priority": "High"
          }
        },
        {
          "issue_key": "PROJ-124",
          "fields": {
            "assignee": "database-dev@company.com",
            "priority": "High"
          }
        },
        {
          "issue_key": "PROJ-125",
          "fields": {
            "assignee": "api-dev@company.com",
            "priority": "Medium"
          }
        }
      ]
    }
  }
}
```

## Sprint Management Tools

### get_sprint

Get sprint details by sprint ID.

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_sprint",
    "arguments": {
      "sprint_id": 12345
    }
  }
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Sprint: 12345\nName: Sprint 15 - User Authentication\nState: Active\nStart Date: 2025-10-01T00:00:00.000Z\nEnd Date: 2025-10-15T00:00:00.000Z\nGoal: Implement user authentication system\nURL: https://your-company.atlassian.net/secure/RapidBoard.jspa?rapidView=123&view=planning.nodetail&selectedIssue=12345"
    }
  ]
}
```

### create_sprint

Create a new sprint.

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "create_sprint",
    "arguments": {
      "name": "Sprint 16 - Payment Integration",
      "rapid_view_id": 123,
      "start_date": "2025-10-16T00:00:00.000Z",
      "end_date": "2025-10-30T00:00:00.000Z",
      "goal": "Integrate payment processing system"
    }
  }
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Sprint created successfully!\nID: 12346\nName: Sprint 16 - Payment Integration\nBoard ID: 123\nURL: https://your-company.atlassian.net/secure/RapidBoard.jspa?rapidView=123&view=planning.nodetail&selectedIssue=12346"
    }
  ]
}
```

### add_issues_to_sprint

Add issues to a sprint.

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "add_issues_to_sprint",
    "arguments": {
      "sprint_id": 12345,
      "issues": ["PROJ-123", "PROJ-124", "PROJ-125"]
    }
  }
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Added 3 issues to sprint 12345\n"
    }
  ]
}
```

### get_sprint_issues

Get all issues in a sprint.

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_sprint_issues",
    "arguments": {
      "sprint_id": 12345,
      "max_results": 50
    }
  }
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Found 3 issues in sprint 12345 (showing 3 of 3 total)\n\n"
    },
    {
      "type": "text",
      "text": "• PROJ-123 - Implement user login\n  Status: In Progress\n  Assignee: John Doe\n  URL: https://your-company.atlassian.net/browse/PROJ-123\n"
    },
    {
      "type": "text",
      "text": "• PROJ-124 - Add password reset\n  Status: To Do\n  Assignee: Jane Smith\n  URL: https://your-company.atlassian.net/browse/PROJ-124\n"
    },
    {
      "type": "text",
      "text": "• PROJ-125 - Update user profile\n  Status: Done\n  Assignee: Bob Johnson\n  URL: https://your-company.atlassian.net/browse/PROJ-125\n"
    }
  ]
}
```

### start_sprint

Start a sprint (set state to active).

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "start_sprint",
    "arguments": {
      "sprint_id": 12345
    }
  }
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Sprint started successfully!\nID: 12345\nName: Sprint 15 - User Authentication\nState: Active\nURL: https://your-company.atlassian.net/secure/RapidBoard.jspa?rapidView=123&view=planning.nodetail&selectedIssue=12345"
    }
  ]
}
```

### close_sprint

Close a sprint (set state to closed).

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "close_sprint",
    "arguments": {
      "sprint_id": 12345
    }
  }
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Sprint closed successfully!\nID: 12345\nName: Sprint 15 - User Authentication\nState: Closed\nURL: https://your-company.atlassian.net/secure/RapidBoard.jspa?rapidView=123&view=planning.nodetail&selectedIssue=12345"
    }
  ]
}
```

### get_board_sprints

Get all sprints for a board (rapid view).

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_board_sprints",
    "arguments": {
      "rapid_view_id": 123
    }
  }
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Found 3 sprints for board 123\n\n"
    },
    {
      "type": "text",
      "text": "• 12345 - Sprint 15 - User Authentication\n  State: Closed\n  Start: 2025-09-15T00:00:00.000Z\n  End: 2025-09-29T00:00:00.000Z\n  Goal: Implement user authentication system\n  URL: https://your-company.atlassian.net/secure/RapidBoard.jspa?rapidView=123&view=planning.nodetail&selectedIssue=12345\n"
    },
    {
      "type": "text",
      "text": "• 12346 - Sprint 16 - Payment Integration\n  State: Active\n  Start: 2025-10-01T00:00:00.000Z\n  End: 2025-10-15T00:00:00.000Z\n  Goal: Integrate payment processing system\n  URL: https://your-company.atlassian.net/secure/RapidBoard.jspa?rapidView=123&view=planning.nodetail&selectedIssue=12346\n"
    },
    {
      "type": "text",
      "text": "• 12347 - Sprint 17 - Mobile App\n  State: Future\n  Start: Not set\n  End: Not set\n  Goal: No goal set\n  URL: https://your-company.atlassian.net/secure/RapidBoard.jspa?rapidView=123&view=planning.nodetail&selectedIssue=12347\n"
    }
  ]
}
```

### Sprint Management Workflow Examples

**Complete Sprint Planning Workflow:**
```json
// 1. Get all sprints for a board
{
  "method": "tools/call",
  "params": {
    "name": "get_board_sprints",
    "arguments": {
      "rapid_view_id": 123
    }
  }
}

// 2. Create a new sprint
{
  "method": "tools/call",
  "params": {
    "name": "create_sprint",
    "arguments": {
      "name": "Sprint 18 - API Documentation",
      "rapid_view_id": 123,
      "start_date": "2025-11-01T00:00:00.000Z",
      "end_date": "2025-11-15T00:00:00.000Z",
      "goal": "Complete API documentation and examples"
    }
  }
}

// 3. Add issues to the sprint
{
  "method": "tools/call",
  "params": {
    "name": "add_issues_to_sprint",
    "arguments": {
      "sprint_id": 12348,
      "issues": ["PROJ-200", "PROJ-201", "PROJ-202"]
    }
  }
}

// 4. Start the sprint
{
  "method": "tools/call",
  "params": {
    "name": "start_sprint",
    "arguments": {
      "sprint_id": 12348
    }
  }
}
```

**Sprint Monitoring Workflow:**
```json
// 1. Get current active sprint issues
{
  "method": "tools/call",
  "params": {
    "name": "get_sprint_issues",
    "arguments": {
      "sprint_id": 12346,
      "max_results": 100
    }
  }
}

// 2. Close the sprint when complete
{
  "method": "tools/call",
  "params": {
    "name": "close_sprint",
    "arguments": {
      "sprint_id": 12346
    }
  }
}
```

## Error Handling

All tools return structured error responses when something goes wrong:

**Authentication Error:**
```json
{
  "error": {
    "code": -32603,
    "message": "Internal error",
    "data": {
      "type": "AuthenticationError",
      "message": "Invalid credentials provided",
      "details": "Please check your JIRA_EMAIL and JIRA_PERSONAL_ACCESS_TOKEN"
    }
  }
}
```

**Validation Error:**
```json
{
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "type": "ValidationError",
      "message": "Required field 'project_key' is missing",
      "field": "project_key"
    }
  }
}
```

**Jira API Error:**
```json
{
  "error": {
    "code": -32603,
    "message": "Internal error",
    "data": {
      "type": "JiraApiError",
      "message": "Issue PROJ-999 does not exist",
      "jira_error_code": "404"
    }
  }
}
```

## Best Practices

1. **Always test authentication first** before running other operations
2. **Use pagination** for large result sets to avoid timeouts
3. **Validate input** before making API calls
4. **Handle errors gracefully** and provide meaningful error messages
5. **Use bulk operations** when working with multiple issues
6. **Include comments** when transitioning issues for audit trails
7. **Use specific JQL queries** to get exactly the data you need
8. **Test with small datasets** before running on production data

## Performance Tips

1. **Limit result sets** using `max_results` parameter
2. **Use specific fields** to reduce response size
3. **Cache project metadata** to avoid repeated API calls
4. **Use bulk operations** instead of individual calls
5. **Implement retry logic** for transient failures
6. **Monitor API rate limits** and implement backoff strategies
