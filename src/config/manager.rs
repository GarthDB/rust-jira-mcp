use anyhow::{Context, Result};
use config::{Config as ConfigBuilder, Environment, File, FileFormat};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::config::jira::JiraConfig;
use crate::config::validation::ConfigValidationError;

/// Configuration manager that handles loading, validation, and hot-reloading
/// of configuration from multiple sources.
#[derive(Debug, Clone)]
pub struct ConfigManager {
    config: Arc<RwLock<JiraConfig>>,
    config_paths: Vec<PathBuf>,
    watch_enabled: bool,
}

/// Configuration sources in order of precedence (highest to lowest)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigSource {
    /// Environment variables
    Environment,
    /// .env file
    DotEnv,
    /// TOML configuration file
    Toml(PathBuf),
    /// YAML configuration file
    Yaml(PathBuf),
    /// JSON configuration file
    Json(PathBuf),
    /// Default values
    Default,
}

/// Configuration loading options
#[derive(Debug, Clone)]
pub struct ConfigOptions {
    /// Whether to enable hot-reloading
    pub hot_reload: bool,
    /// Configuration file paths to watch
    pub watch_paths: Vec<PathBuf>,
    /// Validation strictness level
    pub strict_validation: bool,
    /// Whether to fail on missing required fields
    pub fail_on_missing: bool,
}

impl Default for ConfigOptions {
    fn default() -> Self {
        Self {
            hot_reload: false,
            watch_paths: vec![],
            strict_validation: true,
            fail_on_missing: true,
        }
    }
}

