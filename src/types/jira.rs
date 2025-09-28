use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Jira issue representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssue {
    pub id: String,
    pub key: String,
    pub self_url: String,
    pub fields: HashMap<String, serde_json::Value>,
}

/// Jira project representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraProject {
    pub id: String,
    pub key: String,
    pub name: String,
    pub project_type_key: String,
    pub self_url: String,
}

/// Jira user representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraUser {
    pub account_id: String,
    pub display_name: String,
    pub email_address: Option<String>,
    pub active: bool,
    pub time_zone: Option<String>,
}

/// Jira issue type representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueType {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub subtask: bool,
}

/// Jira priority representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraPriority {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
}

/// Jira status representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatus {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub status_category: JiraStatusCategory,
}

/// Jira status category representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatusCategory {
    pub id: i32,
    pub key: String,
    pub color_name: String,
    pub name: String,
}

/// Jira component representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraComponent {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub self_url: String,
}

/// Jira comment representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraComment {
    pub id: String,
    pub body: String,
    pub author: JiraUser,
    pub created: String,
    pub updated: Option<String>,
}

/// Jira transition representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraTransition {
    pub id: String,
    pub name: String,
    pub to: JiraStatus,
    pub properties: TransitionProperties,
}

/// Properties of a Jira transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionProperties {
    pub screen: ScreenProperty,
    pub scope: ScopeProperty,
    pub availability: AvailabilityProperty,
    pub conditionality: ConditionalityProperty,
}

/// Screen property of a transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScreenProperty {
    HasScreen,
    NoScreen,
}

/// Scope property of a transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScopeProperty {
    Global,
    Local,
}

/// Availability property of a transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AvailabilityProperty {
    Available,
    Unavailable,
}

/// Conditionality property of a transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionalityProperty {
    Conditional,
    Unconditional,
}

impl TransitionProperties {
    /// Create a new transition properties struct with default values
    #[must_use]
    pub fn new() -> Self {
        Self {
            screen: ScreenProperty::NoScreen,
            scope: ScopeProperty::Local,
            availability: AvailabilityProperty::Unavailable,
            conditionality: ConditionalityProperty::Unconditional,
        }
    }

    /// Create a new transition properties struct with all properties set to their positive values
    #[must_use]
    pub fn with_all_properties() -> Self {
        Self {
            screen: ScreenProperty::HasScreen,
            scope: ScopeProperty::Global,
            availability: AvailabilityProperty::Available,
            conditionality: ConditionalityProperty::Conditional,
        }
    }
}

impl Default for TransitionProperties {
    fn default() -> Self {
        Self::new()
    }
}

/// Jira search result representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraSearchResult {
    pub expand: Option<String>,
    pub start_at: i32,
    pub max_results: i32,
    pub total: i32,
    pub issues: Vec<JiraIssue>,
}

/// Jira work log representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraWorkLog {
    pub id: String,
    pub comment: Option<String>,
    pub time_spent: String,
    pub time_spent_seconds: i32,
    pub author: JiraUser,
    pub created: String,
    pub updated: Option<String>,
}

/// Zephyr test step representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestStep {
    pub id: Option<String>,
    pub step: String,
    pub data: Option<String>,
    pub result: Option<String>,
    pub order: i32,
    pub test_case_id: Option<String>,
}

/// Zephyr test case representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestCase {
    pub id: Option<String>,
    pub key: Option<String>,
    pub name: String,
    pub project_key: String,
    pub issue_type: String,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub description: Option<String>,
    pub labels: Option<Vec<String>>,
    pub components: Option<Vec<String>>,
    pub fix_versions: Option<Vec<String>>,
    pub custom_fields: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Zephyr test execution representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestExecution {
    pub id: Option<String>,
    pub test_case_id: String,
    pub test_case_key: Option<String>,
    pub execution_id: Option<String>,
    pub cycle_id: Option<String>,
    pub version_id: Option<String>,
    pub project_id: String,
    pub status: String,
    pub assignee: Option<String>,
    pub executed_by: Option<String>,
    pub executed_on: Option<String>,
    pub comment: Option<String>,
    pub execution_time: Option<i64>,
    pub defects: Option<Vec<String>>,
    pub step_results: Option<Vec<ZephyrStepResult>>,
}

