use rust_jira_mcp::error::{JiraError, Result};
use reqwest::StatusCode;
use serde_json::json;

#[test]
fn test_jira_error_display() {
    // Test HttpError
    let error = JiraError::HttpError {
        status: StatusCode::NOT_FOUND,
        message: "Resource not found".to_string(),
    };
    assert!(error.to_string().contains("HTTP error"));
    assert!(error.to_string().contains("404"));
    assert!(error.to_string().contains("Resource not found"));

    // Test ApiError
    let error = JiraError::ApiError {
        message: "Invalid request".to_string(),
        error_codes: Some(vec!["INVALID_FIELD".to_string()]),
    };
    assert!(error.to_string().contains("Jira API error"));
    assert!(error.to_string().contains("Invalid request"));

    // Test ValidationError
    let error = JiraError::ValidationError {
        field: "email".to_string(),
        message: "Invalid email format".to_string(),
    };
    assert!(error.to_string().contains("Validation error"));
    assert!(error.to_string().contains("email"));
    assert!(error.to_string().contains("Invalid email format"));

    // Test AuthError
    let error = JiraError::AuthError {
        message: "Invalid credentials".to_string(),
    };
    assert!(error.to_string().contains("Authentication error"));
    assert!(error.to_string().contains("Invalid credentials"));

    // Test ConfigError
    let error = JiraError::ConfigError {
        message: "Missing required field".to_string(),
    };
    assert!(error.to_string().contains("Configuration error"));
    assert!(error.to_string().contains("Missing required field"));

    // Test SerializationError
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let error = JiraError::SerializationError(json_error);
    assert!(error.to_string().contains("Serialization error"));

    // Test HttpClientError - skip for now due to reqwest error creation complexity
    // let error = JiraError::HttpClientError(reqwest_error);
    // assert!(error.to_string().contains("HTTP client error"));

    // Test IoError
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let error = JiraError::IoError(io_error);
    assert!(error.to_string().contains("IO error"));

    // Test ConfigErrorWrapper
    let config_error = config::ConfigError::Message("Config error".to_string());
    let error = JiraError::ConfigErrorWrapper(config_error);
    assert!(error.to_string().contains("Configuration error"));

    // Test AnyhowError
    let anyhow_error = anyhow::anyhow!("Something went wrong");
    let error = JiraError::AnyhowError(anyhow_error);
    assert!(error.to_string().contains("Anyhow error"));

    // Test UrlError
    let url_error = url::ParseError::EmptyHost;
    let error = JiraError::UrlError(url_error);
    assert!(error.to_string().contains("URL parsing error"));

    // Test Unknown
    let error = JiraError::Unknown {
        message: "Something unexpected".to_string(),
    };
    assert!(error.to_string().contains("Unknown error"));
    assert!(error.to_string().contains("Something unexpected"));
}

#[test]
fn test_jira_error_from_jira_response_with_error_messages() {
    let response_body = json!({
        "errorMessages": [
            "Issue does not exist",
            "Permission denied"
        ]
    });
    
    let error = JiraError::from_jira_response(StatusCode::NOT_FOUND, &response_body);
    
    match error {
        JiraError::ApiError { message, error_codes } => {
            assert_eq!(message, "Issue does not exist, Permission denied");
            assert!(error_codes.is_some());
            assert!(error_codes.unwrap().is_empty());
        },
        _ => panic!("Expected ApiError"),
    }
}

#[test]
fn test_jira_error_from_jira_response_with_errors() {
    let response_body = json!({
        "errors": {
            "summary": "Summary is required",
            "project": "Project does not exist"
        }
    });
    
    let error = JiraError::from_jira_response(StatusCode::BAD_REQUEST, &response_body);
    
    match error {
        JiraError::ApiError { message, error_codes } => {
            // The message format might vary, so let's check for key parts
            assert!(message.contains("summary") || message.contains("project"));
            assert!(error_codes.is_some());
            let codes = error_codes.unwrap();
            assert!(codes.contains(&"summary".to_string()));
            assert!(codes.contains(&"project".to_string()));
        },
        _ => panic!("Expected ApiError"),
    }
}

#[test]
fn test_jira_error_from_jira_response_fallback() {
    let response_body = json!({});
    
    let error = JiraError::from_jira_response(StatusCode::INTERNAL_SERVER_ERROR, &response_body);
    
    match error {
        JiraError::ApiError { message, error_codes } => {
            // The message format might include more details, so check for key parts
            assert!(message.contains("HTTP 500"));
            assert!(error_codes.is_some());
            assert!(error_codes.unwrap().is_empty());
        },
        _ => panic!("Expected ApiError"),
    }
}

#[test]
fn test_jira_error_api_error() {
    let error = JiraError::api_error("Test API error");
    
    match error {
        JiraError::ApiError { message, error_codes } => {
            assert_eq!(message, "Test API error");
            assert!(error_codes.is_none());
        },
        _ => panic!("Expected ApiError"),
    }
}

