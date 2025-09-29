use serde::{Deserialize, Serialize};

/// Log level configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

/// Log format configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::Pretty
    }
}

/// Simplified logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level for the application
    pub level: LogLevel,
    /// Log format (json, pretty, compact)
    pub format: LogFormat,
    /// Whether to enable console logging
    pub console_enabled: bool,
    /// Whether to enable file logging
    pub file_enabled: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Pretty,
            console_enabled: true,
            file_enabled: false,
        }
    }
}

impl LoggingConfig {
    /// Create a production configuration
    #[must_use]
    pub fn production() -> Self {
        Self {
            level: LogLevel::Warn,
            format: LogFormat::Json,
            console_enabled: true,
            file_enabled: true,
        }
    }

    /// Create a development configuration
    #[must_use]
    pub fn development() -> Self {
        Self {
            level: LogLevel::Debug,
            format: LogFormat::Pretty,
            console_enabled: true,
            file_enabled: false,
        }
    }
}
