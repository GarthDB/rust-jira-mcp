use crate::config::JiraConfig;
use crate::error::Result;
use reqwest::Client;

pub struct JiraClient {
    client: Client,
    config: JiraConfig,
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
            .build()?;

        Ok(Self { client, config })
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
}
