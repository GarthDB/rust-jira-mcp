use crate::config::JiraConfig;
use crate::error::{JiraError, Result};
use crate::types::jira::{JiraComment, JiraIssue, JiraProject, JiraSearchResult, JiraTransition};
use reqwest::{Client, Method, RequestBuilder};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
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
}
