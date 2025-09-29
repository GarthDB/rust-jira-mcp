#![allow(clippy::format_push_string)]

use crate::config::JiraConfig;
use crate::error::Result;
use crate::jira::client::JiraClient;
use crate::types::jira::{
    ZephyrTestCaseCreateRequest, ZephyrTestExecutionCreateRequest, ZephyrTestStepCreateRequest,
    ZephyrTestStepUpdateRequest,
};
use crate::types::mcp::{MCPContent, MCPToolResult};
use tracing::info;

// Get Zephyr Test Steps Tool
pub struct GetZephyrTestStepsTool {
    client: JiraClient,
}

impl GetZephyrTestStepsTool {
    #[must_use]
    /// # Panics
    /// This function does not panic.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetZephyrTestStepsTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let test_case_id = args
            .get("test_case_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: test_case_id")
            })?;

        info!("Getting Zephyr test steps for test case: {}", test_case_id);

        let test_steps = self.client.get_zephyr_test_steps(test_case_id).await?;

        let mut response_text = format!("Test Steps for Test Case {test_case_id}:\n\n");

        if test_steps.is_empty() {
            response_text.push_str("No test steps found.");
        } else {
            for (i, step) in test_steps.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. {}\n   Order: {}\n   Data: {}\n   Expected Result: {}\n\n",
                    i + 1,
                    step.step,
                    step.order,
                    step.data.as_deref().unwrap_or("None"),
                    step.result.as_deref().unwrap_or("None")
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Create Zephyr Test Step Tool
pub struct CreateZephyrTestStepTool {
    client: JiraClient,
}

impl CreateZephyrTestStepTool {
    #[must_use]
    /// # Panics
    /// This function does not panic.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for CreateZephyrTestStepTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let test_case_id = args
            .get("test_case_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: test_case_id")
            })?;

        let step = args.get("step").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::JiraError::api_error("Missing required parameter: step")
        })?;

        let order = i32::try_from(
            args.get("order")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| {
                    crate::error::JiraError::api_error("Missing required parameter: order")
                })?,
        )
        .unwrap_or(0);

        let data = args
            .get("data")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        let result = args
            .get("result")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let test_step_request = ZephyrTestStepCreateRequest {
            step: step.to_string(),
            data,
            result,
            order,
            test_case_id: test_case_id.to_string(),
        };

        info!("Creating Zephyr test step for test case: {}", test_case_id);

        let created_step = self
            .client
            .create_zephyr_test_step(&test_step_request)
            .await?;

        let response_text = format!(
            "Test step created successfully!\n\nStep: {}\nOrder: {}\nData: {}\nExpected Result: {}\nTest Case ID: {}",
            created_step.step,
            created_step.order,
            created_step.data.as_deref().unwrap_or("None"),
            created_step.result.as_deref().unwrap_or("None"),
            created_step.test_case_id.as_deref().unwrap_or("Unknown")
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Update Zephyr Test Step Tool
pub struct UpdateZephyrTestStepTool {
    client: JiraClient,
}

impl UpdateZephyrTestStepTool {
    #[must_use]
    /// # Panics
    /// This function does not panic.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for UpdateZephyrTestStepTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let test_case_id = args
            .get("test_case_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: test_case_id")
            })?;

        let step_id = args
            .get("step_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: step_id")
            })?;

        let step = args
            .get("step")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        let data = args
            .get("data")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        let result = args
            .get("result")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        let order = args
            .get("order")
            .and_then(serde_json::Value::as_i64)
            .map(|v| i32::try_from(v).unwrap_or(0));

        let test_step_request = ZephyrTestStepUpdateRequest {
            step,
            data,
            result,
            order,
        };

        info!(
            "Updating Zephyr test step {} for test case: {}",
            step_id, test_case_id
        );

        let updated_step = self
            .client
            .update_zephyr_test_step(test_case_id, step_id, &test_step_request)
            .await?;

        let response_text = format!(
            "Test step updated successfully!\n\nStep: {}\nOrder: {}\nData: {}\nExpected Result: {}",
            updated_step.step,
            updated_step.order,
            updated_step.data.as_deref().unwrap_or("None"),
            updated_step.result.as_deref().unwrap_or("None")
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Delete Zephyr Test Step Tool
pub struct DeleteZephyrTestStepTool {
    client: JiraClient,
}

impl DeleteZephyrTestStepTool {
    #[must_use]
    /// # Panics
    /// This function does not panic.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for DeleteZephyrTestStepTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let test_case_id = args
            .get("test_case_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: test_case_id")
            })?;

        let step_id = args
            .get("step_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: step_id")
            })?;

        info!(
            "Deleting Zephyr test step {} for test case: {}",
            step_id, test_case_id
        );

        self.client
            .delete_zephyr_test_step(test_case_id, step_id)
            .await?;

        let response_text =
            format!("Test step {step_id} deleted successfully from test case {test_case_id}!");

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Get Zephyr Test Cases Tool
pub struct GetZephyrTestCasesTool {
    client: JiraClient,
}

