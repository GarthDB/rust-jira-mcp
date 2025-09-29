use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

/// Secret management for handling sensitive configuration data
#[derive(Debug, Clone)]
pub struct SecretManager {
    secrets: Arc<RwLock<HashMap<String, SecretValue>>>,
    key_file: Option<PathBuf>,
    encrypted: bool,
}

/// A secret value that can be stored in different formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretValue {
    /// Plain text secret (not recommended for production)
    Plain(String),
    /// Base64 encoded secret
    Base64(String),
    /// Reference to environment variable
    EnvVar(String),
    /// Reference to file path
    FilePath(PathBuf),
    /// Encrypted secret (placeholder for future encryption)
    Encrypted(String),
}

impl SecretValue {
    /// Get the actual secret value, resolving references as needed
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Base64 decoding fails for base64 secrets
    /// - Environment variable is not found for env secrets
    /// - File cannot be read for file secrets
    /// - Invalid UTF-8 in decoded content
    pub async fn resolve(&self) -> Result<String> {
        match self {
            SecretValue::Plain(value) => Ok(value.clone()),
            SecretValue::Base64(encoded) => {
                let decoded = general_purpose::STANDARD
                    .decode(encoded)
                    .context("Failed to decode base64 secret")?;
                String::from_utf8(decoded).context("Invalid UTF-8 in decoded secret")
            }
            SecretValue::EnvVar(var_name) => std::env::var(var_name)
                .context(format!("Environment variable '{var_name}' not found")),
            SecretValue::FilePath(path) => fs::read_to_string(path).await.context(format!(
                "Failed to read secret from file: {}",
                path.display()
            )),
            SecretValue::Encrypted(encrypted) => {
                // For now, just return the encrypted value
                // In a real implementation, you'd decrypt it here
                Ok(encrypted.clone())
            }
        }
    }
}

/// Secret configuration loaded from files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretConfig {
    pub secrets: HashMap<String, SecretValue>,
    pub encryption_key: Option<String>,
    pub key_file: Option<PathBuf>,
}

impl SecretManager {
    /// Create a new secret manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
            key_file: None,
            encrypted: false,
        }
    }

    /// Load secrets from a configuration file
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - File cannot be read
    /// - File content is not valid TOML
    /// - Secret configuration structure is invalid
    pub async fn load_from_file(&mut self, path: &PathBuf) -> Result<()> {
        let content = fs::read_to_string(path)
            .await
            .context(format!("Failed to read secrets file: {}", path.display()))?;

        let secret_config: SecretConfig =
            toml::from_str(&content).context("Failed to parse secrets configuration")?;

        self.key_file.clone_from(&secret_config.key_file);
        self.encrypted = secret_config.encryption_key.is_some();

        let mut secrets = self.secrets.write().await;
        for (key, value) in secret_config.secrets {
            secrets.insert(key, value);
        }

        Ok(())
    }

    /// Load secrets from environment variables with a prefix
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Environment variable parsing fails
    /// - Secret value resolution fails
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - A key that starts with the prefix doesn't have a valid suffix after stripping the prefix
    pub async fn load_from_env(&mut self, prefix: &str) -> Result<()> {
        let mut secrets = self.secrets.write().await;

        for (key, value) in std::env::vars() {
            if key.starts_with(prefix) {
                let secret_key = key.strip_prefix(prefix).unwrap().to_lowercase();
                let secret_value = if value.starts_with("base64:") {
                    SecretValue::Base64(value.strip_prefix("base64:").unwrap().to_string())
                } else if value.starts_with("file:") {
                    SecretValue::FilePath(PathBuf::from(value.strip_prefix("file:").unwrap()))
                } else if value.starts_with("env:") {
                    SecretValue::EnvVar(value.strip_prefix("env:").unwrap().to_string())
                } else {
                    SecretValue::Plain(value)
                };
                secrets.insert(secret_key, secret_value);
            }
        }

        Ok(())
    }

    /// Get a secret value by key
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Secret resolution fails (base64 decode, file read, etc.)
    /// - Environment variable is not found for env secrets
    pub async fn get_secret(&self, key: &str) -> Result<Option<String>> {
        let secrets = self.secrets.read().await;
        if let Some(secret_value) = secrets.get(key) {
            Ok(Some(secret_value.resolve().await?))
        } else {
            Ok(None)
        }
    }
}

impl Default for SecretManager {
    fn default() -> Self {
        Self::new()
    }
}