/// Zephyr test step result representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrStepResult {
    pub id: Option<String>,
    pub step_id: String,
    pub status: String,
    pub comment: Option<String>,
    pub executed_by: Option<String>,
    pub executed_on: Option<String>,
    pub defects: Option<Vec<String>>,
    pub attachments: Option<Vec<ZephyrAttachment>>,
}

/// Zephyr attachment representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrAttachment {
    pub id: String,
    pub name: String,
    pub url: String,
    pub content_type: Option<String>,
    pub size: Option<i64>,
}

/// Zephyr test cycle representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestCycle {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub project_key: String,
    pub version_id: Option<String>,
    pub environment: Option<String>,
    pub created_by: Option<String>,
    pub created_on: Option<String>,
    pub modified_by: Option<String>,
    pub modified_on: Option<String>,
    pub status: Option<String>,
    pub test_executions: Option<Vec<ZephyrTestExecution>>,
}

/// Zephyr test plan representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestPlan {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub project_key: String,
    pub version_id: Option<String>,
    pub created_by: Option<String>,
    pub created_on: Option<String>,
    pub modified_by: Option<String>,
    pub modified_on: Option<String>,
    pub status: Option<String>,
    pub test_cycles: Option<Vec<ZephyrTestCycle>>,
}

/// Zephyr test step creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestStepCreateRequest {
    pub step: String,
    pub data: Option<String>,
    pub result: Option<String>,
    pub order: i32,
    pub test_case_id: String,
}

/// Zephyr test step update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestStepUpdateRequest {
    pub step: Option<String>,
    pub data: Option<String>,
    pub result: Option<String>,
    pub order: Option<i32>,
}

/// Zephyr test execution creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestExecutionCreateRequest {
    pub test_case_id: String,
    pub cycle_id: Option<String>,
    pub version_id: Option<String>,
    pub project_id: String,
    pub status: String,
    pub assignee: Option<String>,
    pub comment: Option<String>,
    pub step_results: Option<Vec<ZephyrStepResultCreateRequest>>,
}

/// Zephyr test step result creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrStepResultCreateRequest {
    pub step_id: String,
    pub status: String,
    pub comment: Option<String>,
    pub defects: Option<Vec<String>>,
}

/// Zephyr test execution update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestExecutionUpdateRequest {
    pub status: Option<String>,
    pub assignee: Option<String>,
    pub comment: Option<String>,
    pub execution_time: Option<i64>,
    pub defects: Option<Vec<String>>,
    pub step_results: Option<Vec<ZephyrStepResultUpdateRequest>>,
}

/// Zephyr test step result update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrStepResultUpdateRequest {
    pub step_id: String,
    pub status: Option<String>,
    pub comment: Option<String>,
    pub defects: Option<Vec<String>>,
}

/// Zephyr test case creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestCaseCreateRequest {
    pub name: String,
    pub project_key: String,
    pub issue_type: String,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub description: Option<String>,
    pub labels: Option<Vec<String>>,
    pub components: Option<Vec<String>>,
    pub fix_versions: Option<Vec<String>>,
    pub custom_fields: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Zephyr test case update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestCaseUpdateRequest {
    pub name: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub description: Option<String>,
    pub labels: Option<Vec<String>>,
    pub components: Option<Vec<String>>,
    pub fix_versions: Option<Vec<String>>,
    pub custom_fields: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Zephyr test cycle creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestCycleCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub project_key: String,
    pub version_id: Option<String>,
    pub environment: Option<String>,
}

/// Zephyr test plan creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestPlanCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub project_key: String,
    pub version_id: Option<String>,
}

/// Zephyr search result for test cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestCaseSearchResult {
    pub total: i32,
    pub start_at: i32,
    pub max_results: i32,
    pub test_cases: Vec<ZephyrTestCase>,
}

