use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

/// Performance metrics collector for monitoring system performance
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    // Request metrics
    pub total_requests: Arc<AtomicU64>,
    pub successful_requests: Arc<AtomicU64>,
    pub failed_requests: Arc<AtomicU64>,
    
    // Timing metrics
    pub total_request_time_ms: Arc<AtomicU64>,
    pub min_request_time_ms: Arc<AtomicU64>,
    pub max_request_time_ms: Arc<AtomicU64>,
    
    // Memory metrics
    pub current_memory_usage_bytes: Arc<AtomicUsize>,
    pub peak_memory_usage_bytes: Arc<AtomicUsize>,
    
    // Cache metrics
    pub cache_hits: Arc<AtomicU64>,
    pub cache_misses: Arc<AtomicU64>,
    
    // Rate limiting metrics
    pub rate_limited_requests: Arc<AtomicU64>,
    
    // Start time for calculating averages
    pub start_time: Instant,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    /// Create a new performance metrics collector
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
            total_request_time_ms: Arc::new(AtomicU64::new(0)),
            min_request_time_ms: Arc::new(AtomicU64::new(u64::MAX)),
            max_request_time_ms: Arc::new(AtomicU64::new(0)),
            current_memory_usage_bytes: Arc::new(AtomicUsize::new(0)),
            peak_memory_usage_bytes: Arc::new(AtomicUsize::new(0)),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            rate_limited_requests: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }
    
    /// Record a request completion
    pub fn record_request(&self, duration: Duration, success: bool) {
        let duration_ms = u64::try_from(duration.as_millis()).unwrap_or(u64::MAX);
        
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        
        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
        
        self.total_request_time_ms.fetch_add(duration_ms, Ordering::Relaxed);
        
        // Update min/max times atomically
        loop {
            let current_min = self.min_request_time_ms.load(Ordering::Relaxed);
            if duration_ms >= current_min {
                break;
            }
            if self.min_request_time_ms.compare_exchange_weak(
                current_min,
                duration_ms,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ).is_ok() {
                break;
            }
        }
        
        loop {
            let current_max = self.max_request_time_ms.load(Ordering::Relaxed);
            if duration_ms <= current_max {
                break;
            }
            if self.max_request_time_ms.compare_exchange_weak(
                current_max,
                duration_ms,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ).is_ok() {
                break;
            }
        }
    }
    
    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a rate limited request
    pub fn record_rate_limited(&self) {
        self.rate_limited_requests.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Update memory usage
    pub fn update_memory_usage(&self, current_bytes: usize) {
        self.current_memory_usage_bytes.store(current_bytes, Ordering::Relaxed);
        
        // Update peak memory usage
        loop {
            let current_peak = self.peak_memory_usage_bytes.load(Ordering::Relaxed);
            if current_bytes <= current_peak {
                break;
            }
            if self.peak_memory_usage_bytes.compare_exchange_weak(
                current_peak,
                current_bytes,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ).is_ok() {
                break;
            }
        }
    }
    
    /// Get current performance statistics
    #[must_use]
    pub fn get_stats(&self) -> PerformanceStats {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let successful_requests = self.successful_requests.load(Ordering::Relaxed);
        let failed_requests = self.failed_requests.load(Ordering::Relaxed);
        let total_time_ms = self.total_request_time_ms.load(Ordering::Relaxed);
        let min_time_ms = self.min_request_time_ms.load(Ordering::Relaxed);
        let max_time_ms = self.max_request_time_ms.load(Ordering::Relaxed);
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);
        let rate_limited = self.rate_limited_requests.load(Ordering::Relaxed);
        
        let uptime = self.start_time.elapsed();
        let requests_per_second = if uptime.as_secs() > 0 {
            (total_requests as f64) / (uptime.as_secs() as f64)
        } else {
            0.0
        };
        
        let average_response_time_ms = if total_requests > 0 {
            (total_time_ms as f64) / (total_requests as f64)
        } else {
            0.0
        };
        
        let success_rate = if total_requests > 0 {
            ((successful_requests as f64) / (total_requests as f64)) * 100.0
        } else {
            0.0
        };
        
        let cache_hit_rate = if cache_hits + cache_misses > 0 {
            (cache_hits as f64 / (cache_hits + cache_misses) as f64) * 100.0
        } else {
            0.0
        };
        
        PerformanceStats {
            total_requests,
            successful_requests,
            failed_requests,
            requests_per_second,
            average_response_time_ms,
            min_response_time_ms: if min_time_ms == u64::MAX { 0 } else { min_time_ms },
            max_response_time_ms: max_time_ms,
            success_rate,
            cache_hits,
            cache_misses,
            cache_hit_rate,
            rate_limited_requests: rate_limited,
            current_memory_usage_bytes: self.current_memory_usage_bytes.load(Ordering::Relaxed),
            peak_memory_usage_bytes: self.peak_memory_usage_bytes.load(Ordering::Relaxed),
            uptime_seconds: uptime.as_secs(),
        }
    }
    
    /// Log current performance statistics
    pub fn log_stats(&self) {
        let stats = self.get_stats();
        
        info!("Performance Statistics:");
        info!("  Total Requests: {}", stats.total_requests);
        info!("  Successful: {} ({:.1}%)", stats.successful_requests, stats.success_rate);
        info!("  Failed: {} ({:.1}%)", stats.failed_requests, 100.0 - stats.success_rate);
        info!("  Requests/sec: {:.2}", stats.requests_per_second);
        info!("  Avg Response Time: {:.2}ms", stats.average_response_time_ms);
        info!("  Min Response Time: {}ms", stats.min_response_time_ms);
        info!("  Max Response Time: {}ms", stats.max_response_time_ms);
        info!("  Cache Hit Rate: {:.1}% ({} hits, {} misses)", 
              stats.cache_hit_rate, stats.cache_hits, stats.cache_misses);
        info!("  Rate Limited: {}", stats.rate_limited_requests);
        info!("  Memory Usage: {} bytes (peak: {} bytes)", 
              stats.current_memory_usage_bytes, stats.peak_memory_usage_bytes);
        info!("  Uptime: {} seconds", stats.uptime_seconds);
    }
    
    /// Reset all metrics
    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.successful_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        self.total_request_time_ms.store(0, Ordering::Relaxed);
        self.min_request_time_ms.store(u64::MAX, Ordering::Relaxed);
        self.max_request_time_ms.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.rate_limited_requests.store(0, Ordering::Relaxed);
    }
}

