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
