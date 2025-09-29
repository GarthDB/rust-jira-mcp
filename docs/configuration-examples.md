# Configuration Examples

This document provides comprehensive examples for configuring the Rust Jira MCP Server in various environments and scenarios.

## Table of Contents

- [Quick Start Examples](#quick-start-examples)
- [Environment-Specific Configurations](#environment-specific-configurations)
- [Advanced Configuration Patterns](#advanced-configuration-patterns)
- [Secret Management Examples](#secret-management-examples)
- [Hot-Reloading Examples](#hot-reloading-examples)
- [Validation Examples](#validation-examples)
- [Troubleshooting Configuration](#troubleshooting-configuration)

## Quick Start Examples

### Basic Environment Variables

**Minimal Configuration:**
```bash
# .env file
JIRA_EMAIL=your.email@company.com
JIRA_PERSONAL_ACCESS_TOKEN=your_personal_access_token_here
```

**Complete Configuration:**
```bash
# .env file
# Required
JIRA_EMAIL=your.email@company.com
JIRA_PERSONAL_ACCESS_TOKEN=your_personal_access_token_here

# Optional
JIRA_API_BASE_URL=https://your-company.atlassian.net/rest/api/2
JIRA_DEFAULT_PROJECT=PROJ
JIRA_MAX_RESULTS=50
JIRA_TIMEOUT_SECONDS=30
JIRA_STRICT_SSL=true
JIRA_LOG_FILE=~/Desktop/jira-api.log
```

### TOML Configuration

**Basic TOML Configuration:**
```toml
# config/default.toml
[default]
api_base_url = "https://your-company.atlassian.net/rest/api/2"
max_results = 50
timeout_seconds = 30
strict_ssl = true
log_file = "~/Desktop/jira-api.log"
```

**Advanced TOML Configuration:**
```toml
# config/production.toml
[default]
api_base_url = "https://your-company.atlassian.net/rest/api/2"
max_results = 100
timeout_seconds = 60
strict_ssl = true
log_file = "/var/log/jira-mcp.log"

[performance]
connection_pool_size = 20
keep_alive = true
compression_enabled = true

[caching]
cache_enabled = true
cache_ttl = 300
cache_max_size = 1000

[logging]
level = "info"
format = "json"
file = "/var/log/jira-mcp.log"
```

### JSON Configuration

**Basic JSON Configuration:**
```json
{
  "default": {
    "api_base_url": "https://your-company.atlassian.net/rest/api/2",
    "max_results": 50,
    "timeout_seconds": 30,
    "strict_ssl": true,
    "log_file": "~/Desktop/jira-api.log"
  }
}
```

**Advanced JSON Configuration:**
```json
{
  "default": {
    "api_base_url": "https://your-company.atlassian.net/rest/api/2",
    "max_results": 100,
    "timeout_seconds": 60,
    "strict_ssl": true,
    "log_file": "/var/log/jira-mcp.log"
  },
  "performance": {
    "connection_pool_size": 20,
    "keep_alive": true,
    "compression_enabled": true
  },
  "caching": {
    "cache_enabled": true,
    "cache_ttl": 300,
    "cache_max_size": 1000
  },
  "logging": {
    "level": "info",
    "format": "json",
    "file": "/var/log/jira-mcp.log"
  }
}
```

## Environment-Specific Configurations

### Development Environment

**config/development.toml:**
```toml
[default]
api_base_url = "https://your-company-dev.atlassian.net/rest/api/2"
max_results = 25
timeout_seconds = 15
strict_ssl = false
log_file = "~/Desktop/jira-dev.log"

[logging]
level = "debug"
format = "pretty"
console = true

[development]
hot_reload = true
watch_paths = ["config/development.toml", ".env"]
```

**Environment Variables:**
```bash
# .env.development
JIRA_EMAIL=dev@company.com
JIRA_PERSONAL_ACCESS_TOKEN=dev_token_here
JIRA_API_BASE_URL=https://your-company-dev.atlassian.net/rest/api/2
JIRA_STRICT_SSL=false
RUST_LOG=debug
```

### Staging Environment

**config/staging.toml:**
```toml
[default]
api_base_url = "https://your-company-staging.atlassian.net/rest/api/2"
max_results = 50
timeout_seconds = 30
strict_ssl = true
log_file = "/var/log/jira-staging.log"

[performance]
connection_pool_size = 10
keep_alive = true

[logging]
level = "info"
format = "json"
```

**Environment Variables:**
```bash
# .env.staging
JIRA_EMAIL=staging@company.com
JIRA_PERSONAL_ACCESS_TOKEN=staging_token_here
JIRA_API_BASE_URL=https://your-company-staging.atlassian.net/rest/api/2
JIRA_STRICT_SSL=true
```

### Production Environment

**config/production.toml:**
```toml
[default]
api_base_url = "https://your-company.atlassian.net/rest/api/2"
max_results = 100
timeout_seconds = 60
strict_ssl = true
log_file = "/var/log/jira-mcp.log"

[performance]
connection_pool_size = 20
keep_alive = true
compression_enabled = true

[caching]
cache_enabled = true
cache_ttl = 300
cache_max_size = 1000

[logging]
level = "warn"
format = "json"
file = "/var/log/jira-mcp.log"
rotation = "daily"
max_files = 30
```

**Environment Variables:**
```bash
# .env.production
JIRA_EMAIL=production@company.com
JIRA_PERSONAL_ACCESS_TOKEN=production_token_here
JIRA_API_BASE_URL=https://your-company.atlassian.net/rest/api/2
JIRA_STRICT_SSL=true
RUST_LOG=warn
```

## Advanced Configuration Patterns

### Multi-Environment Configuration

**config/default.toml (Base Configuration):**
```toml
[default]
api_base_url = "https://your-company.atlassian.net/rest/api/2"
max_results = 50
timeout_seconds = 30
strict_ssl = true

[performance]
connection_pool_size = 10
keep_alive = true

[logging]
level = "info"
format = "pretty"
```

**config/development.toml (Development Overrides):**
```toml
[default]
api_base_url = "https://your-company-dev.atlassian.net/rest/api/2"
max_results = 25
timeout_seconds = 15
strict_ssl = false

[logging]
level = "debug"
console = true
```

**config/production.toml (Production Overrides):**
```toml
[default]
max_results = 100
timeout_seconds = 60

[performance]
connection_pool_size = 20
compression_enabled = true

[caching]
cache_enabled = true
cache_ttl = 300

[logging]
level = "warn"
format = "json"
file = "/var/log/jira-mcp.log"
```

### Configuration with Profiles

**config/profiles.toml:**
```toml
[profile.development]
api_base_url = "https://dev.atlassian.net/rest/api/2"
max_results = 25
timeout_seconds = 15
strict_ssl = false

[profile.staging]
api_base_url = "https://staging.atlassian.net/rest/api/2"
max_results = 50
timeout_seconds = 30
strict_ssl = true

[profile.production]
api_base_url = "https://atlassian.net/rest/api/2"
max_results = 100
timeout_seconds = 60
strict_ssl = true
```

**Usage:**
```bash
# Use development profile
JIRA_PROFILE=development cargo run

# Use staging profile
JIRA_PROFILE=staging cargo run

# Use production profile
JIRA_PROFILE=production cargo run
```

### Configuration with Inheritance

**config/base.toml:**
```toml
[default]
api_base_url = "https://your-company.atlassian.net/rest/api/2"
timeout_seconds = 30
strict_ssl = true

[performance]
connection_pool_size = 10
keep_alive = true
```

**config/enhanced.toml:**
```toml
# Inherit from base configuration
include = "base.toml"

[default]
max_results = 100
timeout_seconds = 60

[performance]
connection_pool_size = 20
compression_enabled = true

[caching]
cache_enabled = true
cache_ttl = 300
```

## Secret Management Examples

### Environment Variable Secrets

**Basic Environment Variables:**
```bash
# .env
JIRA_EMAIL=your.email@company.com
JIRA_PERSONAL_ACCESS_TOKEN=your_personal_access_token_here
```

**Prefixed Environment Variables:**
```bash
# .env
JIRA_EMAIL=your.email@company.com
JIRA_PERSONAL_ACCESS_TOKEN=your_personal_access_token_here
JIRA_API_BASE_URL=https://your-company.atlassian.net/rest/api/2
```

### TOML Secret Configuration

**config/secrets.toml:**
```toml
[secrets]
# Plain text secrets (not recommended for production)
email = { type = "plain", value = "your.email@company.com" }

# Base64 encoded secrets
personal_access_token = { type = "base64", value = "eW91cl90b2tlbg==" }

# Environment variable references
api_base_url = { type = "env", value = "JIRA_API_BASE_URL" }

# File references
private_key = { type = "file", value = "/path/to/private/key" }
```

### JSON Secret Configuration

**config/secrets.json:**
```json
{
  "secrets": {
    "email": {
      "type": "plain",
      "value": "your.email@company.com"
    },
    "personal_access_token": {
      "type": "base64",
      "value": "eW91cl90b2tlbg=="
    },
    "api_base_url": {
      "type": "env",
      "value": "JIRA_API_BASE_URL"
    },
    "private_key": {
      "type": "file",
      "value": "/path/to/private/key"
    }
  }
}
```

### Secret Management with Multiple Sources

**config/secrets.toml:**
```toml
[secrets]
# Primary secrets from environment
email = { type = "env", value = "JIRA_EMAIL" }
personal_access_token = { type = "env", value = "JIRA_PERSONAL_ACCESS_TOKEN" }

# Fallback secrets from files
backup_token = { type = "file", value = "/etc/jira/backup_token" }

# Encrypted secrets
encrypted_secret = { type = "encrypted", value = "encrypted_value_here" }
```

### Secret Rotation

**config/secrets.toml:**
```toml
[secrets]
# Current token
personal_access_token = { type = "env", value = "JIRA_PERSONAL_ACCESS_TOKEN" }

# Backup token for rotation
backup_token = { type = "env", value = "JIRA_BACKUP_TOKEN" }

# Token rotation configuration
[token_rotation]
enabled = true
check_interval = 3600  # 1 hour
warning_threshold = 86400  # 24 hours
```

## Hot-Reloading Examples

### Basic Hot-Reloading

**config/local.toml:**
```toml
[default]
api_base_url = "https://your-company.atlassian.net/rest/api/2"
max_results = 50
timeout_seconds = 30

[hot_reload]
enabled = true
watch_paths = ["config/local.toml", ".env"]
debounce_ms = 500
```

**Usage:**
```rust
use rust_jira_mcp::config::{ConfigManager, ConfigOptions};
use std::path::PathBuf;

let mut config_manager = ConfigManager::new();
let options = ConfigOptions {
    hot_reload: true,
    watch_paths: vec![
        PathBuf::from("config/local.toml"),
        PathBuf::from(".env"),
    ],
    strict_validation: true,
    fail_on_missing: true,
};
config_manager.load_with_options(options).await?;
```

### Advanced Hot-Reloading

**config/hot_reload.toml:**
```toml
[default]
api_base_url = "https://your-company.atlassian.net/rest/api/2"
max_results = 50
timeout_seconds = 30

[hot_reload]
enabled = true
watch_paths = [
    "config/local.toml",
    "config/secrets.toml",
    ".env",
    ".env.local"
]
debounce_ms = 1000
reload_callback = "on_config_reload"
validate_on_reload = true
backup_on_reload = true
```

**Usage with Callback:**
```rust
use rust_jira_mcp::config::{ConfigManager, ConfigOptions};
use std::path::PathBuf;

let mut config_manager = ConfigManager::new();
let options = ConfigOptions {
    hot_reload: true,
    watch_paths: vec![
        PathBuf::from("config/local.toml"),
        PathBuf::from("config/secrets.toml"),
        PathBuf::from(".env"),
    ],
    strict_validation: true,
    fail_on_missing: true,
    reload_callback: Some(Box::new(|config| {
        println!("Configuration reloaded: {:?}", config);
    })),
};
config_manager.load_with_options(options).await?;
```

## Validation Examples

### Basic Validation

**config/validation.toml:**
```toml
[default]
api_base_url = "https://your-company.atlassian.net/rest/api/2"
max_results = 50
timeout_seconds = 30
strict_ssl = true

[validation]
strict_mode = true
validate_on_load = true
fail_on_invalid = true
```

**Usage:**
```rust
use rust_jira_mcp::config::{JiraConfig, ConfigValidator};

let config = JiraConfig::load_from_file("config/validation.toml").await?;
config.validate()?;
```

### Advanced Validation

**config/advanced_validation.toml:**
```toml
[default]
api_base_url = "https://your-company.atlassian.net/rest/api/2"
max_results = 50
timeout_seconds = 30
strict_ssl = true

[validation]
strict_mode = true
validate_on_load = true
fail_on_invalid = true
custom_validators = [
    "email_format",
    "url_format",
    "timeout_range"
]
```

**Custom Validators:**
```rust
use rust_jira_mcp::config::{ConfigValidator, ValidationRule};

let validator = ConfigValidator::new()
    .add_rule(ValidationRule::new("email".to_string())
        .required()
        .custom_validator(|email| {
            if email.contains('@') && email.contains('.') {
                Ok(())
            } else {
                Err("Invalid email format".to_string())
            }
        }))
    .add_rule(ValidationRule::new("api_base_url".to_string())
        .required()
        .custom_validator(|url| {
            if url.starts_with("https://") {
                Ok(())
            } else {
                Err("API URL must use HTTPS".to_string())
            }
        }))
    .add_rule(ValidationRule::new("timeout_seconds".to_string())
        .required()
        .min_value(1)
        .max_value(300));
```

### Validation with Error Reporting

**config/validation_with_errors.toml:**
```toml
[default]
api_base_url = "https://your-company.atlassian.net/rest/api/2"
max_results = 50
timeout_seconds = 30
strict_ssl = true

[validation]
strict_mode = true
validate_on_load = true
fail_on_invalid = false
error_reporting = true
error_format = "detailed"
```

**Error Handling:**
```rust
use rust_jira_mcp::config::{JiraConfig, ConfigValidationError};

match JiraConfig::load_from_file("config/validation_with_errors.toml").await {
    Ok(config) => {
        match config.validate() {
            Ok(()) => println!("Configuration is valid"),
            Err(e) => {
                match e.downcast_ref::<ConfigValidationError>() {
                    Some(ConfigValidationError::MissingRequiredField(field)) => {
                        eprintln!("Required field '{}' is missing", field);
                    }
                    Some(ConfigValidationError::InvalidEmail(email)) => {
                        eprintln!("Invalid email format: '{}'", email);
                    }
                    Some(ConfigValidationError::ValidationFailed(errors)) => {
                        eprintln!("Multiple validation errors:");
                        for error in errors {
                            eprintln!("  - {}", error);
                        }
                    }
                    _ => eprintln!("Configuration error: {}", e),
                }
            }
        }
    }
    Err(e) => eprintln!("Failed to load configuration: {}", e),
}
```

## Troubleshooting Configuration

### Common Configuration Issues

**1. Missing Required Fields:**
```bash
# Error: Required field 'email' is missing
# Solution: Add to .env or config file
JIRA_EMAIL=your.email@company.com
```

**2. Invalid Configuration Format:**
```bash
# Error: Failed to parse configuration file
# Solution: Check TOML/JSON syntax
toml get config/default.toml
```

**3. Environment Variable Not Found:**
```bash
# Error: Environment variable 'JIRA_EMAIL' not found
# Solution: Set environment variable or use config file
export JIRA_EMAIL=your.email@company.com
```

**4. Secret Resolution Failed:**
```bash
# Error: Failed to resolve secret 'personal_access_token'
# Solution: Check secret configuration
cat config/secrets.toml
```

### Debug Configuration Loading

**Enable Debug Logging:**
```bash
RUST_LOG=debug cargo run
```

**Validate Configuration:**
```bash
cargo run -- --validate-config
```

**Test Configuration:**
```bash
cargo run -- --test-config
```

### Configuration Testing

**Test Script:**
```bash
#!/bin/bash
# test_config.sh

echo "Testing configuration loading..."

# Test environment variables
if [ -z "$JIRA_EMAIL" ]; then
    echo "❌ JIRA_EMAIL not set"
    exit 1
fi

if [ -z "$JIRA_PERSONAL_ACCESS_TOKEN" ]; then
    echo "❌ JIRA_PERSONAL_ACCESS_TOKEN not set"
    exit 1
fi

echo "✅ Environment variables set"

# Test configuration file
if [ ! -f "config/default.toml" ]; then
    echo "❌ config/default.toml not found"
    exit 1
fi

echo "✅ Configuration file found"

# Test configuration parsing
if cargo run -- --validate-config; then
    echo "✅ Configuration is valid"
else
    echo "❌ Configuration validation failed"
    exit 1
fi

echo "✅ All configuration tests passed"
```

### Configuration Backup and Recovery

**Backup Configuration:**
```bash
#!/bin/bash
# backup_config.sh

BACKUP_DIR="config/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

# Backup configuration files
cp config/*.toml "$BACKUP_DIR/"
cp .env "$BACKUP_DIR/.env.$TIMESTAMP"

echo "Configuration backed up to $BACKUP_DIR"
```

**Restore Configuration:**
```bash
#!/bin/bash
# restore_config.sh

BACKUP_DIR="config/backups"
BACKUP_FILE="$1"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file>"
    exit 1
fi

if [ ! -f "$BACKUP_DIR/$BACKUP_FILE" ]; then
    echo "Backup file not found: $BACKUP_DIR/$BACKUP_FILE"
    exit 1
fi

cp "$BACKUP_DIR/$BACKUP_FILE" config/
echo "Configuration restored from $BACKUP_FILE"
```

## Best Practices

### 1. Use Environment Variables for Secrets
```bash
# Good
JIRA_PERSONAL_ACCESS_TOKEN=your_token_here

# Bad
personal_access_token = "your_token_here"  # In config file
```

### 2. Use Configuration Files for Settings
```toml
# Good
[default]
max_results = 50
timeout_seconds = 30

# Bad
JIRA_MAX_RESULTS=50  # In environment
JIRA_TIMEOUT_SECONDS=30  # In environment
```

### 3. Validate Configuration on Startup
```rust
let config = JiraConfig::load_from_env().await?;
config.validate()?;
```

### 4. Use Different Configurations for Different Environments
```
config/
├── default.toml
├── development.toml
├── staging.toml
└── production.toml
```

### 5. Enable Hot-Reloading in Development
```toml
[hot_reload]
enabled = true
watch_paths = ["config/local.toml", ".env"]
```

### 6. Use Structured Logging
```toml
[logging]
level = "info"
format = "json"
file = "/var/log/jira-mcp.log"
```

### 7. Implement Configuration Backup
```bash
# Regular backups
0 2 * * * /path/to/backup_config.sh
```

### 8. Monitor Configuration Changes
```rust
config_manager.on_reload(|config| {
    println!("Configuration changed: {:?}", config);
});
```

Remember: Configuration management is crucial for maintaining a reliable and secure Jira MCP server. Always validate your configuration and use appropriate security measures for sensitive data!
