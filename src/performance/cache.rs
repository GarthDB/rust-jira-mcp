use moka::future::Cache;
use std::hash::Hash;
use std::time::Duration;
use tracing::{debug, info};

/// Generic cache trait for different cache implementations
#[async_trait::async_trait]
pub trait CacheStore<K, V>: Send + Sync
where
    K: Hash + Eq + Send + Sync + Clone,
    V: Send + Sync + Clone,
{
    async fn get(&self, key: &K) -> Option<V>;
    async fn insert(&self, key: K, value: V) -> Option<V>;
    async fn remove(&self, key: &K) -> Option<V>;
    async fn clear(&self);
    async fn len(&self) -> usize;
    async fn is_empty(&self) -> bool;
}

/// Moka-based cache implementation
pub struct MokaCache<K, V> {
    cache: Cache<K, V>,
}

impl<K, V> MokaCache<K, V>
where
    K: Hash + Eq + Send + Sync + Clone + 'static,
    V: Send + Sync + Clone + 'static,
{
    /// Create a new Moka cache with the specified capacity and TTL
    #[must_use]
    pub fn new(capacity: u64, ttl: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(capacity)
            .time_to_live(ttl)
            .build();
        
        Self { cache }
    }
    
    /// Create a new Moka cache with default settings
    #[must_use]
    pub fn new_default() -> Self {
        Self::new(1000, Duration::from_secs(300)) // 5 minutes TTL
    }
}

#[async_trait::async_trait]
impl<K, V> CacheStore<K, V> for MokaCache<K, V>
where
    K: Hash + Eq + Send + Sync + Clone + 'static,
    V: Send + Sync + Clone + 'static,
{
    async fn get(&self, key: &K) -> Option<V> {
        self.cache.get(key).await
    }
    
    async fn insert(&self, key: K, value: V) -> Option<V> {
        self.cache.insert(key, value).await;
        None
    }
    
    async fn remove(&self, key: &K) -> Option<V> {
        self.cache.remove(key).await
    }
    
    async fn clear(&self) {
        self.cache.invalidate_all();
    }
    
    async fn len(&self) -> usize {
        usize::try_from(self.cache.entry_count()).unwrap_or(0)
    }
    
    async fn is_empty(&self) -> bool {
        self.cache.entry_count() == 0
    }
}

/// Cache manager for different types of cached data
pub struct CacheManager {
    // Cache for API responses
    pub api_responses: Box<dyn CacheStore<String, serde_json::Value>>,
    // Cache for parsed objects
    pub parsed_objects: Box<dyn CacheStore<String, serde_json::Value>>,
    // Cache for configuration data
    pub config_cache: Box<dyn CacheStore<String, serde_json::Value>>,
}

impl CacheManager {
    /// Create a new cache manager with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            api_responses: Box::new(MokaCache::new(500, Duration::from_secs(300))), // 5 minutes
            parsed_objects: Box::new(MokaCache::new(1000, Duration::from_secs(600))), // 10 minutes
            config_cache: Box::new(MokaCache::new(100, Duration::from_secs(3600))), // 1 hour
        }
    }
    
    /// Create a new cache manager with custom settings
    #[must_use]
    pub fn with_settings(
        api_cache_capacity: u64,
        api_cache_ttl: Duration,
        parsed_cache_capacity: u64,
        parsed_cache_ttl: Duration,
        config_cache_capacity: u64,
        config_cache_ttl: Duration,
    ) -> Self {
        Self {
            api_responses: Box::new(MokaCache::new(api_cache_capacity, api_cache_ttl)),
            parsed_objects: Box::new(MokaCache::new(parsed_cache_capacity, parsed_cache_ttl)),
            config_cache: Box::new(MokaCache::new(config_cache_capacity, config_cache_ttl)),
        }
    }
    
    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        CacheStats {
            api_responses_count: self.api_responses.len().await,
            parsed_objects_count: self.parsed_objects.len().await,
            config_cache_count: self.config_cache.len().await,
        }
    }
    
    /// Clear all caches
    pub async fn clear_all(&self) {
        self.api_responses.clear().await;
        self.parsed_objects.clear().await;
        self.config_cache.clear().await;
        info!("All caches cleared");
    }
    
    /// Log cache statistics
    pub async fn log_stats(&self) {
        let stats = self.get_stats().await;
        info!("Cache Statistics:");
        info!("  API Responses: {} entries", stats.api_responses_count);
        info!("  Parsed Objects: {} entries", stats.parsed_objects_count);
        info!("  Config Cache: {} entries", stats.config_cache_count);
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub api_responses_count: usize,
    pub parsed_objects_count: usize,
    pub config_cache_count: usize,
}

/// Cache key generator for consistent key formatting
pub struct CacheKeyGenerator;

impl CacheKeyGenerator {
    /// Generate a cache key for API responses
    #[must_use]
    pub fn api_response(endpoint: &str, params: &str) -> String {
        format!("api:{endpoint}:{params}")
    }
    
    /// Generate a cache key for parsed objects
    #[must_use]
    pub fn parsed_object(object_type: &str, id: &str) -> String {
        format!("parsed:{object_type}:{id}")
    }
    
    /// Generate a cache key for configuration
    #[must_use]
    pub fn config(config_type: &str) -> String {
        format!("config:{config_type}")
    }
    
    /// Generate a cache key for search results
    #[must_use]
    pub fn search(jql: &str, start_at: i32, max_results: i32) -> String {
        format!("search:{jql}:{start_at}:{max_results}")
    }
}

/// Cache wrapper with performance metrics
pub struct CachedOperation<T> {
    cache: Box<dyn CacheStore<String, T>>,
    metrics: Option<crate::performance::PerformanceMetrics>,
}

impl<T> CachedOperation<T>
where
    T: Send + Sync + Clone + 'static,
{
    /// Create a new cached operation
    #[must_use]
    pub fn new(cache: Box<dyn CacheStore<String, T>>, metrics: Option<crate::performance::PerformanceMetrics>) -> Self {
        Self { cache, metrics }
    }
    
    /// Execute an operation with caching
    pub async fn execute<F, Fut>(&self, key: String, operation: F) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        // Try to get from cache first
        if let Some(cached_value) = self.cache.get(&key).await {
            debug!("Cache hit for key: {}", key);
            if let Some(ref metrics) = self.metrics {
                metrics.record_cache_hit();
            }
            return Ok(cached_value);
        }
        
        debug!("Cache miss for key: {}", key);
        if let Some(ref metrics) = self.metrics {
            metrics.record_cache_miss();
        }
        
        // Execute the operation
        let result = operation().await?;
        
        // Store in cache
        self.cache.insert(key, result.clone()).await;
        
        Ok(result)
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_CACHE_MANAGER: CacheManager = CacheManager::new();
}

/// Get the global cache manager instance
#[must_use]
pub fn get_global_cache_manager() -> &'static CacheManager {
    &GLOBAL_CACHE_MANAGER
}
