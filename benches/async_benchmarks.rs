use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_jira_mcp::performance::{
    AsyncConnectionPool, AsyncRateLimiter, AsyncTaskManager, PerformanceMetrics,
};
use std::sync::Arc;
use std::time::Duration;

fn create_test_metrics() -> Arc<PerformanceMetrics> {
    Arc::new(PerformanceMetrics::new())
}

fn benchmark_async_task_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_task_manager");
    group.measurement_time(Duration::from_secs(10));

    let metrics = create_test_metrics();

    for max_concurrent in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("task_execution", max_concurrent),
            max_concurrent,
            |b, &max_concurrent| {
                b.iter(|| {
                    let task_manager = AsyncTaskManager::new(max_concurrent, metrics.clone());
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let tasks: Vec<_> = (0..100)
                            .map(|i| {
                                let task_id = format!("task_{}", i);
                                let task_name = format!("Test Task {}", i);
                                let task = move || async move {
                                    // Simulate some work
                                    tokio::time::sleep(Duration::from_millis(1)).await;
                                    Ok::<i32, Box<dyn std::error::Error + Send + Sync>>(i)
                                };
                                (task_id, task_name, task)
                            })
                            .collect();

                        let results = task_manager
                            .execute_tasks_concurrent(
                                tasks
                                    .into_iter()
                                    .map(|(id, name, task)| (id, name, task()))
                                    .collect(),
                            )
                            .await;
                        black_box(results);
                    })
                })
            },
        );
    }

    group.finish();
}

fn benchmark_async_rate_limiter(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_rate_limiter");
    group.measurement_time(Duration::from_secs(5));

    for requests_per_second in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("rate_limiting", requests_per_second),
            requests_per_second,
            |b, &requests_per_second| {
                b.iter(|| {
                    let rate_limiter =
                        AsyncRateLimiter::new(requests_per_second, Duration::from_secs(1));
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        for _ in 0..requests_per_second {
                            rate_limiter.wait().await;
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

fn benchmark_async_connection_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_connection_pool");
    group.measurement_time(Duration::from_secs(10));

    for max_connections in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("connection_acquisition", max_connections),
            max_connections,
            |b, &max_connections| {
                b.iter(|| {
                    let pool = AsyncConnectionPool::new(max_connections);
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let handles: Vec<_> = (0..max_connections * 2)
                            .map(|_| {
                                let pool = pool.clone();
                                tokio::spawn(async move {
                                    let _guard = pool.acquire().await;
                                    // Simulate some work
                                    tokio::time::sleep(Duration::from_millis(1)).await;
                                })
                            })
                            .collect();

                        futures::future::join_all(handles).await;
                    })
                })
            },
        );
    }

    group.finish();
}

fn benchmark_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    group.measurement_time(Duration::from_secs(10));

    for concurrency in [1, 2, 4, 8, 16, 32].iter() {
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
                                    let mut results = Vec::new();
                                    for j in 0..100 {
                                        let data = serde_json::json!({
                                            "id": i * 100 + j,
                                            "name": format!("item_{}_{}", i, j),
                                            "data": vec![i * 100 + j; 10],
                                            "metadata": {
                                                "created": "2024-01-01T00:00:00.000+0000",
                                                "updated": "2024-01-01T00:00:00.000+0000"
                                            }
                                        });
                                        let json_str = serde_json::to_string(&data).unwrap();
                                        let parsed: serde_json::Value =
                                            serde_json::from_str(&json_str).unwrap();
                                        results.push(parsed);
                                    }
                                    results
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

fn benchmark_async_batching(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_batching");
    group.measurement_time(Duration::from_secs(10));

    for batch_size in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_processing", batch_size),
            batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut handles = Vec::new();

                        for batch in (0..100).collect::<Vec<_>>().chunks(batch_size) {
                            let batch = batch.to_vec();
                            let handle = tokio::spawn(async move {
                                // Simulate batch processing
                                tokio::time::sleep(Duration::from_millis(1)).await;
                                batch.len()
                            });
                            handles.push(handle);
                        }

                        let results: Vec<_> = futures::future::join_all(handles).await;
                        black_box(results);
                    })
                })
            },
        );
    }

    group.finish();
}

fn benchmark_memory_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation_patterns");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("string_concatenation", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut handles = Vec::new();

                for i in 0..100 {
                    let handle = tokio::spawn(async move {
                        let mut result = String::new();
                        for j in 0..1000 {
                            result.push_str(&format!("item_{}_{}", i, j));
                        }
                        result
                    });
                    handles.push(handle);
                }

                let results: Vec<_> = futures::future::join_all(handles).await;
                black_box(results);
            })
        })
    });

    group.bench_function("vec_operations", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut handles = Vec::new();

                for i in 0..100 {
                    let handle = tokio::spawn(async move {
                        let mut vec = Vec::with_capacity(1000);
                        for j in 0..1000 {
                            vec.push(i * 1000 + j);
                        }
                        vec
                    });
                    handles.push(handle);
                }

                let results: Vec<_> = futures::future::join_all(handles).await;
                black_box(results);
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_async_task_manager,
    benchmark_async_rate_limiter,
    benchmark_async_connection_pool,
    benchmark_concurrent_operations,
    benchmark_async_batching,
    benchmark_memory_allocation_patterns
);
criterion_main!(benches);
