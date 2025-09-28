use crate::config::JiraConfig;
use crate::error::{JiraError, Result};
use crate::types::jira::{
    BulkOperationConfig, BulkOperationItem, BulkOperationResult, BulkOperationSummary,
    BulkOperationType, JiraComment, JiraIssue, JiraProject, JiraSearchResult, JiraTransition,
};
use reqwest::{Client, Method, RequestBuilder};
use serde::{de::DeserializeOwned, Serialize};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use url::Url;

/// Jira HTTP Client with comprehensive API support
pub struct JiraClient {
    client: Client,
    config: JiraConfig,
    rate_limiter: RateLimiter,
}

/// Simple rate limiter to respect Jira API limits
struct RateLimiter {
    last_request: std::sync::Mutex<std::time::Instant>,
    min_interval: Duration,
}

impl RateLimiter {
    fn new(min_interval: Duration) -> Self {
        Self {
            last_request: std::sync::Mutex::new(
                std::time::Instant::now()
                    .checked_sub(min_interval)
                    .unwrap_or_else(std::time::Instant::now),
            ),
            min_interval,
        }
    }

    async fn wait_if_needed(&self) {
        let now = std::time::Instant::now();
        let elapsed = {
            let last_request = self.last_request.lock().unwrap();
            now.duration_since(*last_request)
        };

        if elapsed < self.min_interval {
            let sleep_duration = self.min_interval - elapsed;
            debug!("Rate limiting: sleeping for {:?}", sleep_duration);
            tokio::time::sleep(sleep_duration).await;
        }

        {
            let mut last_request = self.last_request.lock().unwrap();
            *last_request = std::time::Instant::now();
        }
    }
}

impl JiraClient {
    /// Create a new Jira client with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(config: JiraConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout_duration())
            .danger_accept_invalid_certs(!config.strict_ssl.unwrap_or(true))
            .build()
            .map_err(JiraError::HttpClientError)?;

        let rate_limiter = RateLimiter::new(Duration::from_millis(100)); // 10 requests per second max

