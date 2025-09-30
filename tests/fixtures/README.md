# Test Fixtures

This directory contains test data collected from the live Jira API for use in unit and integration tests.

## Data Collection

Test data is collected using the `scripts/collect_test_data.py` script, which:

- Performs **READ-ONLY** operations on the live Jira API
- Anonymizes sensitive information (emails, usernames, etc.)
- Saves responses in a structured format for testing

## Files

- `jira_test_data_latest.json` - Most recent test data collection
- `jira_test_data_YYYYMMDD_HHMMSS.json` - Timestamped collections for historical reference

## Usage in Tests

```rust
use serde_json;
use std::fs;

// Load test data
let test_data = fs::read_to_string("tests/fixtures/jira_test_data_latest.json")?;
let fixtures: serde_json::Value = serde_json::from_str(&test_data)?;

// Use in tests
let auth_response = &fixtures["operations"]["authentication"];
let search_response = &fixtures["operations"]["search_issues"];
```

## Safety

- All data is anonymized before storage
- Only read-only operations are performed
- No production data is modified
- Sensitive information is replaced with test values

## Updating Test Data

To refresh test data with current API responses:

```bash
python3 scripts/collect_test_data.py
```

This will create a new timestamped file and update the `latest.json` file.
