use crate::config::JiraConfig;
use crate::error::{JiraError, Result};
use crate::types::jira::{
    BulkOperationConfig, BulkOperationItem, BulkOperationResult, BulkOperationSummary,
    BulkOperationType, JiraAttachment, JiraComment, JiraComponent, JiraComponentCreateRequest,
    JiraComponentUpdateRequest, JiraIssue, JiraIssueCloneRequest, JiraIssueCloneResponse,
    JiraIssueLink, JiraIssueLinkCreateRequest, JiraLabel, JiraLabelCreateRequest,
    JiraLabelUpdateRequest, JiraLinkType, JiraSearchResult, JiraTransition, JiraWatchersResponse,
    JiraWorkLog, JiraWorkLogCreateRequest, JiraWorkLogUpdateRequest, ZephyrTestCase,
    ZephyrTestCaseCreateRequest, ZephyrTestCaseSearchResult, ZephyrTestCycle, ZephyrTestExecution,
    ZephyrTestExecutionCreateRequest, ZephyrTestPlan, ZephyrTestStep, ZephyrTestStepCreateRequest,
    ZephyrTestStepUpdateRequest,
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
                        let delay = Duration::from_millis(
                            1000 * u64::try_from(retry_count.max(0)).unwrap_or(0),
                        );
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
                        let delay = Duration::from_millis(
                            1000 * u64::try_from(retry_count.max(0)).unwrap_or(0),
                        );
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
        let base_url = Url::parse(&self.config.api_base_url)
            .map_err(|e| JiraError::config_error(&format!("Invalid API base URL: {e}")))?;

        base_url
            .join(endpoint)
            .map_err(|e| JiraError::config_error(&format!("Invalid endpoint URL: {e}")))
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
            .ok_or_else(|| JiraError::api_error("Invalid comments response format"))?;

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
            .ok_or_else(|| JiraError::api_error("Invalid transitions response format"))?;

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
            .ok_or_else(|| {
                JiraError::api_error("Invalid project response format - missing issueTypes")
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

        let fields = response
            .as_array()
            .ok_or_else(|| JiraError::api_error("Invalid custom fields response format"))?;

        Ok(fields.clone())
    }

    // Issue Linking Operations
    /// Get all available link types
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_link_types(&self) -> Result<Vec<JiraLinkType>> {
        self.get("issueLinkType").await
    }

    /// Get issue links for a specific issue
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_issue_links(&self, issue_key: &str) -> Result<Vec<JiraIssueLink>> {
        let endpoint = format!("issue/{issue_key}/remotelink");
        let response: serde_json::Value = self.get(&endpoint).await?;

        let links = response
            .as_array()
            .ok_or_else(|| JiraError::api_error("Invalid issue links response format"))?;

        let mut result = Vec::new();
        for link in links {
            let issue_link: JiraIssueLink =
                serde_json::from_value(link.clone()).map_err(JiraError::SerializationError)?;
            result.push(issue_link);
        }

        Ok(result)
    }

    /// Create a link between two issues
    ///
    /// # Errors
    ///
    /// Returns an error if the link creation fails or the response cannot be parsed.
    pub async fn create_issue_link(&self, link_request: &JiraIssueLinkCreateRequest) -> Result<()> {
        let endpoint = "issueLink";
        let _: serde_json::Value = self.post(endpoint, link_request).await?;
        Ok(())
    }

    /// Delete an issue link
    ///
    /// # Errors
    ///
    /// Returns an error if the link deletion fails.
    pub async fn delete_issue_link(&self, link_id: &str) -> Result<()> {
        let endpoint = format!("issueLink/{link_id}");
        let _: serde_json::Value = self.delete(&endpoint).await?;
        Ok(())
    }

    /// Link two issues with a specific link type
    ///
    /// # Errors
    ///
    /// Returns an error if the link creation fails.
    pub async fn link_issues(
        &self,
        inward_issue_key: &str,
        outward_issue_key: &str,
        link_type_name: &str,
        comment: Option<&str>,
    ) -> Result<()> {
        let link_request = JiraIssueLinkCreateRequest {
            link_type: crate::types::jira::JiraIssueLinkType {
                name: link_type_name.to_string(),
            },
            inward_issue: Some(crate::types::jira::JiraIssueLinkTarget {
                key: inward_issue_key.to_string(),
            }),
            outward_issue: Some(crate::types::jira::JiraIssueLinkTarget {
                key: outward_issue_key.to_string(),
            }),
            comment: comment.map(|c| crate::types::jira::JiraIssueLinkComment {
                body: c.to_string(),
                visibility: None,
            }),
        };

        self.create_issue_link(&link_request).await
    }

    // File Attachment Operations

    /// Get attachments for a specific issue
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_issue_attachments(&self, issue_key: &str) -> Result<Vec<JiraAttachment>> {
        let issue = self.get_issue(issue_key).await?;

        let attachments = issue
            .fields
            .get("attachment")
            .and_then(|a| a.as_array())
            .ok_or_else(|| JiraError::api_error("No attachments field found in issue"))?;

        let mut result = Vec::new();
        for attachment in attachments {
            let attachment: JiraAttachment = serde_json::from_value(attachment.clone())
                .map_err(JiraError::SerializationError)?;
            result.push(attachment);
        }

        Ok(result)
    }

    /// Upload an attachment to a Jira issue
    ///
    /// # Errors
    ///
    /// Returns an error if the attachment upload fails or the response cannot be parsed.
    pub async fn upload_attachment(
        &self,
        issue_key: &str,
        filename: &str,
        content: &[u8],
        mime_type: Option<&str>,
    ) -> Result<Vec<JiraAttachment>> {
        let endpoint = format!("issue/{issue_key}/attachments");
        let url = self.build_url(&endpoint)?;

        // Apply rate limiting
        self.rate_limiter.wait_if_needed().await;

        let mut request = self
            .client
            .request(Method::POST, url.as_str())
            .header("Authorization", self.auth_header())
            .header("X-Atlassian-Token", "no-check"); // Required for file uploads

        // Create multipart form data
        let mut form = reqwest::multipart::Form::new();
        form = form.part(
            "file",
            reqwest::multipart::Part::bytes(content.to_vec()).file_name(filename.to_string()),
        );

        if let Some(mime) = mime_type {
            form = form.part("mimeType", reqwest::multipart::Part::text(mime.to_string()));
        }

        request = request.multipart(form);

        info!("Uploading attachment to issue: {}", issue_key);

        let response = request.send().await.map_err(JiraError::HttpClientError)?;
        let status = response.status();

        if status.is_success() {
            let response_text = response.text().await.map_err(JiraError::HttpClientError)?;
            debug!("Attachment upload response: {}", response_text);

            let attachments: Vec<JiraAttachment> =
                serde_json::from_str(&response_text).map_err(JiraError::SerializationError)?;

            Ok(attachments)
        } else {
            let error_text = response.text().await.map_err(JiraError::HttpClientError)?;
            error!("Attachment upload failed {}: {}", status, error_text);

            let error_json: serde_json::Value = serde_json::from_str(&error_text)
                .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

            Err(JiraError::from_jira_response(status, &error_json))
        }
    }

    /// Delete an attachment
    ///
    /// # Errors
    ///
    /// Returns an error if the attachment deletion fails.
    pub async fn delete_attachment(&self, attachment_id: &str) -> Result<()> {
        let endpoint = format!("attachment/{attachment_id}");
        let _: serde_json::Value = self.delete(&endpoint).await?;
        Ok(())
    }

    /// Download an attachment
    ///
    /// # Errors
    ///
    /// Returns an error if the attachment download fails.
    pub async fn download_attachment(&self, attachment_id: &str) -> Result<Vec<u8>> {
        let endpoint = format!("attachment/{attachment_id}");
        let url = self.build_url(&endpoint)?;

        // Apply rate limiting
        self.rate_limiter.wait_if_needed().await;

        let request = self
            .client
            .request(Method::GET, url.as_str())
            .header("Authorization", self.auth_header());

        info!("Downloading attachment: {}", attachment_id);

        let response = request.send().await.map_err(JiraError::HttpClientError)?;
        let status = response.status();

        if status.is_success() {
            let bytes = response.bytes().await.map_err(JiraError::HttpClientError)?;
            Ok(bytes.to_vec())
        } else {
            let error_text = response.text().await.map_err(JiraError::HttpClientError)?;
            error!("Attachment download failed {}: {}", status, error_text);

            let error_json: serde_json::Value = serde_json::from_str(&error_text)
                .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

            Err(JiraError::from_jira_response(status, &error_json))
        }
    }

    // Work Log Operations

    /// Get work logs for a specific issue
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_issue_work_logs(&self, issue_key: &str) -> Result<Vec<JiraWorkLog>> {
        let endpoint = format!("issue/{issue_key}/worklog");
        let response: serde_json::Value = self.get(&endpoint).await?;

        let work_logs = response
            .get("worklogs")
            .and_then(|w| w.as_array())
            .ok_or_else(|| JiraError::api_error("Invalid work logs response format"))?;

        let mut result = Vec::new();
        for work_log in work_logs {
            let work_log: JiraWorkLog =
                serde_json::from_value(work_log.clone()).map_err(JiraError::SerializationError)?;
            result.push(work_log);
        }

        Ok(result)
    }

    /// Add a work log entry to an issue
    ///
    /// # Errors
    ///
    /// Returns an error if the work log creation fails or the response cannot be parsed.
    pub async fn add_work_log(
        &self,
        issue_key: &str,
        work_log: &JiraWorkLogCreateRequest,
    ) -> Result<JiraWorkLog> {
        let endpoint = format!("issue/{issue_key}/worklog");
        self.post(&endpoint, work_log).await
    }

    /// Update an existing work log entry
    ///
    /// # Errors
    ///
    /// Returns an error if the work log update fails or the response cannot be parsed.
    pub async fn update_work_log(
        &self,
        issue_key: &str,
        work_log_id: &str,
        work_log: &JiraWorkLogUpdateRequest,
    ) -> Result<JiraWorkLog> {
        let endpoint = format!("issue/{issue_key}/worklog/{work_log_id}");
        self.put(&endpoint, work_log).await
    }

    /// Delete a work log entry
    ///
    /// # Errors
    ///
    /// Returns an error if the work log deletion fails.
    pub async fn delete_work_log(&self, issue_key: &str, work_log_id: &str) -> Result<()> {
        let endpoint = format!("issue/{issue_key}/worklog/{work_log_id}");
        let _: serde_json::Value = self.delete(&endpoint).await?;
        Ok(())
    }

    /// Get a specific work log entry
    ///
    /// # Errors
    ///
    /// Returns an error if the work log cannot be found or the request fails.
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
            return Err(JiraError::api_error(
                "Maximum 100 operations allowed per bulk request",
            ));
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

        summary.duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);

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
                Ok(()) => {
                    return BulkOperationResult {
                        issue_key: operation.issue_key.clone(),
                        success: true,
                        error_message: None,
                        operation_type: operation.operation_type.clone(),
                    };
                }
                Err(e) => {
                    if retry_count < max_retries && Self::should_retry_operation(self, &e) {
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
            BulkOperationType::AddComment => {
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
            BulkOperationType::Mixed => {
                // For mixed operations, we need to determine the operation type from the data
                if operation.data.get("fields").is_some() {
                    // This is an update operation
                    self.update_issue(&operation.issue_key, &operation.data)
                        .await
                } else if operation.data.get("transition_id").is_some() {
                    // This is a transition operation
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
                    // This is a comment operation
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

    /// Determine if an operation should be retried based on the error
    fn should_retry_operation(_self: &Self, error: &JiraError) -> bool {
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

    // Issue Watcher Operations

    /// Get watchers for a specific issue
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_issue_watchers(&self, issue_key: &str) -> Result<JiraWatchersResponse> {
        let endpoint = format!("issue/{issue_key}/watchers");
        self.get(&endpoint).await
    }

    /// Add a watcher to an issue
    ///
    /// # Errors
    ///
    /// Returns an error if the watcher addition fails.
    pub async fn add_issue_watcher(&self, issue_key: &str, account_id: &str) -> Result<()> {
        let endpoint = format!("issue/{issue_key}/watchers");
        let watcher_request = serde_json::json!({
            "accountId": account_id
        });
        let _: serde_json::Value = self.post(&endpoint, &watcher_request).await?;
        Ok(())
    }

    /// Remove a watcher from an issue
    ///
    /// # Errors
    ///
    /// Returns an error if the watcher removal fails.
    pub async fn remove_issue_watcher(&self, issue_key: &str, account_id: &str) -> Result<()> {
        let endpoint = format!("issue/{issue_key}/watchers?accountId={account_id}");
        let _: serde_json::Value = self.delete(&endpoint).await?;
        Ok(())
    }

    // Issue Label Operations

    /// Get all available labels
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get_labels(&self) -> Result<Vec<JiraLabel>> {
        let endpoint = "label";
        let response: serde_json::Value = self.get(endpoint).await?;

        let labels = response
            .as_array()
            .ok_or_else(|| JiraError::api_error("Invalid labels response format"))?;

        let mut result = Vec::new();
        for label in labels {
            let label: JiraLabel =
                serde_json::from_value(label.clone()).map_err(JiraError::SerializationError)?;
            result.push(label);
        }

        Ok(result)
    }

    /// Create a new label
    ///
    /// # Errors
    ///
    /// Returns an error if the label creation fails or the response cannot be parsed.
    pub async fn create_label(&self, label: &JiraLabelCreateRequest) -> Result<JiraLabel> {
        self.post("label", label).await
    }

    /// Update an existing label
    ///
    /// # Errors
    ///
    /// Returns an error if the label update fails or the response cannot be parsed.
    pub async fn update_label(
        &self,
        label_name: &str,
        label: &JiraLabelUpdateRequest,
    ) -> Result<JiraLabel> {
        let endpoint = format!("label/{label_name}");
        self.put(&endpoint, label).await
    }

    /// Delete a label
    ///
    /// # Errors
    ///
    /// Returns an error if the label deletion fails.
    pub async fn delete_label(&self, label_name: &str) -> Result<()> {
        let endpoint = format!("label/{label_name}");
        let _: serde_json::Value = self.delete(&endpoint).await?;
        Ok(())
    }

    // Issue Component Operations

    /// Create a new component
    ///
    /// # Errors
    ///
    /// Returns an error if the component creation fails or the response cannot be parsed.
    pub async fn create_component(
        &self,
        component: &JiraComponentCreateRequest,
    ) -> Result<JiraComponent> {
        self.post("component", component).await
    }

    /// Update an existing component
    ///
    /// # Errors
    ///
    /// Returns an error if the component update fails or the response cannot be parsed.
    pub async fn update_component(
        &self,
        component_id: &str,
        component: &JiraComponentUpdateRequest,
    ) -> Result<JiraComponent> {
        let endpoint = format!("component/{component_id}");
        self.put(&endpoint, component).await
    }

    /// Delete a component
    ///
    /// # Errors
    ///
    /// Returns an error if the component deletion fails.
    pub async fn delete_component(&self, component_id: &str) -> Result<()> {
        let endpoint = format!("component/{component_id}");
        let _: serde_json::Value = self.delete(&endpoint).await?;
        Ok(())
    }

    // Issue Cloning Operations

    /// Clone an issue with field mapping and optional copying of related data
    ///
    /// # Errors
    ///
    /// Returns an error if the cloning fails or the response cannot be parsed.
    pub async fn clone_issue(
        &self,
        original_issue_key: &str,
        clone_request: &JiraIssueCloneRequest,
    ) -> Result<JiraIssueCloneResponse> {
        // First, get the original issue
        let original_issue = self.get_issue(original_issue_key).await?;

        // Build the new issue data
        let mut new_issue_data = serde_json::json!({
            "fields": {
                "project": {
                    "key": clone_request.project_key
                },
                "issuetype": {
                    "name": clone_request.issue_type
                },
                "summary": clone_request.summary
            }
        });

        // Add description if provided
        if let Some(description) = &clone_request.description {
            new_issue_data["fields"]["description"] =
                serde_json::Value::String(description.clone());
        }

        // Apply field mapping
        if let Some(field_mapping) = &clone_request.field_mapping {
            Self::apply_field_mapping(&original_issue, &mut new_issue_data, field_mapping);
        }

        // Create the new issue
        let cloned_issue = self.create_issue(&new_issue_data).await?;

        // Return simplified response
        Ok(JiraIssueCloneResponse {
            original_issue_key: original_issue_key.to_string(),
            cloned_issue_key: cloned_issue.key.clone(),
            cloned_issue_id: cloned_issue.id.clone(),
            cloned_issue_url: cloned_issue.self_url.clone(),
            copied_attachments: None,
            copied_comments: None,
            copied_work_logs: None,
            copied_watchers: None,
            copied_links: None,
        })
    }

    /// Apply field mapping to clone issue data
    ///
    /// # Errors
    ///
    /// Returns an error if field mapping fails.
    fn apply_field_mapping(
        original_issue: &crate::types::jira::JiraIssue,
        new_issue_data: &mut serde_json::Value,
        field_mapping: &crate::types::jira::JiraFieldMapping,
    ) {
        use tracing::debug;

        debug!("Applying field mapping: {:?}", field_mapping);

        // Get the fields from the original issue
        let original_fields = &original_issue.fields;

        // Determine which fields to copy
        let fields_to_copy = if field_mapping.copy_fields.is_empty() {
            // If no specific fields are specified, use default behavior
            // Copy all fields except those in exclude_fields
            let mut fields: Vec<String> = original_fields.keys().cloned().collect();

            // Remove excluded fields
            fields.retain(|field| !field_mapping.exclude_fields.contains(field));

            // Always exclude system fields that shouldn't be copied
            let system_exclude_fields = vec![
                "assignee",
                "reporter",
                "created",
                "updated",
                "status",
                "resolution",
                "resolutiondate",
                "worklog",
                "attachment",
                "subtasks",
                "issuelinks",
                "watches",
                "votes",
            ];
            fields.retain(|field| !system_exclude_fields.contains(&field.as_str()));

            fields
        } else {
            // Use the specified copy_fields, but still respect exclude_fields
            field_mapping
                .copy_fields
                .iter()
                .filter(|field| !field_mapping.exclude_fields.contains(field))
                .cloned()
                .collect()
        };

        debug!("Fields to copy: {:?}", fields_to_copy);

        // Copy the specified fields
        for field_id in &fields_to_copy {
            if let Some(field_value) = original_fields.get(field_id) {
                // Apply custom field mapping if specified
                let target_field_id =
                    if let Some(ref custom_mapping) = field_mapping.custom_field_mapping {
                        custom_mapping.get(field_id).unwrap_or(field_id)
                    } else {
                        field_id
                    };

                // Skip if the field is already set (e.g., summary, description)
                if new_issue_data["fields"].get(target_field_id).is_some() {
                    debug!("Skipping field {} as it's already set", target_field_id);
                    continue;
                }

                // Handle special field types
                let processed_value = Self::process_field_value(field_id, field_value);
                new_issue_data["fields"][target_field_id] = processed_value;
                debug!("Copied field {} -> {}", field_id, target_field_id);
            } else {
                debug!("Field {} not found in original issue", field_id);
            }
        }
    }

    /// Process a field value for cloning, handling special field types
    ///
    /// # Errors
    ///
    /// Returns an error if field processing fails.
    fn process_field_value(field_id: &str, field_value: &serde_json::Value) -> serde_json::Value {
        use serde_json::Value;

        // Handle different field types
        match field_id {
            // Priority field - copy the priority object
            "priority" => {
                if let Some(priority) = field_value.as_object() {
                    Value::Object(priority.clone())
                } else {
                    field_value.clone()
                }
            }
            // Labels, components, and fix versions fields - copy the array
            "labels" | "components" | "fixVersions" => {
                if let Some(array) = field_value.as_array() {
                    Value::Array(array.clone())
                } else {
                    field_value.clone()
                }
            }
            // Environment and due date fields - copy as string
            "environment" | "duedate" => {
                if let Some(str_value) = field_value.as_str() {
                    Value::String(str_value.to_string())
                } else {
                    field_value.clone()
                }
            }
            // Custom fields - copy as-is
            _ => {
                if field_id.starts_with("customfield_") {
                    field_value.clone()
                } else {
                    // For other fields, copy as-is but log a warning for unknown fields
                    tracing::debug!("Copying unknown field type {}: {:?}", field_id, field_value);
                    field_value.clone()
                }
            }
        }
    }

    // Zephyr Test Management Operations

    /// Get the Zephyr API base URL
    #[must_use]
    pub fn zephyr_api_base_url(&self) -> String {
        // Zephyr typically uses the same base URL as Jira but with /rest/zapi/latest/ prefix
        format!("{}/rest/zapi/latest", self.config.api_base_url)
    }

    /// Make a GET request to the Zephyr API
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn zephyr_get<T>(&self, endpoint: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.zephyr_request(Method::GET, endpoint, None::<&()>)
            .await
    }

    /// Make a POST request to the Zephyr API
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn zephyr_post<T, U>(&self, endpoint: &str, body: &U) -> Result<T>
    where
        T: DeserializeOwned,
        U: Serialize + ?Sized,
    {
        self.zephyr_request(Method::POST, endpoint, Some(body))
            .await
    }

    /// Make a PUT request to the Zephyr API
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn zephyr_put<T, U>(&self, endpoint: &str, body: &U) -> Result<T>
    where
        T: DeserializeOwned,
        U: Serialize + ?Sized,
    {
        self.zephyr_request(Method::PUT, endpoint, Some(body)).await
    }

    /// Make a DELETE request to the Zephyr API
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn zephyr_delete<T>(&self, endpoint: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.zephyr_request(Method::DELETE, endpoint, None::<&()>)
            .await
    }

    /// Make a generic HTTP request to Zephyr API with retry logic
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails after all retries or the response cannot be parsed
    async fn zephyr_request<T, U>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<&U>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        U: Serialize + ?Sized,
    {
        let url = self.build_zephyr_url(endpoint)?;
        let mut retry_count = 0;
        let max_retries = 3;

        loop {
            // Apply rate limiting
            self.rate_limiter.wait_if_needed().await;

            let request_builder = self.build_zephyr_request(method.clone(), &url, body)?;

            info!("Making Zephyr {} request to {}", method, url);

            match request_builder.send().await {
                Ok(response) => {
                    let status = response.status();
                    debug!("Zephyr response status: {}", status);

                    if status.is_success() {
                        let response_text =
                            response.text().await.map_err(JiraError::HttpClientError)?;

                        debug!("Zephyr response body: {}", response_text);

                        return serde_json::from_str(&response_text).map_err(|e| {
                            error!("Failed to parse Zephyr JSON response: {}", e);
                            JiraError::SerializationError(e)
                        });
                    }

                    let error_text = response.text().await.map_err(JiraError::HttpClientError)?;

                    error!("Zephyr HTTP error {}: {}", status, error_text);

                    // Parse Zephyr error response
                    let error_json: serde_json::Value = serde_json::from_str(&error_text)
                        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

                    let jira_error = JiraError::from_jira_response(status, &error_json);

                    // Retry on certain status codes
                    if retry_count < max_retries && Self::should_retry(status) {
                        retry_count += 1;
                        let delay = Duration::from_millis(
                            1000 * u64::try_from(retry_count.max(0)).unwrap_or(0),
                        );
                        warn!(
                            "Retrying Zephyr request in {:?} (attempt {}/{})",
                            delay, retry_count, max_retries
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    return Err(jira_error);
                }
                Err(e) => {
                    error!("Zephyr request failed: {}", e);

                    if retry_count < max_retries && e.is_timeout() {
                        retry_count += 1;
                        let delay = Duration::from_millis(
                            1000 * u64::try_from(retry_count.max(0)).unwrap_or(0),
                        );
                        warn!(
                            "Retrying Zephyr request after timeout in {:?} (attempt {}/{})",
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

    /// Build a complete URL from the Zephyr endpoint
    fn build_zephyr_url(&self, endpoint: &str) -> Result<Url> {
        let base_url = Url::parse(&self.zephyr_api_base_url())
            .map_err(|e| JiraError::config_error(&format!("Invalid Zephyr API base URL: {e}")))?;

        base_url
            .join(endpoint)
            .map_err(|e| JiraError::config_error(&format!("Invalid Zephyr endpoint URL: {e}")))
    }

    /// Build a request with proper headers and authentication for Zephyr
    fn build_zephyr_request<U>(
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
            debug!("Zephyr request body: {}", json_body);
            request = request.body(json_body);
        }

        Ok(request)
    }

    // Test Step Operations

    /// Get test steps for a test case
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get_zephyr_test_steps(&self, test_case_id: &str) -> Result<Vec<ZephyrTestStep>> {
        let endpoint = format!("teststep/{test_case_id}");
        let response: serde_json::Value = self.zephyr_get(&endpoint).await?;

        // Extract test steps from the response
        let test_steps = response
            .get("testSteps")
            .and_then(|ts| ts.as_array())
            .ok_or_else(|| JiraError::api_error("Invalid test steps response format"))?;

        let mut result = Vec::new();
        for step in test_steps {
            let test_step: ZephyrTestStep =
                serde_json::from_value(step.clone()).map_err(JiraError::SerializationError)?;
            result.push(test_step);
        }

        Ok(result)
    }

    /// Create a new test step
    ///
    /// # Errors
    ///
    /// Returns an error if the test step creation fails or the response cannot be parsed
    pub async fn create_zephyr_test_step(
        &self,
        test_step: &ZephyrTestStepCreateRequest,
    ) -> Result<ZephyrTestStep> {
        let endpoint = format!("teststep/{}", test_step.test_case_id);
        self.zephyr_post(&endpoint, test_step).await
    }

    /// Update an existing test step
    ///
    /// # Errors
    ///
    /// Returns an error if the test step update fails or the response cannot be parsed
    pub async fn update_zephyr_test_step(
        &self,
        test_case_id: &str,
        step_id: &str,
        test_step: &ZephyrTestStepUpdateRequest,
    ) -> Result<ZephyrTestStep> {
        let endpoint = format!("teststep/{test_case_id}/{step_id}");
        self.zephyr_put(&endpoint, test_step).await
    }

    /// Delete a test step
    ///
    /// # Errors
    ///
    /// Returns an error if the test step deletion fails
    pub async fn delete_zephyr_test_step(&self, test_case_id: &str, step_id: &str) -> Result<()> {
        let endpoint = format!("teststep/{test_case_id}/{step_id}");
        let _: serde_json::Value = self.zephyr_delete(&endpoint).await?;
        Ok(())
    }

    // Test Case Operations

    /// Get a test case by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the test case cannot be found or the request fails
    /// Search for test cases
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails or the response cannot be parsed
    pub async fn search_zephyr_test_cases(
        &self,
        project_key: &str,
        start_at: Option<i32>,
        max_results: Option<i32>,
    ) -> Result<ZephyrTestCaseSearchResult> {
        let mut params = vec![("projectKey".to_string(), project_key.to_string())];

        if let Some(start) = start_at {
            params.push(("startAt".to_string(), start.to_string()));
        }

        if let Some(max) = max_results {
            params.push(("maxResults".to_string(), max.to_string()));
        }

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let endpoint = format!("testcase?{query_string}");
        self.zephyr_get(&endpoint).await
    }

    /// Create a new test case
    ///
    /// # Errors
    ///
    /// Returns an error if the test case creation fails or the response cannot be parsed
    pub async fn create_zephyr_test_case(
        &self,
        test_case: &ZephyrTestCaseCreateRequest,
    ) -> Result<ZephyrTestCase> {
        self.zephyr_post("testcase", test_case).await
    }

    /// Update an existing test case
    ///
    /// # Errors
    ///
    /// Returns an error if the test case update fails or the response cannot be parsed
    /// Delete a test case
    ///
    /// # Errors
    ///
    /// Returns an error if the test case deletion fails
    // Test Execution Operations
    /// Get test executions for a test case
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get_zephyr_test_executions(
        &self,
        test_case_id: &str,
    ) -> Result<Vec<ZephyrTestExecution>> {
        let endpoint = format!("execution/testcase/{test_case_id}");
        let response: serde_json::Value = self.zephyr_get(&endpoint).await?;

        // Extract test executions from the response
        let executions = response
            .get("executions")
            .and_then(|e| e.as_array())
            .ok_or_else(|| JiraError::api_error("Invalid test executions response format"))?;

        let mut result = Vec::new();
        for execution in executions {
            let test_execution: ZephyrTestExecution =
                serde_json::from_value(execution.clone()).map_err(JiraError::SerializationError)?;
            result.push(test_execution);
        }

        Ok(result)
    }

    /// Create a new test execution
    ///
    /// # Errors
    ///
    /// Returns an error if the test execution creation fails or the response cannot be parsed
    pub async fn create_zephyr_test_execution(
        &self,
        execution: &ZephyrTestExecutionCreateRequest,
    ) -> Result<ZephyrTestExecution> {
        self.zephyr_post("execution", execution).await
    }

    /// Update an existing test execution
    ///
    /// # Errors
    ///
    /// Returns an error if the test execution update fails or the response cannot be parsed
    /// Delete a test execution
    ///
    /// # Errors
    ///
    /// Returns an error if the test execution deletion fails
    // Test Cycle Operations
    /// Get test cycles for a project
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get_zephyr_test_cycles(&self, project_key: &str) -> Result<Vec<ZephyrTestCycle>> {
        let endpoint = format!("cycle?projectKey={project_key}");
        let response: serde_json::Value = self.zephyr_get(&endpoint).await?;

        // Extract test cycles from the response
        let cycles = response
            .get("cycles")
            .and_then(|c| c.as_array())
            .ok_or_else(|| JiraError::api_error("Invalid test cycles response format"))?;

        let mut result = Vec::new();
        for cycle in cycles {
            let test_cycle: ZephyrTestCycle =
                serde_json::from_value(cycle.clone()).map_err(JiraError::SerializationError)?;
            result.push(test_cycle);
        }

        Ok(result)
    }

    /// Create a new test cycle
    ///
    /// # Errors
    ///
    /// Returns an error if the test cycle creation fails or the response cannot be parsed
    /// Update an existing test cycle
    ///
    /// # Errors
    ///
    /// Returns an error if the test cycle update fails or the response cannot be parsed
    /// Delete a test cycle
    ///
    /// # Errors
    ///
    /// Returns an error if the test cycle deletion fails
    // Test Plan Operations
    /// Get test plans for a project
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get_zephyr_test_plans(&self, project_key: &str) -> Result<Vec<ZephyrTestPlan>> {
        let endpoint = format!("testplan?projectKey={project_key}");
        let response: serde_json::Value = self.zephyr_get(&endpoint).await?;

        // Extract test plans from the response
        let plans = response
            .get("testPlans")
            .and_then(|p| p.as_array())
            .ok_or_else(|| JiraError::api_error("Invalid test plans response format"))?;

        let mut result = Vec::new();
        for plan in plans {
            let test_plan: ZephyrTestPlan =
                serde_json::from_value(plan.clone()).map_err(JiraError::SerializationError)?;
            result.push(test_plan);
        }

        Ok(result)
    }
}
