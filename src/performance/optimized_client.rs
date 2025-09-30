use crate::config::JiraConfig;
use crate::error::{JiraError, Result};
use crate::performance::{CacheManager, PerformanceMetrics};
// Removed object_pool dependency - using Vec instead
use reqwest::{Client, Method, RequestBuilder};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

/// Optimized Jira HTTP Client with connection pooling and object reuse
#[derive(Clone)]
pub struct OptimizedJiraClient {
    client: Client,
    config: JiraConfig,
    cache_manager: Arc<CacheManager>,
    metrics: Arc<PerformanceMetrics>,
}

impl OptimizedJiraClient {
    /// Create a new optimized Jira client
    pub fn new(config: JiraConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout_duration())
            .danger_accept_invalid_certs(!config.strict_ssl.unwrap_or(true))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .map_err(JiraError::HttpClientError)?;

        Ok(Self {
            client,
            config,
            cache_manager: Arc::new(CacheManager::new()),
            metrics: Arc::new(PerformanceMetrics::new()),
        })
    }

    /// Make an optimized GET request with caching
    pub async fn get_cached<T>(&self, endpoint: &str) -> Result<T>
    where
        T: DeserializeOwned + Clone + Send + Sync + Serialize + 'static,
    {
        let cache_key = crate::performance::CacheKeyGenerator::api_response(endpoint, "");

        // Try cache first
        if let Some(cached) = self.cache_manager.api_responses.get(&cache_key).await {
            debug!("Cache hit for endpoint: {}", endpoint);
            self.metrics.record_cache_hit();
            return serde_json::from_value(cached).map_err(JiraError::SerializationError);
        }

        debug!("Cache miss for endpoint: {}", endpoint);
        self.metrics.record_cache_miss();

        // Make the request
        let result = self.get_uncached(endpoint).await?;

        // Cache the result
        let json_value = serde_json::to_value(&result).map_err(JiraError::SerializationError)?;
        self.cache_manager
            .api_responses
            .insert(cache_key, json_value)
            .await;

        Ok(result)
    }

    /// Make a GET request without caching
    pub async fn get_uncached<T>(&self, endpoint: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request(Method::GET, endpoint, None::<&()>).await
    }

    /// Make a POST request with optimized serialization
    pub async fn post_optimized<T, U>(&self, endpoint: &str, body: &U) -> Result<T>
    where
        T: DeserializeOwned,
        U: Serialize + ?Sized,
    {
        self.request(Method::POST, endpoint, Some(body)).await
    }

    /// Make a generic HTTP request with optimized error handling
    async fn request<T, U>(&self, method: Method, endpoint: &str, body: Option<&U>) -> Result<T>
    where
        T: DeserializeOwned,
        U: Serialize + ?Sized,
    {
        let start_time = std::time::Instant::now();

        let url = self.build_url(endpoint)?;
        let request_builder = self.build_request(method, &url, body)?;

        debug!("Making request to {}", url);

        match request_builder.send().await {
            Ok(response) => {
                let status = response.status();
                debug!("Response status: {}", status);

                if status.is_success() {
                    let response_text =
                        response.text().await.map_err(JiraError::HttpClientError)?;
                    debug!("Response body length: {} bytes", response_text.len());

                    let result = serde_json::from_str(&response_text).map_err(|e| {
                        warn!("Failed to parse JSON response: {}", e);
                        JiraError::SerializationError(e)
                    });

                    // Record metrics
                    let duration = start_time.elapsed();
                    self.metrics.record_request(duration, result.is_ok());

                    result
                } else {
                    let error_text = response.text().await.map_err(JiraError::HttpClientError)?;
                    warn!("HTTP error {}: {}", status, error_text);

                    let error_json: serde_json::Value = serde_json::from_str(&error_text)
                        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

                    let jira_error = JiraError::from_jira_response(status, &error_json);

                    // Record metrics
                    let duration = start_time.elapsed();
                    self.metrics.record_request(duration, false);

                    Err(jira_error)
                }
            }
            Err(e) => {
                warn!("Request failed: {}", e);

                // Record metrics
                let duration = start_time.elapsed();
                self.metrics.record_request(duration, false);

                Err(JiraError::HttpClientError(e))
            }
        }
    }

    /// Build a complete URL from the endpoint
    fn build_url(&self, endpoint: &str) -> Result<url::Url> {
        let base_url = url::Url::parse(&self.config.api_base_url)
            .map_err(|e| JiraError::config_error(&format!("Invalid API base URL: {e}")))?;

        base_url
            .join(endpoint)
            .map_err(|e| JiraError::config_error(&format!("Invalid endpoint URL: {e}")))
    }

    /// Build a request with optimized headers
    fn build_request<U>(
        &self,
        method: Method,
        url: &url::Url,
        body: Option<&U>,
    ) -> Result<RequestBuilder>
    where
        U: Serialize + ?Sized,
    {
        let mut request = self
            .client
            .request(method, url.as_str())
            .header("Authorization", self.config.auth_header())
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Connection", "keep-alive")
            .header("Keep-Alive", "timeout=90, max=1000");

        if let Some(body) = body {
            let json_body = serde_json::to_string(body).map_err(JiraError::SerializationError)?;
            debug!("Request body length: {} bytes", json_body.len());
            request = request.body(json_body);
        }

        Ok(request)
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> Arc<PerformanceMetrics> {
        self.metrics.clone()
    }

    /// Get cache manager
    pub fn get_cache_manager(&self) -> Arc<CacheManager> {
        self.cache_manager.clone()
    }

    /// Clear all caches
    pub async fn clear_caches(&self) {
        self.cache_manager.clear_all().await;
    }

    /// Log performance statistics
    pub fn log_performance_stats(&self) {
        self.metrics.log_stats();
    }
}

/// Memory-optimized string builder for reducing allocations
pub struct OptimizedStringBuilder {
    buffer: String,
}

impl OptimizedStringBuilder {
    /// Create a new optimized string builder
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: String::with_capacity(capacity),
        }
    }

    /// Append a string slice
    pub fn push_str(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    /// Append a character
    pub fn push(&mut self, c: char) {
        self.buffer.push(c);
    }

    /// Build the final string
    pub fn build(self) -> String {
        self.buffer
    }

    /// Get current length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

// Removed ObjectPool - using simplified approach

/// Memory usage tracker
pub struct MemoryTracker {
    peak_usage: std::sync::atomic::AtomicUsize,
    current_usage: std::sync::atomic::AtomicUsize,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new() -> Self {
        Self {
            peak_usage: std::sync::atomic::AtomicUsize::new(0),
            current_usage: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Track memory allocation
    pub fn track_allocation(&self, size: usize) {
        let current = self
            .current_usage
            .fetch_add(size, std::sync::atomic::Ordering::Relaxed);
        let new_total = current + size;

        // Update peak usage
        loop {
            let peak = self.peak_usage.load(std::sync::atomic::Ordering::Relaxed);
            if new_total <= peak {
                break;
            }
            if self
                .peak_usage
                .compare_exchange_weak(
                    peak,
                    new_total,
                    std::sync::atomic::Ordering::Relaxed,
                    std::sync::atomic::Ordering::Relaxed,
                )
                .is_ok()
            {
                break;
            }
        }
    }

    /// Track memory deallocation
    pub fn track_deallocation(&self, size: usize) {
        self.current_usage
            .fetch_sub(size, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get current memory usage
    pub fn get_current_usage(&self) -> usize {
        self.current_usage
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get peak memory usage
    pub fn get_peak_usage(&self) -> usize {
        self.peak_usage.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}
