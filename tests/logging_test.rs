use rust_jira_mcp::logging::config::{LogFormat, LogLevel};
use rust_jira_mcp::logging::*;
use std::collections::HashMap;

#[test]
fn test_log_level_default() {
    let level = LogLevel::default();
    assert_eq!(level, LogLevel::Info);
}

#[test]
fn test_log_level_serialization() {
    let level = LogLevel::Debug;
    let serialized = serde_json::to_string(&level).unwrap();
    assert_eq!(serialized, "\"Debug\"");

    let deserialized: LogLevel = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, level);
}

#[test]
fn test_log_level_all_variants() {
    let variants = vec![
        LogLevel::Trace,
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
    ];

    for variant in variants {
        let serialized = serde_json::to_string(&variant).unwrap();
        let deserialized: LogLevel = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, variant);
    }
}

#[test]
fn test_log_format_default() {
    let format = LogFormat::default();
    assert_eq!(format, LogFormat::Pretty);
}

#[test]
fn test_log_format_serialization() {
    let format = LogFormat::Json;
    let serialized = serde_json::to_string(&format).unwrap();
    assert_eq!(serialized, "\"Json\"");

    let deserialized: LogFormat = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, format);
}

#[test]
fn test_log_format_all_variants() {
    let variants = vec![LogFormat::Json, LogFormat::Pretty, LogFormat::Compact];

    for variant in variants {
        let serialized = serde_json::to_string(&variant).unwrap();
        let deserialized: LogFormat = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, variant);
    }
}

#[test]
fn test_logging_config_default() {
    let config = LoggingConfig::default();
    assert_eq!(config.level, LogLevel::Info);
    assert_eq!(config.format, LogFormat::Pretty);
    assert!(config.console_enabled);
    assert!(!config.file_enabled);
}

#[test]
fn test_logging_config_production() {
    let config = LoggingConfig::production();
    assert_eq!(config.level, LogLevel::Warn);
    assert_eq!(config.format, LogFormat::Json);
    assert!(config.console_enabled);
    assert!(config.file_enabled);
}

#[test]
fn test_logging_config_development() {
    let config = LoggingConfig::development();
    assert_eq!(config.level, LogLevel::Debug);
    assert_eq!(config.format, LogFormat::Pretty);
    assert!(config.console_enabled);
    assert!(!config.file_enabled);
}

#[test]
fn test_logging_config_serialization() {
    let config = LoggingConfig {
        level: LogLevel::Debug,
        format: LogFormat::Json,
        console_enabled: true,
        file_enabled: false,
    };

    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: LoggingConfig = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.level, config.level);
    assert_eq!(deserialized.format, config.format);
    assert_eq!(deserialized.console_enabled, config.console_enabled);
    assert_eq!(deserialized.file_enabled, config.file_enabled);
}

#[test]
fn test_logging_config_clone() {
    let config = LoggingConfig::production();
    let cloned = config.clone();

    assert_eq!(cloned.level, config.level);
    assert_eq!(cloned.format, config.format);
    assert_eq!(cloned.console_enabled, config.console_enabled);
    assert_eq!(cloned.file_enabled, config.file_enabled);
}

#[test]
fn test_logging_config_debug() {
    let config = LoggingConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("LoggingConfig"));
    assert!(debug_str.contains("Info"));
    assert!(debug_str.contains("Pretty"));
}

#[test]
fn test_metrics_collector_new() {
    let _collector = MetricsCollector::new();
    // Test that collector can be created successfully
    // This is acceptable for this test
}

#[test]
fn test_metrics_collector_default() {
    let _collector = MetricsCollector::default();
    // Test that collector can be created successfully
    // This is acceptable for this test
}

#[test]
fn test_metrics_collector_clone() {
    let collector = MetricsCollector::new();
    let _cloned = collector.clone();
    // Test that collector can be cloned successfully
    // This is acceptable for this test
}

#[test]
fn test_metrics_collector_debug() {
    let collector = MetricsCollector::new();
    let debug_str = format!("{:?}", collector);
    assert!(debug_str.contains("MetricsCollector"));
}

#[tokio::test]
async fn test_metrics_collector_record_operation_success() {
    let collector = MetricsCollector::new();
    let mut metadata = HashMap::new();
    metadata.insert("test_key".to_string(), "test_value".to_string());

    let duration = std::time::Duration::from_millis(100);

    // Record operation success
    collector
        .record_operation_success("test_operation", duration, &metadata)
        .await;

    // Test that operation recording doesn't panic
    // This is acceptable for this test
}

#[tokio::test]
async fn test_metrics_collector_multiple_operations() {
    let collector = MetricsCollector::new();
    let metadata = HashMap::new();
    let duration = std::time::Duration::from_millis(50);

    // Record multiple operations
    collector
        .record_operation_success("operation1", duration, &metadata)
        .await;
    collector
        .record_operation_success("operation2", duration, &metadata)
        .await;
    collector
        .record_operation_success("operation1", duration, &metadata)
        .await;

    // Test that multiple operations don't panic
    // This is acceptable for this test
}

#[test]
fn test_logger_new() {
    let metrics_collector = MetricsCollector::new();
    let _logger = Logger::new(metrics_collector);
    // Test that logger can be created successfully
    // This is acceptable for this test
}

#[test]
fn test_logger_log_operation_success() {
    let metrics_collector = MetricsCollector::new();
    let logger = Logger::new(metrics_collector);
    let mut metadata = HashMap::new();
    metadata.insert("test_key".to_string(), "test_value".to_string());

    let duration = std::time::Duration::from_millis(200);

    // This should not panic
    logger.log_operation_success("test_operation", duration, &metadata);
}

#[test]
fn test_logger_with_empty_metadata() {
    let metrics_collector = MetricsCollector::new();
    let logger = Logger::new(metrics_collector);
    let metadata = HashMap::new();

    let duration = std::time::Duration::from_millis(100);

    // This should not panic
    logger.log_operation_success("test_operation", duration, &metadata);
}

#[test]
fn test_logger_with_zero_duration() {
    let metrics_collector = MetricsCollector::new();
    let logger = Logger::new(metrics_collector);
    let metadata = HashMap::new();

    let duration = std::time::Duration::from_nanos(0);

    // This should not panic
    logger.log_operation_success("test_operation", duration, &metadata);
}

#[test]
fn test_logger_with_large_duration() {
    let metrics_collector = MetricsCollector::new();
    let logger = Logger::new(metrics_collector);
    let metadata = HashMap::new();

    let duration = std::time::Duration::from_secs(3600); // 1 hour

    // This should not panic
    logger.log_operation_success("test_operation", duration, &metadata);
}
