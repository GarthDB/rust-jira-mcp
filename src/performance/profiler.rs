use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Simple profiler for measuring operation performance
#[derive(Debug, Clone)]
pub struct Profiler {
    name: String,
    start_time: Instant,
    checkpoints: Vec<(String, Instant)>,
}

impl Profiler {
    /// Create a new profiler with a name
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start_time: Instant::now(),
            checkpoints: Vec::new(),
        }
    }

    /// Add a checkpoint with a name
    pub fn checkpoint(&mut self, name: impl Into<String>) {
        let checkpoint_name = name.into();
        let now = Instant::now();
        self.checkpoints.push((checkpoint_name.clone(), now));
        debug!(
            "Profiler '{}' checkpoint '{}' at {:?}",
            self.name, checkpoint_name, now
        );
    }

    /// Get the elapsed time since the profiler started
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get the elapsed time since the last checkpoint
    #[must_use]
    pub fn elapsed_since_last_checkpoint(&self) -> Option<Duration> {
        self.checkpoints.last().map(|(_, time)| time.elapsed())
    }

    /// Get detailed timing information
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn get_timing_report(&self) -> TimingReport {
        let total_elapsed = self.elapsed();
        let mut segments = Vec::new();

        if self.checkpoints.is_empty() {
            segments.push(TimingSegment {
                name: "total".to_string(),
                duration: total_elapsed,
                percentage: 100.0,
            });
        } else {
            let mut last_time = self.start_time;

            for (name, time) in &self.checkpoints {
                let segment_duration = time.duration_since(last_time);
                let percentage =
                    (segment_duration.as_nanos() as f64 / total_elapsed.as_nanos() as f64) * 100.0;

                segments.push(TimingSegment {
                    name: name.clone(),
                    duration: segment_duration,
                    percentage,
                });

                last_time = *time;
            }

            // Add final segment if there's time remaining
            if last_time < Instant::now() {
                let final_duration = Instant::now().duration_since(last_time);
                let percentage =
                    (final_duration.as_nanos() as f64 / total_elapsed.as_nanos() as f64) * 100.0;

                segments.push(TimingSegment {
                    name: "final".to_string(),
                    duration: final_duration,
                    percentage,
                });
            }
        }

        TimingReport {
            profiler_name: self.name.clone(),
            total_duration: total_elapsed,
            segments,
        }
    }

    /// Log the timing report
    pub fn log_timing_report(&self) {
        let report = self.get_timing_report();
        info!(
            "Profiler '{}' completed in {:?}",
            report.profiler_name, report.total_duration
        );

        for segment in &report.segments {
            info!(
                "  {}: {:?} ({:.1}%)",
                segment.name, segment.duration, segment.percentage
            );
        }
    }
}

/// Timing segment information
#[derive(Debug, Clone)]
pub struct TimingSegment {
    pub name: String,
    pub duration: Duration,
    pub percentage: f64,
}

/// Complete timing report
#[derive(Debug, Clone)]
pub struct TimingReport {
    pub profiler_name: String,
    pub total_duration: Duration,
    pub segments: Vec<TimingSegment>,
}

/// Macro for easy profiling of code blocks
#[macro_export]
macro_rules! profile {
    ($name:expr, $code:block) => {{
        let mut profiler = $crate::performance::Profiler::new($name);
        let result = $code;
        profiler.log_timing_report();
        result
    }};
}

/// Macro for profiling async code blocks
#[macro_export]
macro_rules! profile_async {
    ($name:expr, $code:block) => {{
        let mut profiler = $crate::performance::Profiler::new($name);
        let result = $code.await;
        profiler.log_timing_report();
        result
    }};
}

/// Performance measurement utilities
pub struct PerformanceUtils;

impl PerformanceUtils {
    /// Measure the execution time of a closure
    pub fn measure_time<F, T>(f: F) -> (T, Duration)
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Measure the execution time of an async closure
    pub async fn measure_time_async<F, T>(f: F) -> (T, Duration)
    where
        F: std::future::Future<Output = T>,
    {
        let start = Instant::now();
        let result = f.await;
        let duration = start.elapsed();
        (result, duration)
    }

    /// Benchmark a function multiple times and return statistics
    #[allow(clippy::cast_possible_truncation)]
    pub fn benchmark<F, T>(iterations: usize, f: F) -> BenchmarkResult
    where
        F: Fn() -> T,
    {
        let mut durations = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let (_, duration) = Self::measure_time(&f);
            durations.push(duration);
        }

        durations.sort();

        let total: Duration = durations.iter().sum();
        let average = total / durations.len() as u32;
        let min = durations[0];
        let max = durations[durations.len() - 1];
        let median = durations[durations.len() / 2];

        BenchmarkResult {
            iterations,
            total_duration: total,
            average_duration: average,
            min_duration: min,
            max_duration: max,
            median_duration: median,
            durations,
        }
    }
}

/// Benchmark result containing timing statistics
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub iterations: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub median_duration: Duration,
    pub durations: Vec<Duration>,
}

impl BenchmarkResult {
    /// Get the operations per second
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn ops_per_second(&self) -> f64 {
        if self.total_duration.as_secs_f64() > 0.0 {
            self.iterations as f64 / self.total_duration.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Log the benchmark results
    pub fn log_results(&self, operation_name: &str) {
        info!("Benchmark Results for '{}':", operation_name);
        info!("  Iterations: {}", self.iterations);
        info!("  Total Duration: {:?}", self.total_duration);
        info!("  Average Duration: {:?}", self.average_duration);
        info!("  Min Duration: {:?}", self.min_duration);
        info!("  Max Duration: {:?}", self.max_duration);
        info!("  Median Duration: {:?}", self.median_duration);
        info!("  Operations per Second: {:.2}", self.ops_per_second());
    }
}
