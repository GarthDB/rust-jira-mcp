pub mod async_optimizer;
pub mod cache;
pub mod jira_client;
pub mod metrics;
pub mod monitoring;
pub mod optimized_client;
pub mod profiler;

pub use async_optimizer::{
    get_global_task_manager, AsyncBatchProcessor, AsyncConnectionPool, AsyncRateLimiter,
    AsyncTaskManager,
};
pub use cache::{
    get_global_cache_manager, CacheKeyGenerator, CacheManager, CacheStats, CacheStore,
    CachedOperation, MokaCache,
};
pub use jira_client::JiraClientOptimized;
pub use metrics::{get_global_metrics, PerformanceMetrics, PerformanceStats};
pub use monitoring::{
    get_global_performance_monitor, Alert, AlertStats, AlertThresholds, PerformanceMonitor,
};
pub use optimized_client::OptimizedJiraClient;
pub use profiler::{Profiler, TimingSegment};
