use reqwest::StatusCode;
use serde_json::Value;
use thiserror::Error;

/// Comprehensive error types for all Jira MCP operations
#[derive(Error, Debug)]
pub enum JiraError {
    /// HTTP-related errors with status codes
    #[error("HTTP error: {status} - {message}")]
    HttpError { status: StatusCode, message: String },

    /// Jira API-specific errors
    #[error("Jira API error: {message}")]
    ApiError {
        message: String,
        error_codes: Option<Vec<String>>,
    },

    /// Validation errors
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    /// Authentication and authorization errors
    #[error("Authentication error: {message}")]
    AuthError { message: String },

    /// Configuration-related errors
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// HTTP client errors
    #[error("HTTP client error: {0}")]
    HttpClientError(#[from] reqwest::Error),

    /// IO errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigErrorWrapper(#[from] config::ConfigError),

    /// Anyhow errors
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),

    /// URL parsing errors
    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),

    /// Unknown or unexpected errors
    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

impl JiraError {
    /// Create a Jira API error from HTTP response
    #[must_use]
    pub fn from_jira_response(status: StatusCode, body: &Value) -> Self {
        let (message, error_codes) =
            if let Some(error_messages) = body.get("errorMessages").and_then(|v| v.as_array()) {
                let messages: Vec<String> = error_messages
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(std::string::ToString::to_string)
                    .collect();
                (messages.join(", "), vec![])
            } else if let Some(errors) = body.get("errors").and_then(|v| v.as_object()) {
                let error_pairs: Vec<String> =
                    errors.iter().map(|(k, v)| format!("{k}: {v}")).collect();
                let codes: Vec<String> = errors.keys().cloned().collect();
                (error_pairs.join(", "), codes)
            } else {
                (format!("HTTP {status}"), vec![])
            };

        Self::ApiError {
            message,
            error_codes: Some(error_codes),
        }
    }

    /// Create a simple API error
    #[must_use]
    pub fn api_error(message: &str) -> Self {
        Self::ApiError {
            message: message.to_string(),
            error_codes: None,
        }
    }

    /// Create a validation error
    #[must_use]
    pub fn validation_error(field: &str, message: &str) -> Self {
        Self::ValidationError {
            field: field.to_string(),
            message: message.to_string(),
        }
    }

    /// Create an authentication error
    #[must_use]
    pub fn auth_error(message: &str) -> Self {
        Self::AuthError {
            message: message.to_string(),
        }
    }

    /// Create a configuration error
    #[must_use]
    pub fn config_error(message: &str) -> Self {
        Self::ConfigError {
            message: message.to_string(),
        }
    }

    /// Create an unknown error
    #[must_use]
    pub fn unknown_error(message: &str) -> Self {
        Self::Unknown {
            message: message.to_string(),
        }
    }
}
