use serde::{Deserialize, Serialize};
use std::fmt;

/// Type alias for custom validation functions
pub type CustomValidator = Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// Configuration validation errors with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigValidationError {
    /// A required field is missing
    MissingRequiredField(String),
    /// Invalid email format
    InvalidEmail(String),
    /// Invalid URL format
    InvalidUrl(String, String),
    /// Value is outside valid range
    InvalidRange(String, i64, i64, i64), // field, value, min, max
    /// Invalid file path
    InvalidFilePath(String, String), // field, path
    /// Configuration file not found
    ConfigFileNotFound(String),
    /// Configuration file parse error
    ConfigFileParseError(String, String), // file, error
    /// Multiple validation errors
    ValidationFailed(Vec<ConfigValidationError>),
    /// Environment variable not set
    MissingEnvironmentVariable(String),
    /// Invalid environment variable value
    InvalidEnvironmentVariable(String, String), // var, value
}

impl fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValidationError::MissingRequiredField(field) => {
                write!(f, "Required field '{field}' is missing")
            }
            ConfigValidationError::InvalidEmail(email) => {
                write!(f, "Invalid email format: '{email}'")
            }
            ConfigValidationError::InvalidUrl(field, url) => {
                write!(f, "Invalid URL in field '{field}': '{url}'")
            }
            ConfigValidationError::InvalidRange(field, value, min, max) => {
                write!(
                    f,
                    "Field '{field}' value {value} is outside valid range [{min}, {max}]"
                )
            }
            ConfigValidationError::InvalidFilePath(field, path) => {
                write!(f, "Invalid file path in field '{field}': '{path}'")
            }
            ConfigValidationError::ConfigFileNotFound(file) => {
                write!(f, "Configuration file not found: '{file}'")
            }
            ConfigValidationError::ConfigFileParseError(file, error) => {
                write!(f, "Failed to parse configuration file '{file}': {error}")
            }
            ConfigValidationError::ValidationFailed(errors) => {
                write!(
                    f,
                    "Configuration validation failed with {} errors:",
                    errors.len()
                )?;
                for error in errors {
                    write!(f, "\n  - {error}")?;
                }
                Ok(())
            }
            ConfigValidationError::MissingEnvironmentVariable(var) => {
                write!(f, "Required environment variable '{var}' is not set")
            }
            ConfigValidationError::InvalidEnvironmentVariable(var, value) => {
                write!(
                    f,
                    "Invalid value for environment variable '{var}': '{value}'"
                )
            }
        }
    }
}

impl std::error::Error for ConfigValidationError {}

/// Configuration validation rules
pub struct ValidationRule {
    pub field: String,
    pub required: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub pattern: Option<String>,
    pub custom_validator: Option<CustomValidator>,
}

impl fmt::Debug for ValidationRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidationRule")
            .field("field", &self.field)
            .field("required", &self.required)
            .field("min_length", &self.min_length)
            .field("max_length", &self.max_length)
            .field("min_value", &self.min_value)
            .field("max_value", &self.max_value)
            .field("pattern", &self.pattern)
            .field("custom_validator", &self.custom_validator.is_some())
            .finish()
    }
}

impl ValidationRule {
    #[must_use]
    pub fn new(field: String) -> Self {
        Self {
            field,
            required: false,
            min_length: None,
            max_length: None,
            min_value: None,
            max_value: None,
            pattern: None,
            custom_validator: None,
        }
    }

    #[must_use]
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    #[must_use]
    pub fn min_length(mut self, min: usize) -> Self {
        self.min_length = Some(min);
        self
    }

    #[must_use]
    pub fn custom_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        self.custom_validator = Some(Box::new(validator));
        self
    }
}

/// Configuration validator that applies rules to configuration values
pub struct ConfigValidator {
    rules: Vec<ValidationRule>,
}

impl ConfigValidator {
    #[must_use]
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    #[must_use]
    pub fn add_rule(mut self, rule: ValidationRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Validate a field value against all applicable rules
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Required field is missing or empty
    /// - Field value doesn't meet length requirements
    /// - Field value is outside numeric range
    /// - Field value doesn't match pattern
    /// - Custom validator fails
    pub fn validate(&self, field: &str, value: &str) -> Result<(), ConfigValidationError> {
        for rule in &self.rules {
            if rule.field == field {
                return Self::validate_field(rule, value);
            }
        }
        Ok(())
    }

    fn validate_field(rule: &ValidationRule, value: &str) -> Result<(), ConfigValidationError> {
        // Check if required field is present
        if rule.required && value.is_empty() {
            return Err(ConfigValidationError::MissingRequiredField(
                rule.field.clone(),
            ));
        }

        // Skip other validations if field is empty and not required
        if value.is_empty() {
            return Ok(());
        }

        // Check minimum length
        if let Some(min_len) = rule.min_length {
            if value.len() < min_len {
                return Err(ConfigValidationError::InvalidRange(
                    rule.field.clone(),
                    value.len().try_into().unwrap_or(i64::MAX),
                    min_len.try_into().unwrap_or(i64::MAX),
                    i64::MAX,
                ));
            }
        }

        // Check maximum length
        if let Some(max_len) = rule.max_length {
            if value.len() > max_len {
                return Err(ConfigValidationError::InvalidRange(
                    rule.field.clone(),
                    value.len().try_into().unwrap_or(0),
                    0,
                    max_len.try_into().unwrap_or(i64::MAX),
                ));
            }
        }

        // Check numeric range
        if let (Some(min_val), Some(max_val)) = (rule.min_value, rule.max_value) {
            if let Ok(num_value) = value.parse::<i64>() {
                if num_value < min_val || num_value > max_val {
                    return Err(ConfigValidationError::InvalidRange(
                        rule.field.clone(),
                        num_value,
                        min_val,
                        max_val,
                    ));
                }
            }
        }

        // Check pattern (simple regex-like validation)
        if let Some(pattern) = &rule.pattern {
            if !Self::matches_pattern(value, pattern) {
                return Err(ConfigValidationError::InvalidEnvironmentVariable(
                    rule.field.clone(),
                    value.to_string(),
                ));
            }
        }

        // Apply custom validator
        if let Some(validator) = &rule.custom_validator {
            if let Err(error) = validator(value) {
                return Err(ConfigValidationError::InvalidEnvironmentVariable(
                    rule.field.clone(),
                    format!("{value}: {error}"),
                ));
            }
        }

        Ok(())
    }

    fn matches_pattern(value: &str, pattern: &str) -> bool {
        // Simple pattern matching - in a real implementation, you'd use regex
        match pattern {
            "email" => value.contains('@') && value.contains('.'),
            "url" => value.starts_with("http://") || value.starts_with("https://"),
            "alphanumeric" => value.chars().all(char::is_alphanumeric),
            _ => true, // Default to true for unknown patterns
        }
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}
