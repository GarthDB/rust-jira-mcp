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
        dotenvy::dotenv().ok(); // Load .env file if it exists

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
        format!("Bearer {}", self.personal_access_token)
    }

    #[must_use]
    pub fn timeout_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_seconds.unwrap_or(30))
    }
}
