use crate::logging::config::{LogFormat, LogLevel, LoggingConfig};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Set up the tracing subscriber with the given configuration
pub fn setup_logging(config: &LoggingConfig) {
    // Create environment filter
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| match config.level {
        LogLevel::Trace => EnvFilter::new("trace"),
        LogLevel::Debug => EnvFilter::new("debug"),
        LogLevel::Info => EnvFilter::new("info"),
        LogLevel::Warn => EnvFilter::new("warn"),
        LogLevel::Error => EnvFilter::new("error"),
    });

    // Create console layer
    let console_layer = if config.console_enabled {
        match config.format {
            LogFormat::Json => fmt::layer().json().boxed(),
            LogFormat::Pretty => fmt::layer().pretty().boxed(),
            LogFormat::Compact => fmt::layer().compact().boxed(),
        }
    } else {
        fmt::layer().with_writer(std::io::sink).boxed()
    };

    // Initialize the subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .init();
}
