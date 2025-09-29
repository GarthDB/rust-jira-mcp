//! # Configuration Management
//!
//! This module provides comprehensive configuration management for the Rust Jira MCP Server.
//! It supports multiple configuration sources, validation, secret management, and hot-reloading.
//!
//! ## Features
//!
//! - **Multiple Sources**: Environment variables, .env files, TOML/YAML/JSON config files
//! - **Priority Ordering**: Clear precedence for configuration sources
//! - **Validation**: Comprehensive validation with detailed error messages
//! - **Secret Management**: Secure handling of sensitive data
//! - **Hot-Reloading**: Automatic configuration reloading when files change
//! - **Type Safety**: Strongly typed configuration with serde
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rust_jira_mcp::config::{JiraConfig, ConfigManager};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration from environment
//!     let config = JiraConfig::load()?;
//!     
//!     // Or use the configuration manager
//!     let mut manager = ConfigManager::new();
//!     manager.load_with_options(Default::default()).await?;
//!     let config = manager.get_config().await;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration Sources (Priority Order)
//!
//! 1. **Environment Variables** (Highest priority)
//! 2. **.env file**
//! 3. **Custom config files** (specified via `JIRA_CONFIG_FILE`)
//! 4. **config/local.toml**
//! 5. **config/default.toml**
//! 6. **Default values** (Lowest priority)
//!
//! ## Environment Variables
//!
//! ```bash
//! # Required
//! JIRA_EMAIL=your.email@company.com
//! JIRA_PERSONAL_ACCESS_TOKEN=your_personal_access_token_here
//!
//! # Optional
//! JIRA_API_BASE_URL=https://your-company.atlassian.net/rest/api/2
//! JIRA_DEFAULT_PROJECT=PROJ
//! JIRA_MAX_RESULTS=50
//! JIRA_TIMEOUT_SECONDS=30
//! JIRA_STRICT_SSL=true
//! JIRA_LOG_FILE=~/Desktop/jira-api.log
//! ```
//!
//! ## Configuration Files
//!
//! ### TOML Configuration
//!
//! ```toml
//! [default]
//! api_base_url = "https://your-company.atlassian.net/rest/api/2"
//! max_results = 50
//! timeout_seconds = 30
//! strict_ssl = true
//! log_file = "~/Desktop/jira-api.log"
//! ```
//!
//! ### JSON Configuration
//!
//! ```json
//! {
//!   "default": {
//!     "api_base_url": "https://your-company.atlassian.net/rest/api/2",
//!     "max_results": 50,
//!     "timeout_seconds": 30,
//!     "strict_ssl": true,
//!     "log_file": "~/Desktop/jira-api.log"
//!   }
//! }
//! ```
//!
//! ## Secret Management
//!
//! The configuration system supports multiple secret storage methods:
//!
//! - **Plain Text**: Direct storage (not recommended for production)
//! - **Base64 Encoded**: Encoded secrets in config files
//! - **Environment Variables**: Reference environment variables
//! - **File References**: Load secrets from files
//!
//! ```toml
//! [secrets]
//! # Base64 encoded secrets
//! personal_access_token = { type = "base64", value = "eW91cl90b2tlbg==" }
//! email = { type = "plain", value = "your.email@company.com" }
//!
//! # Reference environment variables
//! # personal_access_token = { type = "env", value = "JIRA_TOKEN" }
//!
//! # Reference files
//! # personal_access_token = { type = "file", value = "/path/to/token/file" }
//! ```
//!
//! ## Hot-Reloading
//!
//! Enable automatic configuration reloading when files change:
//!
//! ```rust,no_run
//! use rust_jira_mcp::config::{ConfigManager, ConfigOptions};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut config_manager = ConfigManager::new();
//!     let options = ConfigOptions {
//!         hot_reload: true,
//!         watch_paths: vec![PathBuf::from("config/local.toml")],
//!         strict_validation: true,
//!         fail_on_missing: true,
//!     };
//!     config_manager.load_with_options(options).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Validation
//!
//! The configuration system includes comprehensive validation:
//!
//! ```rust,no_run
//! use rust_jira_mcp::config::validation::{ConfigValidator, ValidationRule};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let validator = ConfigValidator::new()
//!         .add_rule(ValidationRule::new("email".to_string())
//!             .required()
//!             .custom_validator(|email| {
//!                 if email.contains('@') {
//!                     Ok(())
//!                 } else {
//!                     Err("Invalid email format".to_string())
//!                 }
//!             }))
//!         .add_rule(ValidationRule::new("token".to_string())
//!             .required()
//!             .min_length(10));
//!
//!     validator.validate("email", "test@example.com")?;
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! Configuration errors provide detailed information:
//!
//! ```rust,no_run
//! use rust_jira_mcp::config::validation::ConfigValidationError;
//! use rust_jira_mcp::config::JiraConfig;
//!
//! match JiraConfig::load() {
//!     Ok(config) => {
//!         match config.validate() {
//!             Ok(()) => println!("Configuration is valid"),
//!             Err(e) => {
//!                 match e.downcast_ref::<ConfigValidationError>() {
//!                     Some(ConfigValidationError::MissingRequiredField(field)) => {
//!                         println!("Required field '{}' is missing", field);
//!                     }
//!                     Some(ConfigValidationError::InvalidEmail(email)) => {
//!                         println!("Invalid email format: '{}'", email);
//!                     }
//!                     Some(ConfigValidationError::ValidationFailed(errors)) => {
//!                         println!("Multiple validation errors:");
//!                         for error in errors {
//!                             println!("  - {}", error);
//!                         }
//!                     }
//!                     _ => println!("Configuration error: {}", e),
//!                 }
//!             }
//!         }
//!     }
//!     Err(e) => println!("Failed to load configuration: {}", e),
//! }
//! ```

pub mod jira;
pub mod manager;
pub mod secrets;
pub mod validation;

pub use jira::JiraConfig;
pub use manager::{ConfigManager, ConfigOptions};
pub use secrets::SecretManager;
