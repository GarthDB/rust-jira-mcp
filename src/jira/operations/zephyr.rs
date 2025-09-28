use crate::config::JiraConfig;
use crate::error::{JiraError, Result};
use crate::types::jira::{
    ZephyrAttachment, ZephyrStepResult, ZephyrStepResultCreateRequest, ZephyrStepResultUpdateRequest,
    ZephyrTestCase, ZephyrTestCaseCreateRequest, ZephyrTestCaseSearchResult, ZephyrTestCaseUpdateRequest,
    ZephyrTestCycle, ZephyrTestCycleCreateRequest, ZephyrTestExecution, ZephyrTestExecutionCreateRequest,
    ZephyrTestExecutionSearchResult, ZephyrTestExecutionUpdateRequest, ZephyrTestPlan,
    ZephyrTestPlanCreateRequest, ZephyrTestStep, ZephyrTestStepCreateRequest, ZephyrTestStepSearchResult,
    ZephyrTestStepUpdateRequest,
};
use reqwest::{Client, Method, RequestBuilder};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, warn};
use url::Url;

/// Zephyr REST API Client for test management operations
pub struct ZephyrClient {
    client: Client,
    config: JiraConfig,
    rate_limiter: RateLimiter,
}

/// Simple rate limiter for Zephyr API requests
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
            debug!("Zephyr rate limiting: sleeping for {:?}", sleep_duration);
            tokio::time::sleep(sleep_duration).await;
        }

        {
            let mut last_request = self.last_request.lock().unwrap();
            *last_request = std::time::Instant::now();
        }
    }
}

impl ZephyrClient {
    /// Create a new Zephyr client with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created
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

    /// Get the Zephyr API base URL
    #[must_use]
    pub fn zephyr_api_base_url(&self) -> String {
        // Zephyr typically uses the same base URL as Jira but with /rest/zapi/latest/ prefix
        format!("{}/rest/zapi/latest", self.config.api_base_url)
    }

    /// Get the authentication header from the configuration
    #[must_use]
    pub fn auth_header(&self) -> String {
        self.config.auth_header()
    }

    /// Make a GET request to the Zephyr API
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get<T>(&self, endpoint: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request(Method::GET, endpoint, None::<&()>).await
    }

    /// Make a POST request to the Zephyr API
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn post<T, U>(&self, endpoint: &str, body: &U) -> Result<T>
    where
        T: DeserializeOwned,
        U: Serialize + ?Sized,
    {
        self.request(Method::POST, endpoint, Some(body)).await
    }

    /// Make a PUT request to the Zephyr API
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn put<T, U>(&self, endpoint: &str, body: &U) -> Result<T>
    where
        T: DeserializeOwned,
        U: Serialize + ?Sized,
    {
        self.request(Method::PUT, endpoint, Some(body)).await
    }

    /// Make a DELETE request to the Zephyr API
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
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
    /// Returns an error if the request fails after all retries or the response cannot be parsed
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
                        let delay = Duration::from_millis(1000 * retry_count as u64);
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
                        let delay = Duration::from_millis(1000 * retry_count as u64);
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