        Ok(Self {
            client,
            config,
            rate_limiter,
        })
    }

    /// Get the API base URL from the configuration.
    #[must_use]
    pub fn api_base_url(&self) -> &str {
        &self.config.api_base_url
    }

    /// Get the authentication header from the configuration.
    #[must_use]
    pub fn auth_header(&self) -> String {
        self.config.auth_header()
    }

    /// Get a reference to the HTTP client.
    #[must_use]
    pub fn http_client(&self) -> &Client {
        &self.client
    }

    /// Make a GET request to the Jira API
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get<T>(&self, endpoint: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request(Method::GET, endpoint, None::<&()>).await
    }

    /// Make a POST request to the Jira API
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn post<T, U>(&self, endpoint: &str, body: &U) -> Result<T>
    where
        T: DeserializeOwned,
        U: Serialize + ?Sized,
    {
        self.request(Method::POST, endpoint, Some(body)).await
    }

    /// Make a PUT request to the Jira API
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn put<T, U>(&self, endpoint: &str, body: &U) -> Result<T>
    where
        T: DeserializeOwned,
        U: Serialize + ?Sized,
    {
        self.request(Method::PUT, endpoint, Some(body)).await
    }

    /// Make a DELETE request to the Jira API
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn delete<T>(&self, endpoint: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request(Method::DELETE, endpoint, None::<&()>).await
    }

    /// Make a generic HTTP request with retry logic
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails after all retries or the response cannot be parsed.
    async fn request<T, U>(&self, method: Method, endpoint: &str, body: Option<&U>) -> Result<T>
    where
        T: DeserializeOwned,
        U: Serialize + ?Sized,
    {
        let url = self.build_url(endpoint)?;
        let mut retry_count = 0;
        let max_retries = 3;

        loop {
            // Apply rate limiting
            self.rate_limiter.wait_if_needed().await;

            let request_builder = self.build_request(method.clone(), &url, body)?;

            info!("Making {} request to {}", method, url);
            // Note: RequestBuilder doesn't expose headers directly, so we skip this debug log
            // debug!("Request headers: {:?}", request_builder.headers());

            match request_builder.send().await {
                Ok(response) => {
                    let status = response.status();
                    debug!("Response status: {}", status);

                    if status.is_success() {
                        let response_text =
                            response.text().await.map_err(JiraError::HttpClientError)?;

                        debug!("Response body: {}", response_text);

                        return serde_json::from_str(&response_text).map_err(|e| {
                            error!("Failed to parse JSON response: {}", e);
                            JiraError::SerializationError(e)
                        });
                    }

                    let error_text = response.text().await.map_err(JiraError::HttpClientError)?;

                    error!("HTTP error {}: {}", status, error_text);

                    // Parse Jira error response
                    let error_json: serde_json::Value = serde_json::from_str(&error_text)
                        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

                    let jira_error = JiraError::from_jira_response(status, &error_json);

                    // Retry on certain status codes
                    if retry_count < max_retries && Self::should_retry(status) {
                        retry_count += 1;
                        // retry_count is always positive (starts at 0, only incremented)
                        let delay = Duration::from_millis(1000 * retry_count as u64);
                        warn!(
                            "Retrying request in {:?} (attempt {}/{})",
                            delay, retry_count, max_retries
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    return Err(jira_error);
                }
                Err(e) => {
                    error!("Request failed: {}", e);

                    if retry_count < max_retries && e.is_timeout() {
                        retry_count += 1;
                        // retry_count is always positive (starts at 0, only incremented)
                        let delay = Duration::from_millis(1000 * retry_count as u64);
                        warn!(
                            "Retrying request after timeout in {:?} (attempt {}/{})",
                            delay, retry_count, max_retries
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    return Err(JiraError::HttpClientError(e));
                }
            }
        }
    }

    /// Build a complete URL from the endpoint
    fn build_url(&self, endpoint: &str) -> Result<Url> {
        let base_url =
            Url::parse(&self.config.api_base_url).map_err(|e| JiraError::ConfigError {
                message: format!("Invalid API base URL: {e}"),
            })?;

        base_url.join(endpoint).map_err(|e| JiraError::ConfigError {
            message: format!("Invalid endpoint URL: {e}"),
        })
    }

    /// Build a request with proper headers and authentication
    fn build_request<U>(
        &self,
        method: Method,
        url: &Url,
        body: Option<&U>,
    ) -> Result<RequestBuilder>
    where
        U: Serialize + ?Sized,
    {
        let mut request = self
            .client
            .request(method, url.as_str())
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        if let Some(body) = body {
            let json_body = serde_json::to_string(body).map_err(JiraError::SerializationError)?;
            debug!("Request body: {}", json_body);
            request = request.body(json_body);
        }

        Ok(request)
    }

    /// Determine if a request should be retried based on the HTTP status code
    #[must_use]
    pub fn should_retry(status: reqwest::StatusCode) -> bool {
        matches!(
            status.as_u16(),
            429 | // Too Many Requests
            500 | // Internal Server Error
            502 | // Bad Gateway
            503 | // Service Unavailable
            504 // Gateway Timeout
        )
    }

    // High-level API methods for common Jira operations

    /// Get a Jira issue by key
    ///
    /// # Errors
    ///
    /// Returns an error if the issue cannot be found or the request fails.
    pub async fn get_issue(&self, issue_key: &str) -> Result<JiraIssue> {
        let endpoint = format!("issue/{issue_key}");
        self.get(&endpoint).await
    }

    /// Search for Jira issues using JQL
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails or the response cannot be parsed.
    pub async fn search_issues(
        &self,
        jql: &str,
        start_at: Option<i32>,
        max_results: Option<i32>,
    ) -> Result<JiraSearchResult> {
        let mut params = vec![("jql".to_string(), jql.to_string())];

        if let Some(start) = start_at {
            params.push(("startAt".to_string(), start.to_string()));
        }

        if let Some(max) = max_results {
            params.push(("maxResults".to_string(), max.to_string()));
        } else if let Some(default_max) = self.config.max_results {
            params.push(("maxResults".to_string(), default_max.to_string()));
        }

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let endpoint = format!("search?{query_string}");
        self.get(&endpoint).await
    }

    /// Get a Jira project by key
    ///
    /// # Errors
    ///
    /// Returns an error if the project cannot be found or the request fails.
    pub async fn get_project(&self, project_key: &str) -> Result<JiraProject> {
        let endpoint = format!("project/{project_key}");
        self.get(&endpoint).await
    }

    /// Get all projects
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_projects(&self) -> Result<Vec<JiraProject>> {
        self.get("project").await
    }

    /// Create a new Jira issue
    ///
    /// # Errors
    ///
    /// Returns an error if the issue creation fails or the response cannot be parsed.
    pub async fn create_issue(&self, issue_data: &serde_json::Value) -> Result<JiraIssue> {
        self.post("issue", issue_data).await
    }

    /// Update a Jira issue
    ///
    /// # Errors
    ///
    /// Returns an error if the issue update fails or the response cannot be parsed.
    pub async fn update_issue(
        &self,
        issue_key: &str,
        update_data: &serde_json::Value,
    ) -> Result<()> {
        let endpoint = format!("issue/{issue_key}");
        let _: serde_json::Value = self.put(&endpoint, update_data).await?;
        Ok(())
    }

    /// Add a comment to a Jira issue
    ///
    /// # Errors
    ///
    /// Returns an error if the comment creation fails or the response cannot be parsed.
    pub async fn add_comment(&self, issue_key: &str, comment_body: &str) -> Result<JiraComment> {
        let endpoint = format!("issue/{issue_key}/comment");
        let comment_data = serde_json::json!({
            "body": comment_body
        });
        self.post(&endpoint, &comment_data).await
    }

    /// Get comments for a Jira issue
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_comments(&self, issue_key: &str) -> Result<Vec<JiraComment>> {
        let endpoint = format!("issue/{issue_key}/comment");
        let response: serde_json::Value = self.get(&endpoint).await?;

        // Extract comments from the response
        let comments = response
            .get("comments")
            .and_then(|c| c.as_array())
            .ok_or_else(|| JiraError::ApiError {
                message: "Invalid comments response format".to_string(),
            })?;

        let mut result = Vec::new();
        for comment in comments {
            let comment: JiraComment =
                serde_json::from_value(comment.clone()).map_err(JiraError::SerializationError)?;
            result.push(comment);
        }

        Ok(result)
    }

    /// Get available transitions for a Jira issue
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_transitions(&self, issue_key: &str) -> Result<Vec<JiraTransition>> {
        let endpoint = format!("issue/{issue_key}/transitions");
        let response: serde_json::Value = self.get(&endpoint).await?;

        // Extract transitions from the response
        let transitions = response
            .get("transitions")
            .and_then(|t| t.as_array())
            .ok_or_else(|| JiraError::ApiError {
                message: "Invalid transitions response format".to_string(),
            })?;

        let mut result = Vec::new();
        for transition in transitions {
            let transition: JiraTransition = serde_json::from_value(transition.clone())
                .map_err(JiraError::SerializationError)?;
            result.push(transition);
        }

        Ok(result)
    }

    /// Transition a Jira issue to a new status
    ///
    /// # Errors
    ///
    /// Returns an error if the transition fails or the response cannot be parsed.
    pub async fn transition_issue(
        &self,
        issue_key: &str,
        transition_id: &str,
        comment: Option<&str>,
    ) -> Result<()> {
        let endpoint = format!("issue/{issue_key}/transitions");

        let mut transition_data = serde_json::json!({
            "transition": {
                "id": transition_id
            }
        });

        if let Some(comment_text) = comment {
            transition_data["update"]["comment"][0]["add"]["body"] =
                serde_json::Value::String(comment_text.to_string());
        }

        let _: serde_json::Value = self.post(&endpoint, &transition_data).await?;
        Ok(())
    }

    // Project Configuration and Metadata methods

    /// Get project configuration details
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_project_configuration(&self, project_key: &str) -> Result<serde_json::Value> {
        let endpoint = format!("project/{project_key}/configuration");
        self.get(&endpoint).await
    }

    /// Get issue types for a project
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_project_issue_types(
        &self,
        project_key: &str,
    ) -> Result<Vec<crate::types::jira::JiraIssueType>> {
        let endpoint = format!("project/{project_key}");
        let project: serde_json::Value = self.get(&endpoint).await?;

        let issue_types = project
            .get("issueTypes")
            .and_then(|it| it.as_array())
            .ok_or_else(|| JiraError::ApiError {
                message: "Invalid project response format - missing issueTypes".to_string(),
            })?;

        let mut result = Vec::new();
        for issue_type in issue_types {
            let issue_type: crate::types::jira::JiraIssueType =
                serde_json::from_value(issue_type.clone())
                    .map_err(JiraError::SerializationError)?;
            result.push(issue_type);
        }

        Ok(result)
    }

    /// Get issue type metadata by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_issue_type_metadata(
        &self,
        issue_type_id: &str,
    ) -> Result<crate::types::jira::JiraIssueType> {
        let endpoint = format!("issuetype/{issue_type_id}");
        self.get(&endpoint).await
    }

    /// Get all issue types
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_issue_types(&self) -> Result<Vec<crate::types::jira::JiraIssueType>> {
        self.get("issuetype").await
    }

    /// Get project components
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_project_components(
        &self,
        project_key: &str,
    ) -> Result<Vec<crate::types::jira::JiraComponent>> {
        let endpoint = format!("project/{project_key}/components");
        self.get(&endpoint).await
    }

    /// Get all priorities
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_priorities(&self) -> Result<Vec<crate::types::jira::JiraPriority>> {
        self.get("priority").await
    }

    /// Get all statuses
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_statuses(&self) -> Result<Vec<crate::types::jira::JiraStatus>> {
        self.get("status").await
    }

    /// Get custom fields
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_custom_fields(&self) -> Result<Vec<serde_json::Value>> {
        let endpoint = "field";
        let response: serde_json::Value = self.get(endpoint).await?;

        let fields = response.as_array().ok_or_else(|| JiraError::ApiError {
            message: "Invalid custom fields response format".to_string(),
        })?;

        Ok(fields.clone())
    }

    /// Get project metadata including all configuration details
    ///
    /// # Errors
    ///
    /// Returns an error if any of the metadata requests fail.
    pub async fn get_project_metadata(&self, project_key: &str) -> Result<serde_json::Value> {
        let (configuration, issue_types, components, priorities, statuses, custom_fields) = tokio::try_join!(
            self.get_project_configuration(project_key),
            self.get_project_issue_types(project_key),
            self.get_project_components(project_key),
            self.get_priorities(),
            self.get_statuses(),
            self.get_custom_fields()
        )?;

        Ok(serde_json::json!({
            "project_key": project_key,
            "configuration": configuration,
            "issue_types": issue_types,
            "components": components,
            "priorities": priorities,
            "statuses": statuses,
            "custom_fields": custom_fields
        }))
    }

    // Bulk Operations

    /// Execute bulk operations on multiple Jira issues
    ///
    /// # Errors
    ///
    /// Returns an error if the bulk operation fails or cannot be processed.
    pub async fn execute_bulk_operations(
        &self,
        operations: Vec<BulkOperationItem>,
        config: BulkOperationConfig,
    ) -> Result<BulkOperationSummary> {
        let start_time = Instant::now();
        let mut summary = BulkOperationSummary::new();
        
        info!("Starting bulk operation with {} items", operations.len());

        // Validate that we don't exceed the maximum number of operations
        if operations.len() > 100 {
            return Err(JiraError::ApiError {
                message: "Maximum 100 operations allowed per bulk request".to_string(),
            });
        }

        // Process operations in batches
        let batch_size = config.batch_size.unwrap_or(10);
        let mut processed = 0;

        for chunk in operations.chunks(batch_size) {
            let batch_results = self.process_batch(chunk, &config).await;
            
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

        summary.duration_ms = start_time.elapsed().as_millis() as u64;
        
        info!(
            "Bulk operation completed: {} successful, {} failed, {:.1}% success rate",
            summary.successful_operations,
            summary.failed_operations,
            summary.success_rate()
        );

        Ok(summary)
    }

    /// Process a batch of operations
    async fn process_batch(
        &self,
        operations: &[BulkOperationItem],
        config: &BulkOperationConfig,
    ) -> Vec<BulkOperationResult> {
        let mut results = Vec::new();

        for operation in operations {
            let result = self.execute_single_operation(operation, config).await;
            results.push(result);

            // Apply rate limiting between individual operations
            if let Some(rate_limit_ms) = config.rate_limit_ms {
                tokio::time::sleep(Duration::from_millis(rate_limit_ms)).await;
            }
        }

        results
    }

    /// Execute a single operation within a bulk operation
    async fn execute_single_operation(
        &self,
        operation: &BulkOperationItem,
        config: &BulkOperationConfig,
    ) -> BulkOperationResult {
        let mut retry_count = 0;
        let max_retries = config.max_retries.unwrap_or(3);

        loop {
            match self.perform_operation(operation).await {
                Ok(_) => {
                    return BulkOperationResult {
                        issue_key: operation.issue_key.clone(),
                        success: true,
                        error_message: None,
                        operation_type: operation.operation_type.clone(),
                    };
                }
                Err(e) => {
                    if retry_count < max_retries && self.should_retry_operation(&e) {
                        retry_count += 1;
                        let delay = Duration::from_millis(1000 * retry_count as u64);
                        warn!(
                            "Retrying operation for {} in {:?} (attempt {}/{})",
                            operation.issue_key, delay, retry_count, max_retries
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    return BulkOperationResult {
                        issue_key: operation.issue_key.clone(),
                        success: false,
                        error_message: Some(e.to_string()),
                        operation_type: operation.operation_type.clone(),
                    };
                }
            }
        }
    }

    /// Perform the actual operation based on the operation type
    async fn perform_operation(&self, operation: &BulkOperationItem) -> Result<()> {
        match operation.operation_type {
            BulkOperationType::Update => {
                let fields = &operation.data;
                self.update_issue(&operation.issue_key, fields).await
            }
            BulkOperationType::Transition => {
                let transition_id = operation.data
                    .get("transition_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JiraError::ApiError {
                        message: "Missing transition_id in operation data".to_string(),
                    })?;
                let comment = operation.data
                    .get("comment")
                    .and_then(|v| v.as_str());
                self.transition_issue(&operation.issue_key, transition_id, comment).await
            }
            BulkOperationType::AddComment => {
                let comment_body = operation.data
                    .get("comment_body")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JiraError::ApiError {
                        message: "Missing comment_body in operation data".to_string(),
                    })?;
                self.add_comment(&operation.issue_key, comment_body).await?;
                Ok(())
            }
            BulkOperationType::Mixed => {
                // For mixed operations, we need to determine the operation type from the data
                if operation.data.get("fields").is_some() {
                    // This is an update operation
                    self.update_issue(&operation.issue_key, &operation.data).await
                } else if operation.data.get("transition_id").is_some() {
                    // This is a transition operation
                    let transition_id = operation.data
                        .get("transition_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| JiraError::ApiError {
                            message: "Missing transition_id in operation data".to_string(),
                        })?;
                    let comment = operation.data
                        .get("comment")
                        .and_then(|v| v.as_str());
                    self.transition_issue(&operation.issue_key, transition_id, comment).await
                } else if operation.data.get("comment_body").is_some() {
                    // This is a comment operation
                    let comment_body = operation.data
                        .get("comment_body")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| JiraError::ApiError {
                            message: "Missing comment_body in operation data".to_string(),
                        })?;
                    self.add_comment(&operation.issue_key, comment_body).await?;
                    Ok(())
                } else {
                    Err(JiraError::ApiError {
                        message: "Unable to determine operation type from data".to_string(),
                    })
                }
            }
        }
    }

    /// Determine if an operation should be retried based on the error
    fn should_retry_operation(&self, error: &JiraError) -> bool {
        match error {
            JiraError::HttpClientError(e) => e.is_timeout(),
            JiraError::ApiError { message } => {
                message.contains("timeout") || 
                message.contains("rate limit") ||
                message.contains("too many requests")
            }
            _ => false,
        }
    }

    /// Bulk update multiple issues with the same fields
    ///
    /// # Errors
    ///
    /// Returns an error if the bulk update fails or cannot be processed.
    pub async fn bulk_update_issues(
        &self,
        issue_keys: Vec<String>,
        fields: serde_json::Value,
        config: Option<BulkOperationConfig>,
    ) -> Result<BulkOperationSummary> {
        let config = config.unwrap_or_default();
        let operations = issue_keys
            .into_iter()
            .map(|issue_key| BulkOperationItem {
                issue_key,
                operation_type: BulkOperationType::Update,
                data: fields.clone(),
            })
            .collect();

        self.execute_bulk_operations(operations, config).await
    }

    /// Bulk transition multiple issues to the same status
    ///
    /// # Errors
    ///
    /// Returns an error if the bulk transition fails or cannot be processed.
    pub async fn bulk_transition_issues(
        &self,
        issue_keys: Vec<String>,
        transition_id: String,
        comment: Option<String>,
        config: Option<BulkOperationConfig>,
    ) -> Result<BulkOperationSummary> {
        let config = config.unwrap_or_default();
        let mut operation_data = serde_json::json!({
            "transition_id": transition_id
        });
        
        if let Some(comment_text) = comment {
            operation_data["comment"] = serde_json::Value::String(comment_text);
        }

        let operations = issue_keys
            .into_iter()
            .map(|issue_key| BulkOperationItem {
                issue_key,
                operation_type: BulkOperationType::Transition,
                data: operation_data.clone(),
            })
            .collect();

        self.execute_bulk_operations(operations, config).await
    }

    /// Bulk add comments to multiple issues
    ///
    /// # Errors
    ///
    /// Returns an error if the bulk comment operation fails or cannot be processed.
    pub async fn bulk_add_comments(
        &self,
        issue_keys: Vec<String>,
        comment_body: String,
        config: Option<BulkOperationConfig>,
    ) -> Result<BulkOperationSummary> {
        let config = config.unwrap_or_default();
        let operation_data = serde_json::json!({
            "comment_body": comment_body
        });

        let operations = issue_keys
            .into_iter()
            .map(|issue_key| BulkOperationItem {
                issue_key,
                operation_type: BulkOperationType::AddComment,
                data: operation_data.clone(),
            })
            .collect();

        self.execute_bulk_operations(operations, config).await
    }
}
