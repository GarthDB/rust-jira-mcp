use crate::config::secrets::SecretManager;
use anyhow::{Context, Result};
use config::{Config as ConfigBuilder, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    pub api_base_url: String,
    pub email: String,
    pub personal_access_token: String,
    pub default_project: Option<String>,
    pub max_results: Option<u32>,
    pub timeout_seconds: Option<u64>,
    pub log_file: Option<PathBuf>,
    pub strict_ssl: Option<bool>,
}

impl Default for JiraConfig {
    fn default() -> Self {
        Self {
            api_base_url: "https://jira.corp.adobe.com/rest/api/2".to_string(),
            email: String::new(),
            personal_access_token: String::new(),
            default_project: None,
            max_results: Some(50),
            timeout_seconds: Some(30),
            log_file: None,
            strict_ssl: Some(true),
        }
    }
}

impl JiraConfig {
    /// Load Jira configuration from environment variables and config files.
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are missing or if configuration
    /// files cannot be read or parsed.
    pub fn load() -> Result<Self> {
        // Load .env file if it exists, or use custom filename from environment
        if let Ok(env_file) = std::env::var("DOTENV_FILENAME") {
            dotenvy::from_filename(env_file).ok();
        } else {
            dotenvy::dotenv().ok();
        }

        let mut builder = ConfigBuilder::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("JIRA"));

        // Try to load from .env file
        if let Ok(env_path) = std::env::var("JIRA_CONFIG_FILE") {
            builder = builder.add_source(File::with_name(&env_path).required(false));
        }

        let config = builder.build().context("Failed to build configuration")?;

        let mut jira_config: JiraConfig = config
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

        // Validate required fields
        if jira_config.email.is_empty() {
            anyhow::bail!("JIRA_EMAIL environment variable is required");
        }
        if jira_config.personal_access_token.is_empty() {
            anyhow::bail!("JIRA_PERSONAL_ACCESS_TOKEN environment variable is required");
        }

        // Set default log file if not specified
        if jira_config.log_file.is_none() {
            jira_config.log_file = Some(
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("Desktop")
                    .join("jira-api.log"),
            );
        }

        Ok(jira_config)
    }

    #[must_use]
    pub fn auth_header(&self) -> String {
        // Jira Personal Access Tokens require Basic authentication, not Bearer
        // Format: Basic base64(email:token)
        let credentials = format!("{}:{}", self.email, self.personal_access_token);
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, credentials);
        format!("Basic {}", encoded)
    }

    #[must_use]
    pub fn timeout_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_seconds.unwrap_or(30))
    }

    /// Load configuration with secret management
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Configuration loading fails
    /// - Secret resolution fails
    /// - Required environment variables are missing
    pub async fn load_with_secrets(secret_manager: &SecretManager) -> Result<Self> {
        let mut config = Self::load()?;

        // Override sensitive fields with secrets if available
        if let Some(token) = secret_manager.get_secret("personal_access_token").await? {
            config.personal_access_token = token;
        }

        if let Some(email) = secret_manager.get_secret("email").await? {
            config.email = email;
        }

        Ok(config)
    }

    /// Validate the configuration
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Email format is invalid
    /// - Personal access token is too short
    /// - API base URL format is invalid
    pub fn validate(&self) -> Result<()> {
        use crate::config::validation::ConfigValidator;

        let validator = ConfigValidator::new()
            .add_rule(
                crate::config::validation::ValidationRule::new("email".to_string())
                    .required()
                    .custom_validator(|email| {
                        if email.contains('@')
                            && email.contains('.')
                            && !email.starts_with('@')
                            && !email.ends_with('@')
                        {
                            Ok(())
                        } else {
                            Err("Invalid email format".to_string())
                        }
                    }),
            )
            .add_rule(
                crate::config::validation::ValidationRule::new("personal_access_token".to_string())
                    .required()
                    .min_length(10),
            )
            .add_rule(
                crate::config::validation::ValidationRule::new("api_base_url".to_string())
                    .required()
                    .custom_validator(|url| {
                        if url.starts_with("http://") || url.starts_with("https://") {
                            Ok(())
                        } else {
                            Err("Invalid URL format".to_string())
                        }
                    }),
            );

        validator.validate("email", &self.email)?;
        validator.validate("personal_access_token", &self.personal_access_token)?;
        validator.validate("api_base_url", &self.api_base_url)?;

        Ok(())
    }
}
