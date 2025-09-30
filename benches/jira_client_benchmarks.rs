use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_jira_mcp::config::JiraConfig;
use rust_jira_mcp::jira::client::JiraClient;
use serde_json::json;
use std::time::Duration;

fn create_test_config() -> JiraConfig {
    JiraConfig {
        api_base_url: "https://test.atlassian.net".to_string(),
        email: "test@example.com".to_string(),
        personal_access_token: "test_token".to_string(),
        timeout_seconds: Some(30),
        max_results: Some(100),
        strict_ssl: Some(true),
        default_project: Some("TEST".to_string()),
        log_file: Some("test.log".into()),
    }
}

fn create_test_client() -> JiraClient {
    JiraClient::new(create_test_config()).expect("Failed to create test client")
}

fn benchmark_http_client_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("http_client_creation");
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("jira_client_new", |b| {
        b.iter(|| {
            black_box(JiraClient::new(create_test_config()).unwrap())
        })
    });
    
    group.finish();
}

fn benchmark_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");
    group.measurement_time(Duration::from_secs(10));
    
    // Test data representing typical Jira responses
    let issue_json = json!({
        "id": "10001",
        "key": "TEST-1",
        "self": "https://test.atlassian.net/rest/api/3/issue/10001",
        "fields": {
            "summary": "Test Issue",
            "status": {
                "name": "To Do",
                "id": "1"
            },
            "priority": {
                "name": "High",
                "id": "2"
            },
            "assignee": {
                "displayName": "Test User",
                "emailAddress": "test@example.com"
            },
            "created": "2024-01-01T00:00:00.000+0000",
            "updated": "2024-01-01T00:00:00.000+0000"
        }
    });
    
    let search_result_json = json!({
        "expand": "names,schema",
        "startAt": 0,
        "maxResults": 50,
        "total": 100,
        "issues": vec![issue_json.clone(); 50]
    });
    
    group.bench_function("parse_issue", |b| {
        b.iter(|| {
            let json_str = serde_json::to_string(&issue_json).unwrap();
            black_box(serde_json::from_str::<serde_json::Value>(&json_str).unwrap())
        })
    });
    
    group.bench_function("parse_search_result", |b| {
        b.iter(|| {
            let json_str = serde_json::to_string(&search_result_json).unwrap();
            black_box(serde_json::from_str::<serde_json::Value>(&json_str).unwrap())
        })
    });
    
    group.finish();
}

fn benchmark_rate_limiting(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_limiting");
    group.measurement_time(Duration::from_secs(5));
    
    let _client = create_test_client();
    
    group.bench_function("rate_limiter_wait", |b| {
        b.iter(|| {
            // This would test the rate limiter if we exposed it
            // For now, we'll simulate the wait time
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                tokio::time::sleep(Duration::from_millis(1)).await
            })
        })
    });
    
    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("string_allocations", |b| {
        b.iter(|| {
            let mut strings = Vec::new();
            for i in 0..1000 {
                strings.push(format!("test_string_{}", i));
            }
            black_box(strings)
        })
    });
    
    group.bench_function("json_value_allocations", |b| {
        b.iter(|| {
            let mut values = Vec::new();
            for i in 0..1000 {
                values.push(json!({
                    "id": i,
                    "name": format!("item_{}", i),
                    "data": vec![i; 10]
                }));
            }
            black_box(values)
        })
    });
    
    group.finish();
}

fn benchmark_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    group.measurement_time(Duration::from_secs(10));
    
    for concurrency in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_json_parsing", concurrency),
            concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let handles: Vec<_> = (0..concurrency)
                            .map(|i| {
                                tokio::spawn(async move {
                                    let data = json!({
                                        "id": i,
                                        "data": vec![i; 100]
                                    });
                                    serde_json::to_string(&data).unwrap()
                                })
                            })
                            .collect();
                        
                        let results: Vec<_> = futures::future::join_all(handles).await;
                        black_box(results)
                    })
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_http_client_creation,
    benchmark_json_parsing,
    benchmark_rate_limiting,
    benchmark_memory_usage,
    benchmark_concurrent_operations
);
criterion_main!(benches);