    /// Build a complete URL from the endpoint
    fn build_url(&self, endpoint: &str) -> Result<Url> {
        let base_url = Url::parse(&self.zephyr_api_base_url()).map_err(|e| JiraError::ConfigError {
            message: format!("Invalid Zephyr API base URL: {e}"),
        })?;

        base_url.join(endpoint).map_err(|e| JiraError::ConfigError {
            message: format!("Invalid Zephyr endpoint URL: {e}"),
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
            debug!("Zephyr request body: {}", json_body);
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

    // Test Step Operations

    /// Get test steps for a test case
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get_test_steps(&self, test_case_id: &str) -> Result<Vec<ZephyrTestStep>> {
        let endpoint = format!("teststep/{}", test_case_id);
        let response: serde_json::Value = self.get(&endpoint).await?;

        // Extract test steps from the response
        let test_steps = response
            .get("testSteps")
            .and_then(|ts| ts.as_array())
            .ok_or_else(|| JiraError::ApiError {
                message: "Invalid test steps response format".to_string(),
            })?;

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
    pub async fn create_test_step(&self, test_step: &ZephyrTestStepCreateRequest) -> Result<ZephyrTestStep> {
        let endpoint = format!("teststep/{}", test_step.test_case_id);
        self.post(&endpoint, test_step).await
    }

    /// Update an existing test step
    ///
    /// # Errors
    ///
    /// Returns an error if the test step update fails or the response cannot be parsed
    pub async fn update_test_step(
        &self,
        test_case_id: &str,
        step_id: &str,
        test_step: &ZephyrTestStepUpdateRequest,
    ) -> Result<ZephyrTestStep> {
        let endpoint = format!("teststep/{}/{}", test_case_id, step_id);
        self.put(&endpoint, test_step).await
    }

    /// Delete a test step
    ///
    /// # Errors
    ///
    /// Returns an error if the test step deletion fails
    pub async fn delete_test_step(&self, test_case_id: &str, step_id: &str) -> Result<()> {
        let endpoint = format!("teststep/{}/{}", test_case_id, step_id);
        let _: serde_json::Value = self.delete(&endpoint).await?;
        Ok(())
    }

    // Test Case Operations

    /// Get a test case by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the test case cannot be found or the request fails
    pub async fn get_test_case(&self, test_case_id: &str) -> Result<ZephyrTestCase> {
        let endpoint = format!("testcase/{}", test_case_id);
        self.get(&endpoint).await
    }

    /// Search for test cases
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails or the response cannot be parsed
    pub async fn search_test_cases(
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

        let endpoint = format!("testcase?{}", query_string);
        self.get(&endpoint).await
    }

    /// Create a new test case
    ///
    /// # Errors
    ///
    /// Returns an error if the test case creation fails or the response cannot be parsed
    pub async fn create_test_case(&self, test_case: &ZephyrTestCaseCreateRequest) -> Result<ZephyrTestCase> {
        self.post("testcase", test_case).await
    }

    /// Update an existing test case
    ///
    /// # Errors
    ///
    /// Returns an error if the test case update fails or the response cannot be parsed
    pub async fn update_test_case(
        &self,
        test_case_id: &str,
        test_case: &ZephyrTestCaseUpdateRequest,
    ) -> Result<ZephyrTestCase> {
        let endpoint = format!("testcase/{}", test_case_id);
        self.put(&endpoint, test_case).await
    }

    /// Delete a test case
    ///
    /// # Errors
    ///
    /// Returns an error if the test case deletion fails
    pub async fn delete_test_case(&self, test_case_id: &str) -> Result<()> {
        let endpoint = format!("testcase/{}", test_case_id);
        let _: serde_json::Value = self.delete(&endpoint).await?;
        Ok(())
    }

    // Test Execution Operations

    /// Get test executions for a test case
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get_test_executions(&self, test_case_id: &str) -> Result<Vec<ZephyrTestExecution>> {
        let endpoint = format!("execution/testcase/{}", test_case_id);
        let response: serde_json::Value = self.get(&endpoint).await?;

        // Extract test executions from the response
        let executions = response
            .get("executions")
            .and_then(|e| e.as_array())
            .ok_or_else(|| JiraError::ApiError {
                message: "Invalid test executions response format".to_string(),
            })?;

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
    pub async fn create_test_execution(&self, execution: &ZephyrTestExecutionCreateRequest) -> Result<ZephyrTestExecution> {
        self.post("execution", execution).await
    }

    /// Update an existing test execution
    ///
    /// # Errors
    ///
    /// Returns an error if the test execution update fails or the response cannot be parsed
    pub async fn update_test_execution(
        &self,
        execution_id: &str,
        execution: &ZephyrTestExecutionUpdateRequest,
    ) -> Result<ZephyrTestExecution> {
        let endpoint = format!("execution/{}", execution_id);
        self.put(&endpoint, execution).await
    }

    /// Delete a test execution
    ///
    /// # Errors
    ///
    /// Returns an error if the test execution deletion fails
    pub async fn delete_test_execution(&self, execution_id: &str) -> Result<()> {
        let endpoint = format!("execution/{}", execution_id);
        let _: serde_json::Value = self.delete(&endpoint).await?;
        Ok(())
    }

    // Test Cycle Operations

    /// Get test cycles for a project
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get_test_cycles(&self, project_key: &str) -> Result<Vec<ZephyrTestCycle>> {
        let endpoint = format!("cycle?projectKey={}", project_key);
        let response: serde_json::Value = self.get(&endpoint).await?;

        // Extract test cycles from the response
        let cycles = response
            .get("cycles")
            .and_then(|c| c.as_array())
            .ok_or_else(|| JiraError::ApiError {
                message: "Invalid test cycles response format".to_string(),
            })?;

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
    pub async fn create_test_cycle(&self, cycle: &ZephyrTestCycleCreateRequest) -> Result<ZephyrTestCycle> {
        self.post("cycle", cycle).await
    }

    /// Update an existing test cycle
    ///
    /// # Errors
    ///
    /// Returns an error if the test cycle update fails or the response cannot be parsed
    pub async fn update_test_cycle(
        &self,
        cycle_id: &str,
        cycle: &ZephyrTestCycleCreateRequest,
    ) -> Result<ZephyrTestCycle> {
        let endpoint = format!("cycle/{}", cycle_id);
        self.put(&endpoint, cycle).await
    }

    /// Delete a test cycle
    ///
    /// # Errors
    ///
    /// Returns an error if the test cycle deletion fails
    pub async fn delete_test_cycle(&self, cycle_id: &str) -> Result<()> {
        let endpoint = format!("cycle/{}", cycle_id);
        let _: serde_json::Value = self.delete(&endpoint).await?;
        Ok(())
    }

    // Test Plan Operations

    /// Get test plans for a project
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get_test_plans(&self, project_key: &str) -> Result<Vec<ZephyrTestPlan>> {
        let endpoint = format!("testplan?projectKey={}", project_key);
        let response: serde_json::Value = self.get(&endpoint).await?;

        // Extract test plans from the response
        let plans = response
            .get("testPlans")
            .and_then(|p| p.as_array())
            .ok_or_else(|| JiraError::ApiError {
                message: "Invalid test plans response format".to_string(),
            })?;

        let mut result = Vec::new();
        for plan in plans {
            let test_plan: ZephyrTestPlan =
                serde_json::from_value(plan.clone()).map_err(JiraError::SerializationError)?;
            result.push(test_plan);
        }

        Ok(result)
    }

    /// Create a new test plan
    ///
    /// # Errors
    ///
    /// Returns an error if the test plan creation fails or the response cannot be parsed
    pub async fn create_test_plan(&self, plan: &ZephyrTestPlanCreateRequest) -> Result<ZephyrTestPlan> {
        self.post("testplan", plan).await
    }

    /// Update an existing test plan
    ///
    /// # Errors
    ///
    /// Returns an error if the test plan update fails or the response cannot be parsed
    pub async fn update_test_plan(
        &self,
        plan_id: &str,
        plan: &ZephyrTestPlanCreateRequest,
    ) -> Result<ZephyrTestPlan> {
        let endpoint = format!("testplan/{}", plan_id);
        self.put(&endpoint, plan).await
    }

    /// Delete a test plan
    ///
    /// # Errors
    ///
    /// Returns an error if the test plan deletion fails
    pub async fn delete_test_plan(&self, plan_id: &str) -> Result<()> {
        let endpoint = format!("testplan/{}", plan_id);
        let _: serde_json::Value = self.delete(&endpoint).await?;
        Ok(())
    }
}
