use crate::performance::PerformanceMetrics;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{error, info, warn};

/// Performance monitoring and alerting system
pub struct PerformanceMonitor {
    metrics: Arc<PerformanceMetrics>,
    alerts: Arc<RwLock<Vec<Alert>>>,
    thresholds: AlertThresholds,
    monitoring_active: Arc<RwLock<bool>>,
}

/// Alert thresholds for different metrics
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub max_response_time_ms: u64,
    pub min_success_rate_percent: f64,
    pub max_memory_usage_bytes: usize,
    pub max_error_rate_percent: f64,
    pub max_requests_per_second: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_response_time_ms: 5000,                // 5 seconds
            min_success_rate_percent: 95.0,            // 95%
            max_memory_usage_bytes: 100 * 1024 * 1024, // 100MB
            max_error_rate_percent: 5.0,               // 5%
            max_requests_per_second: 1000.0,           // 1000 RPS
        }
    }
}

/// Alert types
#[derive(Debug, Clone)]
pub enum AlertType {
    HighResponseTime,
    LowSuccessRate,
    HighMemoryUsage,
    HighErrorRate,
    HighRequestRate,
    SystemOverload,
}

/// Alert information
#[derive(Debug, Clone)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: Instant,
    pub resolved: bool,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    #[must_use]
    pub fn new(metrics: Arc<PerformanceMetrics>, thresholds: AlertThresholds) -> Self {
        Self {
            metrics,
            alerts: Arc::new(RwLock::new(Vec::new())),
            thresholds,
            monitoring_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Start monitoring with the specified interval
    pub async fn start_monitoring(&self, check_interval: Duration) {
        let mut monitoring_active = self.monitoring_active.write().await;
        if *monitoring_active {
            warn!("Monitoring is already active");
            return;
        }
        *monitoring_active = true;
        drop(monitoring_active);

        info!(
            "Starting performance monitoring with {}s interval",
            check_interval.as_secs()
        );

        let metrics = self.metrics.clone();
        let alerts = self.alerts.clone();
        let thresholds = self.thresholds.clone();
        let monitoring_active = self.monitoring_active.clone();

        tokio::spawn(async move {
            let mut interval = interval(check_interval);

            while *monitoring_active.read().await {
                interval.tick().await;

                let stats = metrics.get_stats();
                let mut alerts_guard = alerts.write().await;

                // Check for various alert conditions
                Self::check_response_time_alert(&stats, &thresholds, &mut alerts_guard);
                Self::check_success_rate_alert(&stats, &thresholds, &mut alerts_guard);
                Self::check_memory_usage_alert(&stats, &thresholds, &mut alerts_guard);
                Self::check_error_rate_alert(&stats, &thresholds, &mut alerts_guard);
                Self::check_request_rate_alert(&stats, &thresholds, &mut alerts_guard);
                Self::check_system_overload_alert(&stats, &thresholds, &mut alerts_guard);

                // Clean up old alerts
                Self::cleanup_old_alerts(&mut alerts_guard);
            }
        });
    }

    /// Stop monitoring
    pub async fn stop_monitoring(&self) {
        let mut monitoring_active = self.monitoring_active.write().await;
        *monitoring_active = false;
        info!("Performance monitoring stopped");
    }

    /// Check for high response time alerts
    #[allow(clippy::cast_precision_loss)]
    fn check_response_time_alert(
        stats: &crate::performance::PerformanceStats,
        thresholds: &AlertThresholds,
        alerts: &mut Vec<Alert>,
    ) {
        if stats.average_response_time_ms > thresholds.max_response_time_ms as f64 {
            let alert = Alert {
                id: format!("high_response_time_{}", Instant::now().elapsed().as_secs()),
                alert_type: AlertType::HighResponseTime,
                message: format!(
                    "Average response time {}ms exceeds threshold {}ms",
                    stats.average_response_time_ms, thresholds.max_response_time_ms
                ),
                severity: if stats.average_response_time_ms
                    > thresholds.max_response_time_ms as f64 * 2.0
                {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::High
                },
                timestamp: Instant::now(),
                resolved: false,
                metadata: std::collections::HashMap::from([
                    (
                        "average_response_time_ms".to_string(),
                        stats.average_response_time_ms.to_string(),
                    ),
                    (
                        "threshold_ms".to_string(),
                        thresholds.max_response_time_ms.to_string(),
                    ),
                ]),
            };
            alerts.push(alert);
            warn!(
                "High response time alert triggered: {}ms",
                stats.average_response_time_ms
            );
        }
    }

    /// Check for low success rate alerts
    fn check_success_rate_alert(
        stats: &crate::performance::PerformanceStats,
        thresholds: &AlertThresholds,
        alerts: &mut Vec<Alert>,
    ) {
        if stats.success_rate < thresholds.min_success_rate_percent {
            let alert = Alert {
                id: format!("low_success_rate_{}", Instant::now().elapsed().as_secs()),
                alert_type: AlertType::LowSuccessRate,
                message: format!(
                    "Success rate {:.1}% below threshold {:.1}%",
                    stats.success_rate, thresholds.min_success_rate_percent
                ),
                severity: if stats.success_rate < thresholds.min_success_rate_percent * 0.5 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::High
                },
                timestamp: Instant::now(),
                resolved: false,
                metadata: std::collections::HashMap::from([
                    ("success_rate".to_string(), stats.success_rate.to_string()),
                    (
                        "threshold".to_string(),
                        thresholds.min_success_rate_percent.to_string(),
                    ),
                ]),
            };
            alerts.push(alert);
            warn!(
                "Low success rate alert triggered: {:.1}%",
                stats.success_rate
            );
        }
    }

    /// Check for high memory usage alerts
    fn check_memory_usage_alert(
        stats: &crate::performance::PerformanceStats,
        thresholds: &AlertThresholds,
        alerts: &mut Vec<Alert>,
    ) {
        if stats.current_memory_usage_bytes > thresholds.max_memory_usage_bytes {
            let alert = Alert {
                id: format!("high_memory_usage_{}", Instant::now().elapsed().as_secs()),
                alert_type: AlertType::HighMemoryUsage,
                message: format!(
                    "Memory usage {} bytes exceeds threshold {} bytes",
                    stats.current_memory_usage_bytes, thresholds.max_memory_usage_bytes
                ),
                severity: if stats.current_memory_usage_bytes
                    > thresholds.max_memory_usage_bytes * 2
                {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::High
                },
                timestamp: Instant::now(),
                resolved: false,
                metadata: std::collections::HashMap::from([
                    (
                        "current_memory_bytes".to_string(),
                        stats.current_memory_usage_bytes.to_string(),
                    ),
                    (
                        "peak_memory_bytes".to_string(),
                        stats.peak_memory_usage_bytes.to_string(),
                    ),
                    (
                        "threshold_bytes".to_string(),
                        thresholds.max_memory_usage_bytes.to_string(),
                    ),
                ]),
            };
            alerts.push(alert);
            warn!(
                "High memory usage alert triggered: {} bytes",
                stats.current_memory_usage_bytes
            );
        }
    }

    /// Check for high error rate alerts
    fn check_error_rate_alert(
        stats: &crate::performance::PerformanceStats,
        thresholds: &AlertThresholds,
        alerts: &mut Vec<Alert>,
    ) {
        let error_rate = 100.0 - stats.success_rate;
        if error_rate > thresholds.max_error_rate_percent {
            let alert = Alert {
                id: format!("high_error_rate_{}", Instant::now().elapsed().as_secs()),
                alert_type: AlertType::HighErrorRate,
                message: format!(
                    "Error rate {:.1}% exceeds threshold {:.1}%",
                    error_rate, thresholds.max_error_rate_percent
                ),
                severity: if error_rate > thresholds.max_error_rate_percent * 2.0 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::High
                },
                timestamp: Instant::now(),
                resolved: false,
                metadata: std::collections::HashMap::from([
                    ("error_rate".to_string(), error_rate.to_string()),
                    ("success_rate".to_string(), stats.success_rate.to_string()),
                    (
                        "threshold".to_string(),
                        thresholds.max_error_rate_percent.to_string(),
                    ),
                ]),
            };
            alerts.push(alert);
            warn!("High error rate alert triggered: {:.1}%", error_rate);
        }
    }

    /// Check for high request rate alerts
    fn check_request_rate_alert(
        stats: &crate::performance::PerformanceStats,
        thresholds: &AlertThresholds,
        alerts: &mut Vec<Alert>,
    ) {
        if stats.requests_per_second > thresholds.max_requests_per_second {
            let alert = Alert {
                id: format!("high_request_rate_{}", Instant::now().elapsed().as_secs()),
                alert_type: AlertType::HighRequestRate,
                message: format!(
                    "Request rate {:.1} RPS exceeds threshold {:.1} RPS",
                    stats.requests_per_second, thresholds.max_requests_per_second
                ),
                severity: if stats.requests_per_second > thresholds.max_requests_per_second * 1.5 {
                    AlertSeverity::High
                } else {
                    AlertSeverity::Medium
                },
                timestamp: Instant::now(),
                resolved: false,
                metadata: std::collections::HashMap::from([
                    (
                        "requests_per_second".to_string(),
                        stats.requests_per_second.to_string(),
                    ),
                    (
                        "threshold".to_string(),
                        thresholds.max_requests_per_second.to_string(),
                    ),
                ]),
            };
            alerts.push(alert);
            warn!(
                "High request rate alert triggered: {:.1} RPS",
                stats.requests_per_second
            );
        }
    }

    /// Check for system overload alerts
    #[allow(clippy::cast_precision_loss)]
    fn check_system_overload_alert(
        stats: &crate::performance::PerformanceStats,
        thresholds: &AlertThresholds,
        alerts: &mut Vec<Alert>,
    ) {
        let is_overloaded = stats.average_response_time_ms
            > thresholds.max_response_time_ms as f64 * 1.5
            && stats.success_rate < thresholds.min_success_rate_percent * 0.8
            && stats.requests_per_second > thresholds.max_requests_per_second * 0.8;

        if is_overloaded {
            let alert = Alert {
                id: format!("system_overload_{}", Instant::now().elapsed().as_secs()),
                alert_type: AlertType::SystemOverload,
                message: "System appears to be overloaded based on multiple metrics".to_string(),
                severity: AlertSeverity::Critical,
                timestamp: Instant::now(),
                resolved: false,
                metadata: std::collections::HashMap::from([
                    (
                        "average_response_time_ms".to_string(),
                        stats.average_response_time_ms.to_string(),
                    ),
                    ("success_rate".to_string(), stats.success_rate.to_string()),
                    (
                        "requests_per_second".to_string(),
                        stats.requests_per_second.to_string(),
                    ),
                    (
                        "memory_usage_bytes".to_string(),
                        stats.current_memory_usage_bytes.to_string(),
                    ),
                ]),
            };
            alerts.push(alert);
            error!("System overload alert triggered!");
        }
    }

    /// Clean up old alerts (older than 1 hour)
    fn cleanup_old_alerts(alerts: &mut Vec<Alert>) {
        let cutoff_time = Instant::now()
            .checked_sub(Duration::from_secs(3600))
            .unwrap(); // 1 hour
        alerts.retain(|alert| alert.timestamp > cutoff_time);
    }

    /// Get all active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts
            .iter()
            .filter(|alert| !alert.resolved)
            .cloned()
            .collect()
    }

    /// Get alerts by severity
    pub async fn get_alerts_by_severity(&self, severity: AlertSeverity) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts
            .iter()
            .filter(|alert| alert.severity == severity && !alert.resolved)
            .cloned()
            .collect()
    }

    /// Resolve an alert by ID
    pub async fn resolve_alert(&self, alert_id: &str) -> bool {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolved = true;
            info!("Alert resolved: {}", alert_id);
            true
        } else {
            false
        }
    }

    /// Get alert statistics
    pub async fn get_alert_stats(&self) -> AlertStats {
        let alerts = self.alerts.read().await;
        let total_alerts = alerts.len();
        let active_alerts = alerts.iter().filter(|a| !a.resolved).count();
        let critical_alerts = alerts
            .iter()
            .filter(|a| a.severity == AlertSeverity::Critical && !a.resolved)
            .count();
        let high_alerts = alerts
            .iter()
            .filter(|a| a.severity == AlertSeverity::High && !a.resolved)
            .count();
        let medium_alerts = alerts
            .iter()
            .filter(|a| a.severity == AlertSeverity::Medium && !a.resolved)
            .count();
        let low_alerts = alerts
            .iter()
            .filter(|a| a.severity == AlertSeverity::Low && !a.resolved)
            .count();

        AlertStats {
            total_alerts,
            active_alerts,
            critical_alerts,
            high_alerts,
            medium_alerts,
            low_alerts,
        }
    }

    /// Log current alert status
    pub async fn log_alert_status(&self) {
        let stats = self.get_alert_stats().await;
        info!("Alert Status:");
        info!("  Total Alerts: {}", stats.total_alerts);
        info!("  Active Alerts: {}", stats.active_alerts);
        info!(
            "  Critical: {}, High: {}, Medium: {}, Low: {}",
            stats.critical_alerts, stats.high_alerts, stats.medium_alerts, stats.low_alerts
        );
    }
}

/// Alert statistics
#[derive(Debug, Clone)]
pub struct AlertStats {
    pub total_alerts: usize,
    pub active_alerts: usize,
    pub critical_alerts: usize,
    pub high_alerts: usize,
    pub medium_alerts: usize,
    pub low_alerts: usize,
}

static GLOBAL_PERFORMANCE_MONITOR: std::sync::LazyLock<PerformanceMonitor> =
    std::sync::LazyLock::new(|| {
        PerformanceMonitor::new(
            crate::performance::get_global_metrics(),
            AlertThresholds::default(),
        )
    });

/// Get the global performance monitor
#[must_use]
pub fn get_global_performance_monitor() -> &'static PerformanceMonitor {
    &GLOBAL_PERFORMANCE_MONITOR
}
