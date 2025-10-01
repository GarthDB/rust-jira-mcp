# Configuration Management System

This document describes the comprehensive configuration management system implemented for the Rust Jira MCP project.

## Overview

The configuration system provides:
- ✅ Environment variable support
- ✅ Configuration file support (.env, TOML, YAML, JSON)
- ✅ Configuration validation on startup
- ✅ Default value handling
- ✅ Configuration hot-reloading
- ✅ Secret management
- ✅ Clear error messages for invalid config
- ✅ Support for multiple config sources

## Quick Start

### 1. Environment Variables (Recommended)

Create a `.env` file in your project root. The server supports **two authentication methods** and automatically detects which one to use:

#### Adobe Jira (Bearer Token Authentication)
```bash
# Required for Adobe Jira
JIRA_EMAIL=your.email@adobe.com
JIRA_PERSONAL_ACCESS_TOKEN=YOUR_ADOBE_JIRA_TOKEN_HERE
JIRA_API_BASE_URL=https://jira.corp.adobe.com/rest/api/2

# Optional
JIRA_DEFAULT_PROJECT=PROJ
JIRA_MAX_RESULTS=50
JIRA_TIMEOUT_SECONDS=30
JIRA_STRICT_SSL=true
JIRA_LOG_FILE=~/Desktop/jira-api.log
```

#### Standard Jira (Basic Authentication)
```bash
# Required for Standard Jira
JIRA_EMAIL=your.email@company.com
JIRA_PERSONAL_ACCESS_TOKEN=your_standard_pat_token
JIRA_API_BASE_URL=https://your-company.atlassian.net/rest/api/2

# Optional
JIRA_DEFAULT_PROJECT=PROJ
JIRA_MAX_RESULTS=50
JIRA_TIMEOUT_SECONDS=30
JIRA_STRICT_SSL=true
JIRA_LOG_FILE=~/Desktop/jira-api.log
```

#### Smart Authentication Detection
The server automatically detects your authentication method:
- **Bearer Token**: If your token is long (>20 chars) and contains no colons
- **Basic Auth**: If your token is short or contains colons (like `user:password`)

### 2. Configuration Files

Create configuration files in the `config/` directory:

**config/default.toml:**
```toml
[default]
api_base_url = "https://jira.corp.adobe.com/rest/api/2"
max_results = 50
timeout_seconds = 30
strict_ssl = true
log_file = "~/Desktop/jira-api.log"
```

**config/local.toml:**
```toml
[default]
# Override specific settings for local development
api_base_url = "https://jira-dev.corp.adobe.com/rest/api/2"
max_results = 25
```

### 3. Secret Management

For sensitive data, use the secrets system:

**config/secrets.toml:**
```toml
[secrets]
# Base64 encoded secrets
personal_access_token = { type = "base64", value = "eW91cl90b2tlbg==" }
email = { type = "plain", value = "your.email@company.com" }

# Reference environment variables
# personal_access_token = { type = "env", value = "JIRA_TOKEN" }

# Reference files
# personal_access_token = { type = "file", value = "/path/to/token/file" }
```

## Configuration Sources (Priority Order)

1. **Environment Variables** (Highest priority)
2. **.env file**
3. **Custom config files** (specified via `JIRA_CONFIG_FILE`)
4. **config/local.toml**
5. **config/default.toml**
6. **Default values** (Lowest priority)

## Usage Examples

### Basic Configuration Loading

```rust
use rust_jira_mcp::config::{ConfigManager, ConfigOptions};

let mut config_manager = ConfigManager::new();
config_manager.load().await?;
let config = config_manager.get_config().await;
```

### Configuration with Hot-Reloading

```rust
use rust_jira_mcp::config::{ConfigManager, ConfigOptions};
use std::path::PathBuf;

let mut config_manager = ConfigManager::new();
let options = ConfigOptions {
    hot_reload: true,
    watch_paths: vec![PathBuf::from("config/local.toml")],
    strict_validation: true,
    fail_on_missing: true,
};
config_manager.load_with_options(options).await?;
```

### Secret Management

```rust
use rust_jira_mcp::config::{SecretManager, SecretValue};
use rust_jira_mcp::config::secrets::helpers as secret_helpers;

let mut secret_manager = SecretManager::new();

// Add secrets
secret_manager.set_secret(
    "personal_access_token".to_string(),
    SecretValue::Plain("your_token".to_string()),
).await;

// Load from file
secret_manager.load_from_file(&PathBuf::from("config/secrets.toml")).await?;

// Load from environment variables
secret_manager.load_from_env("JIRA_").await?;

// Get secret value
if let Some(token) = secret_manager.get_secret("personal_access_token").await? {
    println!("Token: {}", token);
}
```

### Configuration with Secrets