/// Zephyr search result for test executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestExecutionSearchResult {
    pub total: i32,
    pub start_at: i32,
    pub max_results: i32,
    pub test_executions: Vec<ZephyrTestExecution>,
}

/// Zephyr search result for test steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrTestStepSearchResult {
    pub total: i32,
    pub start_at: i32,
    pub max_results: i32,
    pub test_steps: Vec<ZephyrTestStep>,
}

/// Jira link type representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraLinkType {
    pub id: String,
    pub name: String,
    pub inward: String,
    pub outward: String,
    pub self_url: String,
}

/// Jira issue link representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueLink {
    pub id: String,
    pub self_url: String,
    pub link_type: JiraLinkType,
    pub inward_issue: Option<JiraIssue>,
    pub outward_issue: Option<JiraIssue>,
}

/// Jira issue link creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueLinkCreateRequest {
    pub link_type: JiraIssueLinkType,
    pub inward_issue: Option<JiraIssueLinkTarget>,
    pub outward_issue: Option<JiraIssueLinkTarget>,
    pub comment: Option<JiraIssueLinkComment>,
}

/// Jira issue link type for creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueLinkType {
    pub name: String,
}

/// Jira issue link target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueLinkTarget {
    pub key: String,
}

/// Jira issue link comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueLinkComment {
    pub body: String,
    pub visibility: Option<JiraCommentVisibility>,
}

/// Jira comment visibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraCommentVisibility {
    pub r#type: String,
    pub value: String,
}

/// Jira attachment representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraAttachment {
    pub id: String,
    pub self_url: String,
    pub filename: String,
    pub author: JiraUser,
    pub created: String,
    pub size: i64,
    pub mime_type: String,
    pub content: Option<String>,
}

/// Jira attachment creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraAttachmentCreateRequest {
    pub filename: String,
    pub content: String, // Base64 encoded content
    pub mime_type: Option<String>,
}

/// Jira work log creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraWorkLogCreateRequest {
    pub comment: Option<String>,
    pub time_spent: String,      // e.g., "1h 30m", "2d", "3w"
    pub started: Option<String>, // ISO 8601 datetime
    pub visibility: Option<JiraCommentVisibility>,
}

/// Jira work log update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraWorkLogUpdateRequest {
    pub comment: Option<String>,
    pub time_spent: Option<String>,
    pub started: Option<String>,
    pub visibility: Option<JiraCommentVisibility>,
}

/// Bulk operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BulkOperationType {
    Update,
    Transition,
    AddComment,
    Mixed,
}

/// Individual operation within a bulk operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationItem {
    pub issue_key: String,
    pub operation_type: BulkOperationType,
    pub data: serde_json::Value,
}

/// Bulk operation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationConfig {
    pub batch_size: Option<usize>,
    pub continue_on_error: bool,
    pub rate_limit_ms: Option<u64>,
    pub max_retries: Option<usize>,
}

impl Default for BulkOperationConfig {
    fn default() -> Self {
        Self {
            batch_size: Some(10),
            continue_on_error: true,
            rate_limit_ms: Some(100),
            max_retries: Some(3),
        }
    }
}

/// Result of a single operation within a bulk operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationResult {
    pub issue_key: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub operation_type: BulkOperationType,
}

/// Result of a complete bulk operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationSummary {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub results: Vec<BulkOperationResult>,
    pub duration_ms: u64,
}

impl BulkOperationSummary {
    /// Create a new bulk operation summary
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            results: Vec::new(),
            duration_ms: 0,
        }
    }

    /// Add a result to the summary
    pub fn add_result(&mut self, result: BulkOperationResult) {
        self.total_operations += 1;
        if result.success {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }
        self.results.push(result);
    }

    /// Get success rate as a percentage
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let result = (self.successful_operations as f64 / self.total_operations as f64) * 100.0;
            result
        }
    }
}

impl Default for BulkOperationSummary {
    fn default() -> Self {
        Self::new()
    }
}