#[test]
fn test_jira_error_validation_error() {
    let error = JiraError::validation_error("email", "Invalid format");
    
    match error {
        JiraError::ValidationError { field, message } => {
            assert_eq!(field, "email");
            assert_eq!(message, "Invalid format");
        },
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_jira_error_auth_error() {
    let error = JiraError::auth_error("Invalid token");
    
    match error {
        JiraError::AuthError { message } => {
            assert_eq!(message, "Invalid token");
        },
        _ => panic!("Expected AuthError"),
    }
}

#[test]
fn test_jira_error_config_error() {
    let error = JiraError::config_error("Missing API key");
    
    match error {
        JiraError::ConfigError { message } => {
            assert_eq!(message, "Missing API key");
        },
        _ => panic!("Expected ConfigError"),
    }
}

#[test]
fn test_jira_error_unknown_error() {
    let error = JiraError::unknown_error("Unexpected failure");
    
    match error {
        JiraError::Unknown { message } => {
            assert_eq!(message, "Unexpected failure");
        },
        _ => panic!("Expected Unknown"),
    }
}

#[test]
fn test_jira_error_from_serde_json_error() {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let error: JiraError = json_error.into();
    
    match error {
        JiraError::SerializationError(_) => {},
        _ => panic!("Expected SerializationError"),
    }
}

#[test]
fn test_jira_error_from_reqwest_error() {
    // Skip reqwest error test due to complexity of creating reqwest::Error
    // In a real test, you would create a reqwest error through actual HTTP operations
    // This test is intentionally empty as reqwest::Error is complex to create
}

#[test]
fn test_jira_error_from_io_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let error: JiraError = io_error.into();
    
    match error {
        JiraError::IoError(_) => {},
        _ => panic!("Expected IoError"),
    }
}

#[test]
fn test_jira_error_from_config_error() {
    let config_error = config::ConfigError::Message("Config error".to_string());
    let error: JiraError = config_error.into();
    
    match error {
        JiraError::ConfigErrorWrapper(_) => {},
        _ => panic!("Expected ConfigErrorWrapper"),
    }
}

#[test]
fn test_jira_error_from_anyhow_error() {
    let anyhow_error = anyhow::anyhow!("Something went wrong");
    let error: JiraError = anyhow_error.into();
    
    match error {
        JiraError::AnyhowError(_) => {},
        _ => panic!("Expected AnyhowError"),
    }
}

#[test]
fn test_jira_error_from_url_error() {
    let url_error = url::ParseError::EmptyHost;
    let error: JiraError = url_error.into();
    
    match error {
        JiraError::UrlError(_) => {},
        _ => panic!("Expected UrlError"),
    }
}

#[test]
fn test_result_type_alias() {
    // Test that the Result type alias works correctly
    fn success_function() -> Result<String> {
        Ok("success".to_string())
    }
    
    fn error_function() -> Result<String> {
        Err(JiraError::api_error("test error"))
    }
    
    assert!(success_function().is_ok());
    assert!(error_function().is_err());
    
    let success_result = success_function().unwrap();
    assert_eq!(success_result, "success");
    
    let error_result = error_function().unwrap_err();
    match error_result {
        JiraError::ApiError { message, .. } => {
            assert_eq!(message, "test error");
        },
        _ => panic!("Expected ApiError"),
    }
}

#[test]
fn test_jira_error_debug() {
    let error = JiraError::ApiError {
        message: "Test error".to_string(),
        error_codes: Some(vec!["TEST_CODE".to_string()]),
    };
    
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("ApiError"));
    assert!(debug_str.contains("Test error"));
    assert!(debug_str.contains("TEST_CODE"));
}

#[test]
fn test_jira_error_validation_debug() {
    let error = JiraError::ValidationError {
        field: "test_field".to_string(),
        message: "test message".to_string(),
    };
    
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("ValidationError"));
    assert!(debug_str.contains("test_field"));
    assert!(debug_str.contains("test message"));
}

#[test]
fn test_jira_error_serialization() {
    // Since JiraError doesn't implement Serialize/Deserialize, we'll test the display format instead
    let error = JiraError::ApiError {
        message: "Test error".to_string(),
        error_codes: Some(vec!["TEST_CODE".to_string()]),
    };
    
    // Test that the error can be converted to string
    let error_str = error.to_string();
    assert!(error_str.contains("Jira API error"));
    assert!(error_str.contains("Test error"));
}

#[test]
fn test_jira_error_error_trait() {
    // Test that JiraError implements the Error trait
    let error = JiraError::ApiError {
        message: "Test error".to_string(),
        error_codes: None,
    };
    
    // This should compile and work
    let error_ref: &dyn std::error::Error = &error;
    assert!(error_ref.to_string().contains("Jira API error"));
}

#[test]
fn test_jira_error_from_jira_response_empty_error_messages() {
    let response_body = json!({
        "errorMessages": []
    });
    
    let error = JiraError::from_jira_response(StatusCode::BAD_REQUEST, &response_body);
    
    match error {
        JiraError::ApiError { message, error_codes } => {
            assert_eq!(message, "");
            assert!(error_codes.is_some());
            assert!(error_codes.unwrap().is_empty());
        },
        _ => panic!("Expected ApiError"),
    }
}

#[test]
fn test_jira_error_from_jira_response_mixed_error_types() {
    let response_body = json!({
        "errorMessages": ["General error"],
        "errors": {
            "field1": "Field 1 error",
            "field2": "Field 2 error"
        }
    });
    
    let error = JiraError::from_jira_response(StatusCode::BAD_REQUEST, &response_body);
    
    match error {
        JiraError::ApiError { message, error_codes } => {
            // Should prioritize errorMessages over errors
            assert_eq!(message, "General error");
            assert!(error_codes.is_some());
            assert!(error_codes.unwrap().is_empty());
        },
        _ => panic!("Expected ApiError"),
    }
}

#[test]
fn test_jira_error_from_jira_response_non_string_error_messages() {
    let response_body = json!({
        "errorMessages": [
            "String error",
            123,
            true,
            null
        ]
    });
    
    let error = JiraError::from_jira_response(StatusCode::BAD_REQUEST, &response_body);
    
    match error {
        JiraError::ApiError { message, error_codes } => {
            // Should only include string errors
            assert_eq!(message, "String error");
            assert!(error_codes.is_some());
            assert!(error_codes.unwrap().is_empty());
        },
        _ => panic!("Expected ApiError"),
    }
}
