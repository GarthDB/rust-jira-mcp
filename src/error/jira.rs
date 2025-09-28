use reqwest::StatusCode;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JiraError {
    #[error("HTTP error: {status} - {message}")]
    HttpError { status: StatusCode, message: String },

    #[error("Jira API error: {message}")]
    ApiError { message: String },

    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    #[error("Authentication error: {message}")]
    AuthError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("HTTP client error: {0}")]
    HttpClientError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigErrorWrapper(#[from] config::ConfigError),

    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),

    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

impl JiraError {
    #[must_use]
    pub fn from_jira_response(status: StatusCode, body: &Value) -> Self {
        let message =
            if let Some(error_messages) = body.get("errorMessages").and_then(|v| v.as_array()) {
                error_messages
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            } else if let Some(errors) = body.get("errors").and_then(|v| v.as_object()) {
                errors
                    .iter()
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                format!("HTTP {status}")
            };

        Self::ApiError { message }
    }

    #[must_use]
    pub fn validation_error(field: &str, message: &str) -> Self {
        Self::ValidationError {
            field: field.to_string(),
            message: message.to_string(),
        }
    }

    #[must_use]
    pub fn auth_error(message: &str) -> Self {
        Self::AuthError {
            message: message.to_string(),
        }
    }

    #[must_use]
    pub fn config_error(message: &str) -> Self {
        Self::ConfigError {
            message: message.to_string(),
        }
    }
}