impl GetZephyrTestCasesTool {
    #[must_use]
    /// # Panics
    /// This function does not panic.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetZephyrTestCasesTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: project_key")
            })?;

        let start_at = args
            .get("start_at")
            .and_then(serde_json::Value::as_i64)
            .map(|v| i32::try_from(v).unwrap_or(0));

        let max_results = args
            .get("max_results")
            .and_then(serde_json::Value::as_i64)
            .map(|v| i32::try_from(v).unwrap_or(0));

        info!("Getting Zephyr test cases for project: {}", project_key);

        let search_result = self
            .client
            .search_zephyr_test_cases(project_key, start_at, max_results)
            .await?;

        let mut response_text = format!(
            "Found {} test cases (showing {} of {} total)\n\n",
            search_result.test_cases.len(),
            search_result.test_cases.len(),
            search_result.total
        );

        if search_result.test_cases.is_empty() {
            response_text.push_str("No test cases found.");
        } else {
            for (i, test_case) in search_result.test_cases.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. {} - {}\n   Project: {}\n   Type: {}\n   Status: {}\n   Priority: {}\n\n",
                    i + 1,
                    test_case.key.as_deref().unwrap_or("No Key"),
                    test_case.name,
                    test_case.project_key,
                    test_case.issue_type,
                    test_case.status.as_deref().unwrap_or("Unknown"),
                    test_case.priority.as_deref().unwrap_or("Unknown")
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Create Zephyr Test Case Tool
pub struct CreateZephyrTestCaseTool {
    client: JiraClient,
}

impl CreateZephyrTestCaseTool {
    #[must_use]
    /// # Panics
    /// This function does not panic.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for CreateZephyrTestCaseTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::JiraError::api_error("Missing required parameter: name")
        })?;

        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: project_key")
            })?;

        let issue_type = args
            .get("issue_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: issue_type")
            })?;

        let priority = args
            .get("priority")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        let assignee = args
            .get("assignee")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        let description = args
            .get("description")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let test_case_request = ZephyrTestCaseCreateRequest {
            name: name.to_string(),
            project_key: project_key.to_string(),
            issue_type: issue_type.to_string(),
            priority,
            assignee,
            description,
            labels: None,
            components: None,
            fix_versions: None,
            custom_fields: None,
        };

        info!("Creating Zephyr test case: {}", name);

        let created_test_case = self
            .client
            .create_zephyr_test_case(&test_case_request)
            .await?;

        let response_text = format!(
            "Test case created successfully!\n\nName: {}\nKey: {}\nProject: {}\nType: {}\nPriority: {}\nAssignee: {}",
            created_test_case.name,
            created_test_case.key.as_deref().unwrap_or("No Key"),
            created_test_case.project_key,
            created_test_case.issue_type,
            created_test_case.priority.as_deref().unwrap_or("None"),
            created_test_case.assignee.as_deref().unwrap_or("Unassigned")
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Get Zephyr Test Executions Tool
pub struct GetZephyrTestExecutionsTool {
    client: JiraClient,
}

impl GetZephyrTestExecutionsTool {
    #[must_use]
    /// # Panics
    /// This function does not panic.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetZephyrTestExecutionsTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let test_case_id = args
            .get("test_case_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: test_case_id")
            })?;

        info!(
            "Getting Zephyr test executions for test case: {}",
            test_case_id
        );

        let test_executions = self.client.get_zephyr_test_executions(test_case_id).await?;

        let mut response_text = format!("Test Executions for Test Case {test_case_id}:\n\n");

        if test_executions.is_empty() {
            response_text.push_str("No test executions found.");
        } else {
            for (i, execution) in test_executions.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. Execution ID: {}\n   Status: {}\n   Assignee: {}\n   Executed By: {}\n   Executed On: {}\n   Comment: {}\n\n",
                    i + 1,
                    execution.id.as_deref().unwrap_or("Unknown"),
                    execution.status,
                    execution.assignee.as_deref().unwrap_or("Unassigned"),
                    execution.executed_by.as_deref().unwrap_or("Unknown"),
                    execution.executed_on.as_deref().unwrap_or("Unknown"),
                    execution.comment.as_deref().unwrap_or("None")
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Create Zephyr Test Execution Tool
pub struct CreateZephyrTestExecutionTool {
    client: JiraClient,
}

impl CreateZephyrTestExecutionTool {
    #[must_use]
    /// # Panics
    /// This function does not panic.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for CreateZephyrTestExecutionTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let test_case_id = args
            .get("test_case_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: test_case_id")
            })?;

        let project_id = args
            .get("project_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: project_id")
            })?;

        let status = args.get("status").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::JiraError::api_error("Missing required parameter: status")
        })?;

        let cycle_id = args
            .get("cycle_id")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        let version_id = args
            .get("version_id")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        let assignee = args
            .get("assignee")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        let comment = args
            .get("comment")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let execution_request = ZephyrTestExecutionCreateRequest {
            test_case_id: test_case_id.to_string(),
            cycle_id,
            version_id,
            project_id: project_id.to_string(),
            status: status.to_string(),
            assignee,
            comment,
            step_results: None,
        };

        info!(
            "Creating Zephyr test execution for test case: {}",
            test_case_id
        );

        let created_execution = self
            .client
            .create_zephyr_test_execution(&execution_request)
            .await?;

        let response_text = format!(
            "Test execution created successfully!\n\nTest Case ID: {}\nExecution ID: {}\nStatus: {}\nProject ID: {}\nAssignee: {}\nComment: {}",
            created_execution.test_case_id,
            created_execution.id.as_deref().unwrap_or("Unknown"),
            created_execution.status,
            created_execution.project_id,
            created_execution.assignee.as_deref().unwrap_or("Unassigned"),
            created_execution.comment.as_deref().unwrap_or("None")
        );

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Get Zephyr Test Cycles Tool
pub struct GetZephyrTestCyclesTool {
    client: JiraClient,
}