/// Performance statistics snapshot
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
    pub success_rate: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f64,
    pub rate_limited_requests: u64,
    pub current_memory_usage_bytes: usize,
    pub peak_memory_usage_bytes: usize,
    pub uptime_seconds: u64,
}

/// Performance monitoring wrapper for async operations
pub struct PerformanceMonitor {
    metrics: Arc<PerformanceMetrics>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    #[must_use]
    pub fn new(metrics: Arc<PerformanceMetrics>) -> Self {
        Self { metrics }
    }
    
    /// Monitor an async operation
    pub async fn monitor<F, T>(&self, operation: F) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        let start_time = Instant::now();
        let result = operation.await;
        let duration = start_time.elapsed();
        
        let success = result.is_ok();
        self.metrics.record_request(duration, success);
        
        result
    }
    
    /// Monitor a sync operation
    pub fn monitor_sync<F, T>(&self, operation: F) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> Result<T, Box<dyn std::error::Error + Send + Sync>>,
    {
        let start_time = Instant::now();
        let result = operation();
        let duration = start_time.elapsed();
        
        let success = result.is_ok();
        self.metrics.record_request(duration, success);
        
        result
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_METRICS: Arc<PerformanceMetrics> = Arc::new(PerformanceMetrics::new());
}

/// Get the global performance metrics instance
#[must_use]
pub fn get_global_metrics() -> Arc<PerformanceMetrics> {
    GLOBAL_METRICS.clone()
}
