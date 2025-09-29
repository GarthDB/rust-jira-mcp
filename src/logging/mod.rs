pub mod config;
pub mod metrics;
pub mod tracing_setup;

pub use config::LoggingConfig;
pub use metrics::MetricsCollector;
pub use tracing_setup::setup_logging;

use std::collections::HashMap;

/// Comprehensive logging utilities for the Jira MCP server
pub struct Logger {
    metrics_collector: MetricsCollector,
}

impl Logger {
    /// Create a new logger instance
    #[must_use]
    pub fn new(metrics_collector: MetricsCollector) -> Self {
        Self { metrics_collector }
    }

    /// Log operation success with timing
    pub fn log_operation_success(
        &self,
        operation: &str,
        duration: std::time::Duration,
        metadata: &HashMap<String, String>,
    ) {
        tracing::debug!(
            operation = %operation,
            duration_ms = u64::try_from(duration.as_millis()).unwrap_or(u64::MAX),
            ?metadata,
            "Operation completed successfully"
        );

        std::mem::drop(
            self.metrics_collector
                .record_operation_success(operation, duration, metadata),
        );
    }
}