impl GetZephyrTestCyclesTool {
    #[must_use]
    /// # Panics
    /// This function does not panic.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetZephyrTestCyclesTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: project_key")
            })?;

        info!("Getting Zephyr test cycles for project: {}", project_key);

        let test_cycles = self.client.get_zephyr_test_cycles(project_key).await?;

        let mut response_text = format!("Test Cycles for Project {project_key}:\n\n");

        if test_cycles.is_empty() {
            response_text.push_str("No test cycles found.");
        } else {
            for (i, cycle) in test_cycles.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. {}\n   ID: {}\n   Description: {}\n   Environment: {}\n   Status: {}\n   Created By: {}\n\n",
                    i + 1,
                    cycle.name,
                    cycle.id.as_deref().unwrap_or("Unknown"),
                    cycle.description.as_deref().unwrap_or("None"),
                    cycle.environment.as_deref().unwrap_or("None"),
                    cycle.status.as_deref().unwrap_or("Unknown"),
                    cycle.created_by.as_deref().unwrap_or("Unknown")
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}

// Get Zephyr Test Plans Tool
pub struct GetZephyrTestPlansTool {
    client: JiraClient,
}

impl GetZephyrTestPlansTool {
    #[must_use]
    /// # Panics
    /// This function does not panic.
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: JiraClient::new(config).expect("Failed to create JiraClient"),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::server::MCPToolHandler for GetZephyrTestPlansTool {
    async fn handle(&self, args: serde_json::Value) -> Result<MCPToolResult> {
        let project_key = args
            .get("project_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::JiraError::api_error("Missing required parameter: project_key")
            })?;

        info!("Getting Zephyr test plans for project: {}", project_key);

        let test_plans = self.client.get_zephyr_test_plans(project_key).await?;

        let mut response_text = format!("Test Plans for Project {project_key}:\n\n");

        if test_plans.is_empty() {
            response_text.push_str("No test plans found.");
        } else {
            for (i, plan) in test_plans.iter().enumerate() {
                response_text.push_str(&format!(
                    "{}. {}\n   ID: {}\n   Description: {}\n   Status: {}\n   Created By: {}\n\n",
                    i + 1,
                    plan.name,
                    plan.id.as_deref().unwrap_or("Unknown"),
                    plan.description.as_deref().unwrap_or("None"),
                    plan.status.as_deref().unwrap_or("Unknown"),
                    plan.created_by.as_deref().unwrap_or("Unknown")
                ));
            }
        }

        Ok(MCPToolResult {
            content: vec![MCPContent::text(response_text)],
            is_error: Some(false),
        })
    }
}
