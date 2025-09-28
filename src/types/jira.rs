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
    pub id: String,
    pub step: String,
    pub data: Option<String>,
    pub result: Option<String>,
    pub order: i32,
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
            (self.successful_operations as f64 / self.total_operations as f64) * 100.0
        }
    }
}

impl Default for BulkOperationSummary {
    fn default() -> Self {
        Self::new()
    }
}