```rust
use rust_jira_mcp::config::{JiraConfig, SecretManager};

let mut secret_manager = SecretManager::new();
secret_manager.load_from_env("JIRA_").await?;

let config = JiraConfig::load_with_secrets(&secret_manager).await?;
config.validate()?;
```

## Configuration Validation

The system includes comprehensive validation:

```rust
use rust_jira_mcp::config::{ConfigValidator, ValidationRule};

let validator = ConfigValidator::new()
    .add_rule(ValidationRule::new("email".to_string())
        .required()
        .custom_validator(|email| {
            if email.contains('@') {
                Ok(())
            } else {
                Err("Invalid email format".to_string())
            }
        }))
    .add_rule(ValidationRule::new("token".to_string())
        .required()
        .min_length(10));

validator.validate("email", "test@example.com")?;
```

## Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `JIRA_EMAIL` | Your Jira email address | - | ✅ |
| `JIRA_PERSONAL_ACCESS_TOKEN` | Your Jira personal access token | - | ✅ |
| `JIRA_API_BASE_URL` | Jira API base URL | `https://jira.corp.adobe.com/rest/api/2` | ❌ |
| `JIRA_DEFAULT_PROJECT` | Default project key | - | ❌ |
| `JIRA_MAX_RESULTS` | Maximum results per request | `50` | ❌ |
| `JIRA_TIMEOUT_SECONDS` | Request timeout in seconds | `30` | ❌ |
| `JIRA_STRICT_SSL` | Enable strict SSL verification | `true` | ❌ |
| `JIRA_LOG_FILE` | Log file path | `~/Desktop/jira-api.log` | ❌ |
| `JIRA_CONFIG_FILE` | Custom configuration file path | - | ❌ |
| `DOTENV_FILENAME` | Custom .env file path | `.env` | ❌ |
| `JIRA_HOT_RELOAD` | Enable hot-reloading | - | ❌ |

## Secret Types

### Plain Text
```toml
personal_access_token = { type = "plain", value = "your_token_here" }
```

### Base64 Encoded
```toml
personal_access_token = { type = "base64", value = "eW91cl90b2tlbg==" }
```

### Environment Variable Reference
```toml
personal_access_token = { type = "env", value = "JIRA_TOKEN" }
```

### File Reference
```toml
personal_access_token = { type = "file", value = "/path/to/token/file" }
```

## Error Handling

The system provides detailed error messages:

```rust
use rust_jira_mcp::config::ConfigValidationError;

match config.validate() {
    Ok(()) => println!("Configuration is valid"),
    Err(e) => {
        match e.downcast_ref::<ConfigValidationError>() {
            Some(ConfigValidationError::MissingRequiredField(field)) => {
                println!("Required field '{}' is missing", field);
            }
            Some(ConfigValidationError::InvalidEmail(email)) => {
                println!("Invalid email format: '{}'", email);
            }
            Some(ConfigValidationError::ValidationFailed(errors)) => {
                println!("Multiple validation errors:");
                for error in errors {
                    println!("  - {}", error);
                }
            }
            _ => println!("Configuration error: {}", e),
        }
    }
}
```

## Hot-Reloading

Enable hot-reloading to automatically reload configuration when files change:

```rust
let options = ConfigOptions {
    hot_reload: true,
    watch_paths: vec![
        PathBuf::from("config/local.toml"),
        PathBuf::from(".env"),
    ],
    strict_validation: true,
    fail_on_missing: true,
};
```

## Security Best Practices

1. **Never commit secrets to version control**
2. **Use environment variables for sensitive data in production**
3. **Use base64 encoding for secrets in config files**
4. **Set appropriate file permissions on secret files**
5. **Use separate config files for different environments**

## File Structure

```
project/
├── .env                          # Environment variables
├── config/
│   ├── default.toml             # Default configuration
│   ├── local.toml               # Local overrides
│   ├── production.toml          # Production configuration
│   └── secrets.toml             # Secret configuration
└── src/
    └── config/
        ├── mod.rs               # Configuration module
        ├── jira.rs              # Jira-specific config
        ├── manager.rs           # Configuration manager
        ├── secrets.rs           # Secret management
        └── validation.rs        # Configuration validation
```

## Troubleshooting

### Common Issues

1. **"Required field 'email' is missing"**
   - Set `JIRA_EMAIL` environment variable or add to config file

2. **"Invalid email format"**
   - Ensure email contains '@' and '.' characters

3. **"Configuration file not found"**
   - Check file path and permissions
   - Ensure file exists and is readable

4. **"Failed to deserialize configuration"**
   - Check TOML/JSON/YAML syntax
   - Verify field names match expected schema

### Debug Mode

Enable debug logging to see configuration loading details:

```bash
RUST_LOG=debug cargo run
```

This will show:
- Configuration sources being loaded
- Validation results
- Secret resolution
- Hot-reload events
