use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_jira_mcp::performance::{CacheKeyGenerator, CacheManager, CacheStore, MokaCache};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

fn create_test_data() -> Vec<(String, serde_json::Value)> {
    (0..1000)
        .map(|i| {
            let key = format!("test_key_{i}");
            let value = json!({
                "id": i,
                "name": format!("item_{i}"),
                "data": vec![i; 10],
                "metadata": {
                    "created": "2024-01-01T00:00:00.000+0000",
                    "updated": "2024-01-01T00:00:00.000+0000"
                }
            });
            (key, value)
        })
        .collect()
}

#[allow(clippy::cast_sign_loss)]
fn benchmark_cache_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_insertion");
    group.measurement_time(Duration::from_secs(10));

    let test_data = create_test_data();

    for cache_size in &[100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::new("moka_cache", cache_size),
            cache_size,
            |b, &cache_size| {
        b.iter(|| {
            let cache = MokaCache::new(cache_size as u64, Duration::from_secs(300));
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                for (key, value) in &test_data[..cache_size] {
                    cache.insert(key.clone(), value.clone()).await;
                }
            });
        });
            },
        );
    }

    group.finish();
}

#[allow(clippy::cast_sign_loss)]
fn benchmark_cache_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_retrieval");
    group.measurement_time(Duration::from_secs(10));

    let test_data = create_test_data();
    let cache_size = 1000;

    group.bench_function("moka_cache_get", |b| {
        b.iter(|| {
            let cache = MokaCache::new(cache_size as u64, Duration::from_secs(300));
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Insert data first
                for (key, value) in &test_data[..100] {
                    cache.insert(key.clone(), value.clone()).await;
                }

                // Then retrieve
                for (key, _) in &test_data[..100] {
                    black_box(cache.get(key).await);
                }
            });
        });
    });

    group.finish();
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
fn benchmark_cache_hit_rates(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_rates");
    group.measurement_time(Duration::from_secs(10));

    let test_data = create_test_data();

    for hit_rate in &[0.1, 0.5, 0.9] {
        group.bench_with_input(
            BenchmarkId::new("cache_hit_rate", (hit_rate * 100.0) as u32),
            hit_rate,
            |b, &hit_rate| {
                b.iter(|| {
                    let cache = MokaCache::new(100, Duration::from_secs(300));
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        // Insert some data
                        for (key, value) in &test_data[..50] {
                            cache.insert(key.clone(), value.clone()).await;
                        }

                        // Simulate different hit rates
                        let total_requests = 1000;
                        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
                        let hits = (total_requests as f64 * hit_rate) as usize;

                        for i in 0..total_requests {
                            let key = if i < hits {
                                // Cache hit
                                format!("test_key_{}", i % 50)
                            } else {
                                // Cache miss
                                format!("test_key_{}", i + 1000)
                            };
                            black_box(cache.get(&key).await);
                        }
                    });
                });
            },
        );
    }

    group.finish();
}

fn benchmark_cache_key_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_key_generation");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("api_response_key", |b| {
        b.iter(|| {
            for i in 0..1000 {
                black_box(CacheKeyGenerator::api_response(
                    &format!("endpoint_{i}"),
                    &format!("params_{i}"),
                ));
            }
        });
    });

    group.bench_function("search_key", |b| {
        b.iter(|| {
            for i in 0..1000 {
                black_box(CacheKeyGenerator::search(&format!("jql_{i}"), i, i * 2));
            }
        });
    });

    group.bench_function("parsed_object_key", |b| {
        b.iter(|| {
            for i in 0..1000 {
                black_box(CacheKeyGenerator::parsed_object(
                    &format!("type_{i}"),
                    &format!("id_{i}"),
                ));
            }
        });
    });

    group.finish();
}

fn benchmark_cache_manager_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_manager_operations");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("cache_manager_operations", |b| {
        b.iter(|| {
            let cache_manager = CacheManager::new();
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Test API responses cache
                for i in 0..100 {
                    let key = format!("api_response_{i}");
                    let value = json!({
                        "id": i,
                        "data": vec![i; 10]
                    });
                    cache_manager.api_responses.insert(key, value).await;
                }

                // Test parsed objects cache
                for i in 0..100 {
                    let key = format!("parsed_object_{i}");
                    let value = json!({
                        "id": i,
                        "parsed": true
                    });
                    cache_manager.parsed_objects.insert(key, value).await;
                }

                // Test config cache
                for i in 0..50 {
                    let key = format!("config_{i}");
                    let value = json!({
                        "setting": i,
                        "enabled": true
                    });
                    cache_manager.config_cache.insert(key, value).await;
                }

                // Get stats
                black_box(cache_manager.get_stats().await);
            });
        });
    });

    group.finish();
}

fn benchmark_concurrent_cache_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_cache_access");
    group.measurement_time(Duration::from_secs(10));

    for concurrency in &[1, 2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::new("concurrent_cache_ops", concurrency),
            concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    let cache = Arc::new(MokaCache::new(1000, Duration::from_secs(300)));
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let handles: Vec<_> = (0..concurrency)
                            .map(|i| {
                                let cache = Arc::clone(&cache);
                                tokio::spawn(async move {
                                    for j in 0..100 {
                                        let key = format!("key_{i}_{j}");
                                        let value = json!({
                                            "id": i * 100 + j,
                                            "data": vec![i * 100 + j; 5]
                                        });
                                        cache.insert(key.clone(), value).await;
                                        black_box(cache.get(&key).await);
                                    }
                                })
                            })
                            .collect();

                        let results: Vec<_> = futures::future::join_all(handles).await;
                        black_box(results);
                    });
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_cache_insertion,
    benchmark_cache_retrieval,
    benchmark_cache_hit_rates,
    benchmark_cache_key_generation,
    benchmark_cache_manager_operations,
    benchmark_concurrent_cache_access
);
criterion_main!(benches);
