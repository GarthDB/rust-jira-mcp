use crate::config::JiraConfig;
use crate::error::{JiraError, Result};
use crate::performance::{CacheManager, PerformanceMetrics, OptimizedJiraClient as BaseClient, CacheKeyGenerator};
use crate::types::jira::{
    JiraIssue, JiraSearchResult, JiraComment, JiraTransition,
    BulkOperationConfig, BulkOperationSummary, BulkOperationItem,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Optimized Jira Client with performance enhancements
pub struct OptimizedJiraClient {
    base_client: BaseClient,
    config: JiraConfig,
    metrics: Arc<PerformanceMetrics>,
    cache_manager: Arc<CacheManager>,
}

impl OptimizedJiraClient {
    /// Create a new optimized Jira client
    pub fn new(config: JiraConfig) -> Result<Self> {
        let base_client = BaseClient::new(config.clone())?;
        let metrics = base_client.get_metrics();
        let cache_manager = base_client.get_cache_manager();

        Ok(Self {
            base_client,
            config,
            metrics,
            cache_manager,
        })
    }

    /// Get a Jira issue with caching
    pub async fn get_issue(&self, issue_key: &str) -> Result<JiraIssue> {
        let endpoint = format!("issue/{}", issue_key);
        self.base_client.get_cached(&endpoint).await
    }

    /// Search for Jira issues with optimized caching
    pub async fn search_issues(
        &self,
        jql: &str,
        start_at: Option<i32>,
        max_results: Option<i32>,
    ) -> Result<JiraSearchResult> {
        let start_at = start_at.unwrap_or(0);
        let max_results = max_results.unwrap_or_else(|| self.config.max_results.unwrap_or(50) as i32);
        
        let cache_key = CacheKeyGenerator::search(jql, start_at, max_results);
        
        // Try cache first
        if let Some(cached) = self.cache_manager.parsed_objects.get(&cache_key).await {
            debug!("Cache hit for search: {}", jql);
            self.metrics.record_cache_hit();
            return serde_json::from_value(cached).map_err(JiraError::SerializationError);
        }

        debug!("Cache miss for search: {}", jql);
        self.metrics.record_cache_miss();

        // Build query parameters efficiently
        let start_at_str = start_at.to_string();
        let max_results_str = max_results.to_string();
        let mut params = Vec::with_capacity(3);
        params.push(("jql", jql));
        params.push(("startAt", start_at_str.as_str()));
        params.push(("maxResults", max_results_str.as_str()));

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let endpoint = format!("search?{}", query_string);
        let result = self.base_client.get_uncached(&endpoint).await?;
        
        // Cache the result
        let json_value = serde_json::to_value(&result).map_err(JiraError::SerializationError)?;
        self.cache_manager.parsed_objects.insert(cache_key, json_value).await;
        
        Ok(result)
    }

    /// Create a new Jira issue with optimized serialization
    pub async fn create_issue(&self, issue_data: &serde_json::Value) -> Result<JiraIssue> {
        self.base_client.post_optimized("issue", issue_data).await
    }

    /// Update a Jira issue with optimized error handling
    pub async fn update_issue(
        &self,
        issue_key: &str,
        update_data: &serde_json::Value,
    ) -> Result<()> {
        let endpoint = format!("issue/{}", issue_key);
        let _: serde_json::Value = self.base_client.post_optimized(&endpoint, update_data).await?;
        Ok(())
    }

    /// Add a comment with optimized serialization
    pub async fn add_comment(&self, issue_key: &str, comment_body: &str) -> Result<JiraComment> {
        let endpoint = format!("issue/{}/comment", issue_key);
        let comment_data = json!({
            "body": comment_body
        });
        self.base_client.post_optimized(&endpoint, &comment_data).await
    }

    /// Get comments with caching
    pub async fn get_comments(&self, issue_key: &str) -> Result<Vec<JiraComment>> {
        let endpoint = format!("issue/{}/comment", issue_key);
        let response: serde_json::Value = self.base_client.get_cached(&endpoint).await?;

        // Extract comments efficiently
        let comments = response
            .get("comments")
            .and_then(|c| c.as_array())
            .ok_or_else(|| JiraError::api_error("Invalid comments response format"))?;

        let mut result = Vec::with_capacity(comments.len());
        for comment in comments {
            let comment: JiraComment =
                serde_json::from_value(comment.clone()).map_err(JiraError::SerializationError)?;
            result.push(comment);
        }

        Ok(result)
    }

    /// Get transitions with caching
    pub async fn get_transitions(&self, issue_key: &str) -> Result<Vec<JiraTransition>> {
        let endpoint = format!("issue/{}/transitions", issue_key);
        let response: serde_json::Value = self.base_client.get_cached(&endpoint).await?;

        // Extract transitions efficiently
        let transitions = response
            .get("transitions")
            .and_then(|t| t.as_array())
            .ok_or_else(|| JiraError::api_error("Invalid transitions response format"))?;

        let mut result = Vec::with_capacity(transitions.len());
        for transition in transitions {
            let transition: JiraTransition = serde_json::from_value(transition.clone())
                .map_err(JiraError::SerializationError)?;
            result.push(transition);
        }

        Ok(result)
    }

    /// Transition an issue with optimized serialization
    pub async fn transition_issue(
        &self,
        issue_key: &str,
        transition_id: &str,
        comment: Option<&str>,
    ) -> Result<()> {
        let endpoint = format!("issue/{}/transitions", issue_key);

        let mut transition_data = json!({
            "transition": {
                "id": transition_id
            }
        });

        if let Some(comment_text) = comment {
            transition_data["update"]["comment"][0]["add"]["body"] =
                serde_json::Value::String(comment_text.to_string());
        }

        let _: serde_json::Value = self.base_client.post_optimized(&endpoint, &transition_data).await?;
        Ok(())
    }

    /// Execute bulk operations with optimized batching
    pub async fn execute_bulk_operations(
        &self,
        operations: Vec<BulkOperationItem>,
        config: BulkOperationConfig,
    ) -> Result<BulkOperationSummary> {
        let start_time = std::time::Instant::now();
        let mut summary = BulkOperationSummary::new();

        info!("Starting optimized bulk operation with {} items", operations.len());

        // Validate operation count
        if operations.len() > 100 {
            return Err(JiraError::api_error(
                "Maximum 100 operations allowed per bulk request",
            ));
        }

        // Process operations in optimized batches
        let batch_size = config.batch_size.unwrap_or(10);
        let mut processed = 0;

        for chunk in operations.chunks(batch_size) {
            let batch_results = self.process_batch_optimized(chunk, &config).await;

            for result in batch_results {
                summary.add_result(result);
            }

            processed += chunk.len();
            info!("Processed {}/{} operations", processed, operations.len());

            // Apply rate limiting between batches
            if let Some(rate_limit_ms) = config.rate_limit_ms {
                tokio::time::sleep(Duration::from_millis(rate_limit_ms)).await;
            }
        }

        summary.duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);

        info!(
            "Optimized bulk operation completed: {} successful, {} failed, {:.1}% success rate",
            summary.successful_operations,
            summary.failed_operations,
            summary.success_rate()
        );

        Ok(summary)
    }

    /// Process a batch of operations with optimizations
    async fn process_batch_optimized(
        &self,
        operations: &[BulkOperationItem],
        config: &BulkOperationConfig,
    ) -> Vec<crate::types::jira::BulkOperationResult> {
        let mut results = Vec::with_capacity(operations.len());

        // Process operations concurrently for better performance
        let handles: Vec<_> = operations
            .iter()
            .map(|operation| {
                let client = self.clone();
                let config = config.clone();
                let operation = operation.clone();
                tokio::spawn(async move {
                    client.execute_single_operation_optimized(&operation, &config).await
                })
            })
            .collect();

        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Task failed: {}", e);
                    // Add a failed result
                    results.push(crate::types::jira::BulkOperationResult {
                        issue_key: "unknown".to_string(),
                        success: false,
                        error_message: Some(format!("Task failed: {}", e)),
                        operation_type: crate::types::jira::BulkOperationType::Update,
                    });
                }
            }
        }

        results
    }

    /// Execute a single operation with optimizations
    async fn execute_single_operation_optimized(
        &self,
        operation: &BulkOperationItem,
        config: &BulkOperationConfig,
    ) -> crate::types::jira::BulkOperationResult {
        let mut retry_count = 0;
        let max_retries = config.max_retries.unwrap_or(3);

        loop {
            match self.perform_operation_optimized(operation).await {
                Ok(()) => {
                    return crate::types::jira::BulkOperationResult {
                        issue_key: operation.issue_key.clone(),
                        success: true,
                        error_message: None,
                        operation_type: operation.operation_type.clone(),
                    };
                }
                Err(e) => {
                    if retry_count < max_retries && self.should_retry_operation(&e) {
                        retry_count += 1;
                        let delay = Duration::from_millis(
                            1000 * u64::try_from(retry_count.max(0)).unwrap_or(0),
                        );
                        warn!(
                            "Retrying operation for {} in {:?} (attempt {}/{})",
                            operation.issue_key, delay, retry_count, max_retries
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    return crate::types::jira::BulkOperationResult {
                        issue_key: operation.issue_key.clone(),
                        success: false,
                        error_message: Some(e.to_string()),
                        operation_type: operation.operation_type.clone(),
                    };
                }
            }
        }
    }

    /// Perform the actual operation with optimizations
    async fn perform_operation_optimized(
        &self,
        operation: &BulkOperationItem,
    ) -> Result<()> {
        match operation.operation_type {
            crate::types::jira::BulkOperationType::Update => {
                let fields = &operation.data;
                self.update_issue(&operation.issue_key, fields).await
            }
            crate::types::jira::BulkOperationType::Transition => {
                let transition_id = operation
                    .data
                    .get("transition_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        JiraError::api_error("Missing transition_id in operation data")
                    })?;
                let comment = operation.data.get("comment").and_then(|v| v.as_str());
                self.transition_issue(&operation.issue_key, transition_id, comment)
                    .await
            }
            crate::types::jira::BulkOperationType::AddComment => {
                let comment_body = operation
                    .data
                    .get("comment_body")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        JiraError::api_error("Missing comment_body in operation data")
                    })?;
                self.add_comment(&operation.issue_key, comment_body).await?;
                Ok(())
            }
            crate::types::jira::BulkOperationType::Mixed => {
                // Handle mixed operations
                if operation.data.get("fields").is_some() {
                    self.update_issue(&operation.issue_key, &operation.data).await
                } else if operation.data.get("transition_id").is_some() {
                    let transition_id = operation
                        .data
                        .get("transition_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            JiraError::api_error("Missing transition_id in operation data")
                        })?;
                    let comment = operation.data.get("comment").and_then(|v| v.as_str());
                    self.transition_issue(&operation.issue_key, transition_id, comment)
                        .await
                } else if operation.data.get("comment_body").is_some() {
                    let comment_body = operation
                        .data
                        .get("comment_body")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            JiraError::api_error("Missing comment_body in operation data")
                        })?;
                    self.add_comment(&operation.issue_key, comment_body).await?;
                    Ok(())
                } else {
                    Err(JiraError::api_error(
                        "Unable to determine operation type from data",
                    ))
                }
            }
        }
    }

    /// Determine if an operation should be retried
    fn should_retry_operation(&self, error: &JiraError) -> bool {
        match error {
            JiraError::HttpClientError(e) => e.is_timeout(),
            JiraError::ApiError { message, .. } => {
                message.contains("timeout")
                    || message.contains("rate limit")
                    || message.contains("too many requests")
            }
            _ => false,
        }
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

// Implement Clone for the client
impl Clone for OptimizedJiraClient {
    fn clone(&self) -> Self {
        Self {
            base_client: self.base_client.clone(),
            config: self.config.clone(),
            metrics: self.metrics.clone(),
            cache_manager: self.cache_manager.clone(),
        }
    }
}
