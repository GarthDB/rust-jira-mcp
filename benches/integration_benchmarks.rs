use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_jira_mcp::config::JiraConfig;
use rust_jira_mcp::performance::{CacheManager, OptimizedJiraClient};
use rust_jira_mcp::types::jira::{BulkOperationItem, BulkOperationType};
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

fn benchmark_client_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("client_creation");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("optimized_client_new", |b| {
        b.iter(|| {
            let config = create_test_config();
            black_box(OptimizedJiraClient::new(config).unwrap())
        })
    });

    group.finish();
}

fn benchmark_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");
    group.measurement_time(Duration::from_secs(10));

    let test_data = create_large_test_data();

    group.bench_function("serialize_large_object", |b| {
        b.iter(|| black_box(serde_json::to_string(&test_data).unwrap()))
    });

    group.bench_function("deserialize_large_object", |b| {
        let json_str = serde_json::to_string(&test_data).unwrap();
        b.iter(|| black_box(serde_json::from_str::<serde_json::Value>(&json_str).unwrap()))
    });

    group.finish();
}

fn benchmark_search_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_operations");
    group.measurement_time(Duration::from_secs(10));

    let config = create_test_config();
    let _client = OptimizedJiraClient::new(config).unwrap();

    let jql_queries = [
        "project = TEST",
        "project = TEST AND status = Open",
        "project = TEST AND assignee = currentUser()",
        "project = TEST AND created >= -7d",
        "project = TEST AND priority in (High, Critical)",
    ];

    for (i, jql) in jql_queries.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("search_jql", i), jql, |b, &_jql| {
            b.iter(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    // This would normally make a real API call
                    // For benchmarking, we'll simulate the operation
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Ok::<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>(json!({
                        "total": 100,
                        "issues": vec![create_test_issue(); 10]
                    }))
                })
            })
        });
    }

    group.finish();
}

fn benchmark_bulk_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_operations");
    group.measurement_time(Duration::from_secs(15));

    let config = create_test_config();
    let _client = OptimizedJiraClient::new(config).unwrap();

    for batch_size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("bulk_update", batch_size),
            batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let operations = create_bulk_operations(batch_size);

                        // Simulate bulk operation processing
                        let mut results = Vec::new();
                        for chunk in operations.chunks(10) {
                            for operation in chunk {
                                // Simulate individual operation
                                tokio::time::sleep(Duration::from_millis(1)).await;
                                results.push(serde_json::json!({
                                    "issue_key": operation.issue_key,
                                    "success": true
                                }));
                            }
                        }

                        black_box(results);
                    })
                })
            },
        );
    }

    group.finish();
}

fn benchmark_caching_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("caching_performance");
    group.measurement_time(Duration::from_secs(10));

    let cache_manager = CacheManager::new();

    group.bench_function("cache_insertion", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                for i in 0..1000 {
                    let key = format!("test_key_{}", i);
                    let value = json!({
                        "id": i,
                        "data": vec![i; 10]
                    });
                    cache_manager.api_responses.insert(key, value).await;
                }
            })
        })
    });

    group.bench_function("cache_retrieval", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Pre-populate cache
            for i in 0..100 {
                let key = format!("test_key_{}", i);
                let value = json!({
                    "id": i,
                    "data": vec![i; 10]
                });
                cache_manager.api_responses.insert(key, value).await;
            }
        });

        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                for i in 0..100 {
                    let key = format!("test_key_{}", i);
                    black_box(cache_manager.api_responses.get(&key).await);
                }
            })
        })
    });

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("large_object_creation", |b| {
        b.iter(|| {
            let mut objects = Vec::new();
            for i in 0..10000 {
                let obj = json!({
                    "id": i,
                    "name": format!("object_{}", i),
                    "data": vec![i; 100],
                    "metadata": {
                        "created": "2024-01-01T00:00:00.000+0000",
                        "updated": "2024-01-01T00:00:00.000+0000",
                        "tags": vec![format!("tag_{}", i % 10); 5]
                    }
                });
                objects.push(obj);
            }
            black_box(objects);
        })
    });

    group.bench_function("string_operations", |b| {
        b.iter(|| {
            let mut strings = Vec::new();
            for i in 0..10000 {
                let s = format!("string_{}_with_lots_of_data_{}", i, i * 2);
                strings.push(s);
            }
            black_box(strings);
        })
    });

    group.finish();
}

fn benchmark_concurrent_requests(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_requests");
    group.measurement_time(Duration::from_secs(15));

    for concurrency in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_api_calls", concurrency),
            concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let handles: Vec<_> = (0..concurrency)
                            .map(|i| {
                                tokio::spawn(async move {
                                    // Simulate API call
                                    tokio::time::sleep(Duration::from_millis(10)).await;
                                    json!({
                                        "id": i,
                                        "result": "success"
                                    })
                                })
                            })
                            .collect();

                        let results: Vec<_> = futures::future::join_all(handles).await;
                        black_box(results);
                    })
                })
            },
        );
    }

    group.finish();
}

fn benchmark_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("error_creation", |b| {
        b.iter(|| {
            let error = rust_jira_mcp::error::JiraError::api_error("Test error message");
            black_box(error);
        })
    });

    group.bench_function("error_serialization", |b| {
        b.iter(|| {
            let error = rust_jira_mcp::error::JiraError::api_error("Test error message");
            let json = error.to_string();
            black_box(json);
        })
    });

    group.finish();
}

// Helper functions

fn create_large_test_data() -> serde_json::Value {
    json!({
        "issues": (0..100).map(|_i| create_test_issue()).collect::<Vec<_>>(),
        "total": 100,
        "startAt": 0,
        "maxResults": 100
    })
}

fn create_test_issue() -> serde_json::Value {
    json!({
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
            "updated": "2024-01-01T00:00:00.000+0000",
            "description": "This is a test issue for benchmarking purposes",
            "labels": ["test", "benchmark", "performance"],
            "components": [
                {
                    "name": "Test Component",
                    "id": "10001"
                }
            ]
        }
    })
}

fn create_bulk_operations(count: usize) -> Vec<BulkOperationItem> {
    (0..count)
        .map(|i| BulkOperationItem {
            issue_key: format!("TEST-{}", i + 1),
            operation_type: BulkOperationType::Update,
            data: json!({
                "fields": {
                    "summary": format!("Updated issue {}", i + 1),
                    "description": format!("Updated description for issue {}", i + 1)
                }
            }),
        })
        .collect()
}

criterion_group!(
    benches,
    benchmark_client_creation,
    benchmark_json_serialization,
    benchmark_search_operations,
    benchmark_bulk_operations,
    benchmark_caching_performance,
    benchmark_memory_usage,
    benchmark_concurrent_requests,
    benchmark_error_handling
);
criterion_main!(benches);
