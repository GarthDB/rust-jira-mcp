pub mod jira;

pub use jira::JiraError;

pub type Result<T> = std::result::Result<T, JiraError>;