impl ConfigManager {
    /// Create a new configuration manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(JiraConfig::default())),
            config_paths: Vec::new(),
            watch_enabled: false,
        }
    }

    /// Load configuration from multiple sources with the given options
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Configuration files cannot be read or parsed
    /// - Environment variables are invalid
    /// - Configuration validation fails (if strict validation is enabled)
    /// - Hot-reload setup fails
    pub async fn load_with_options(&mut self, options: ConfigOptions) -> Result<()> {
        let config = Self::load_config_from_sources(&options)?;
        *self.config.write().await = config;

        if options.hot_reload {
            self.enable_hot_reload(&options.watch_paths)?;
        }

        Ok(())
    }


    /// Get the current configuration (read-only)
    pub async fn get_config(&self) -> JiraConfig {
        self.config.read().await.clone()
    }


    /// Enable hot-reloading for the specified paths
    fn enable_hot_reload(&mut self, watch_paths: &[PathBuf]) -> Result<()> {
        if watch_paths.is_empty() {
            return Ok(());
        }

        let config = self.config.clone();
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        let _ = tx.try_send(event);
                    }
                }
            },
            notify::Config::default(),
        )?;

        for path in watch_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::NonRecursive)?;
                self.config_paths.push(path.clone());
            }
        }

        self.watch_enabled = true;

        // Spawn a task to handle file changes
        tokio::spawn(async move {
            while let Some(_event) = rx.recv().await {
                // Debounce reloads to avoid rapid successive reloads
                tokio::time::sleep(Duration::from_millis(500)).await;

                // Reload configuration
                let options = ConfigOptions::default();
                if let Ok(new_config) = Self::load_config_from_sources_static(&options) {
                    *config.write().await = new_config;
                    tracing::info!("Configuration reloaded due to file change");
                }
            }
        });

        Ok(())
    }

    /// Load configuration from all available sources
    fn load_config_from_sources(options: &ConfigOptions) -> Result<JiraConfig> {
        Self::load_config_from_sources_static(options)
    }

    /// Static method to load configuration (used by hot-reload)
    fn load_config_from_sources_static(options: &ConfigOptions) -> Result<JiraConfig> {
        // Load .env file if it exists
        if let Ok(env_file) = std::env::var("DOTENV_FILENAME") {
            dotenvy::from_filename(env_file).ok();
        } else {
            dotenvy::dotenv().ok();
        }

        let mut builder = ConfigBuilder::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("JIRA"));

        // Add custom config files from environment
        if let Ok(config_file) = std::env::var("JIRA_CONFIG_FILE") {
            let path = Path::new(&config_file);
            let format = match path.extension().and_then(|ext| ext.to_str()) {
                Some("yaml" | "yml") => FileFormat::Yaml,
                Some("json") => FileFormat::Json,
                _ => FileFormat::Toml, // Default to TOML
            };
            builder =
                builder.add_source(File::with_name(&config_file).format(format).required(false));
        }

        // Add watch paths as config sources
        for path in &options.watch_paths {
            if path.exists() {
                let format = match path.extension().and_then(|ext| ext.to_str()) {
                    Some("yaml" | "yml") => FileFormat::Yaml,
                    Some("json") => FileFormat::Json,
                    _ => FileFormat::Toml,
                };
                builder = builder.add_source(
                    File::with_name(path.to_str().unwrap())
                        .format(format)
                        .required(false),
                );
            }
        }

        let config = builder.build().context("Failed to build configuration")?;

        let mut jira_config: JiraConfig = config
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

        // Validate configuration
        if options.strict_validation {
            Self::validate_config(&jira_config, options.fail_on_missing)?;
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

    /// Validate configuration with detailed error messages
    fn validate_config(config: &JiraConfig, fail_on_missing: bool) -> Result<()> {
        let mut errors = Vec::new();

        // Validate required fields
        if config.email.is_empty() {
            errors.push(ConfigValidationError::MissingRequiredField(
                "email".to_string(),
            ));
        } else if !Self::is_valid_email(&config.email) {
            errors.push(ConfigValidationError::InvalidEmail(config.email.clone()));
        }

        if config.personal_access_token.is_empty() {
            errors.push(ConfigValidationError::MissingRequiredField(
                "personal_access_token".to_string(),
            ));
        }

        // Validate URL format
        if !Self::is_valid_url(&config.api_base_url) {
            errors.push(ConfigValidationError::InvalidUrl(
                "api_base_url".to_string(),
                config.api_base_url.clone(),
            ));
        }

        // Validate numeric ranges
        if let Some(max_results) = config.max_results {
            if max_results == 0 || max_results > 1000 {
                errors.push(ConfigValidationError::InvalidRange(
                    "max_results".to_string(),
                    i64::from(max_results),
                    1,
                    1000,
                ));
            }
        }

        if let Some(timeout) = config.timeout_seconds {
            if timeout == 0 || timeout > 300 {
                errors.push(ConfigValidationError::InvalidRange(
                    "timeout_seconds".to_string(),
                    timeout.try_into().unwrap_or(300),
                    1,
                    300,
                ));
            }
        }

        if !errors.is_empty() && fail_on_missing {
            return Err(ConfigValidationError::ValidationFailed(errors).into());
        }

        if !errors.is_empty() {
            tracing::warn!("Configuration validation warnings: {:?}", errors);
        }

        Ok(())
    }

    /// Check if email format is valid
    fn is_valid_email(email: &str) -> bool {
        email.contains('@')
            && email.contains('.')
            && !email.starts_with('@')
            && !email.ends_with('@')
    }

    /// Check if URL format is valid
    fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    /// Get configuration sources that were used
    #[must_use]
    pub fn get_config_sources(&self) -> Vec<ConfigSource> {
        let mut sources = vec![ConfigSource::Default];

        // Check for .env file
        if std::env::var("DOTENV_FILENAME").is_ok() || Path::new(".env").exists() {
            sources.insert(0, ConfigSource::DotEnv);
        }

        // Check for environment variables
        if std::env::var("JIRA_EMAIL").is_ok()
            || std::env::var("JIRA_PERSONAL_ACCESS_TOKEN").is_ok()
        {
            sources.insert(0, ConfigSource::Environment);
        }

        // Add config file sources
        for path in &self.config_paths {
            let format = match path.extension().and_then(|ext| ext.to_str()) {
                Some("yaml" | "yml") => ConfigSource::Yaml(path.clone()),
                Some("json") => ConfigSource::Json(path.clone()),
                _ => ConfigSource::Toml(path.clone()),
            };
            sources.insert(0, format);
        }

        sources
    }

    /// Check if hot-reloading is enabled
    #[must_use]
    pub fn is_hot_reload_enabled(&self) -> bool {
        self.watch_enabled
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
