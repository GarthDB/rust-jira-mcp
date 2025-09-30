use crate::performance::PerformanceMetrics;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::task::JoinHandle;
use tracing::{debug, warn};

/// Async task manager for optimized concurrent operations
pub struct AsyncTaskManager {
    semaphore: Arc<Semaphore>,
    metrics: Arc<PerformanceMetrics>,
    max_concurrent_tasks: usize,
    task_queue: Arc<RwLock<Vec<TaskInfo>>>,
}

/// Information about a running task
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub start_time: Instant,
    pub status: TaskStatus,
}

/// Task status
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl AsyncTaskManager {
    /// Create a new async task manager
    #[must_use]
    pub fn new(max_concurrent_tasks: usize, metrics: Arc<PerformanceMetrics>) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
            metrics,
            max_concurrent_tasks,
            task_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Execute a task with concurrency control
    ///
    /// # Errors
    ///
    /// Returns an error if the task execution fails or if there are issues with concurrency control.
    pub async fn execute_task<F, T>(
        &self,
        task_id: String,
        task_name: String,
        task: F,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + 'static,
        T: Send + 'static,
    {
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            Box::new(std::io::Error::other(format!(
                "Failed to acquire semaphore: {e}"
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;

        let start_time = Instant::now();
        let task_info = TaskInfo {
            id: task_id.clone(),
            name: task_name.clone(),
            start_time,
            status: TaskStatus::Running,
        };

        // Add task to queue
        {
            let mut queue = self.task_queue.write().await;
            queue.push(task_info);
        }

        debug!("Starting task: {} ({})", task_name, task_id);

        let result = task.await;
        let duration = start_time.elapsed();

        // Update task status
        {
            let mut queue = self.task_queue.write().await;
            if let Some(task_info) = queue.iter_mut().find(|t| t.id == task_id) {
                task_info.status = if result.is_ok() {
                    TaskStatus::Completed
                } else {
                    TaskStatus::Failed
                };
            }
        }

        // Record metrics
        self.metrics.record_request(duration, result.is_ok());

        debug!(
            "Task completed: {} ({}) in {:?} - Success: {}",
            task_name,
            task_id,
            duration,
            result.is_ok()
        );

        result
    }

    /// Execute multiple tasks concurrently with controlled concurrency
    pub async fn execute_tasks_concurrent<F, T>(
        &self,
        tasks: Vec<(String, String, F)>,
    ) -> Vec<Result<T, Box<dyn std::error::Error + Send + Sync>>>
    where
        F: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + 'static,
        T: Send + 'static,
    {
        let handles: Vec<JoinHandle<Result<T, Box<dyn std::error::Error + Send + Sync>>>> = tasks
            .into_iter()
            .map(|(task_id, task_name, task)| {
                let manager = self.clone();
                tokio::spawn(async move { manager.execute_task(task_id, task_name, task).await })
            })
            .collect();

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Task join error: {}", e);
                    results.push(Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>));
                }
            }
        }

        results
    }

    /// Execute tasks in batches to avoid overwhelming the system
    pub async fn execute_tasks_batched<F, T>(
        &self,
        tasks: Vec<(String, String, F)>,
        batch_size: usize,
        batch_delay: Duration,
    ) -> Vec<Result<T, Box<dyn std::error::Error + Send + Sync>>>
    where
        F: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + 'static
            + Clone,
        T: Send + 'static,
    {
        let mut all_results = Vec::new();

        for chunk in tasks.chunks(batch_size) {
            let batch_tasks = chunk.to_vec();
            let batch_results = self.execute_tasks_concurrent(batch_tasks).await;
            all_results.extend(batch_results);

            // Add delay between batches
            if chunk.len() == batch_size {
                tokio::time::sleep(batch_delay).await;
            }
        }

        all_results
    }

    /// Get current task queue status
    pub async fn get_task_status(&self) -> Vec<TaskInfo> {
        let queue = self.task_queue.read().await;
        queue.clone()
    }

    /// Get active task count
    pub async fn get_active_task_count(&self) -> usize {
        let queue = self.task_queue.read().await;
        queue
            .iter()
            .filter(|t| t.status == TaskStatus::Running)
            .count()
    }

    /// Clear completed tasks from queue
    pub async fn cleanup_completed_tasks(&self) {
        let mut queue = self.task_queue.write().await;
        queue.retain(|t| t.status == TaskStatus::Running || t.status == TaskStatus::Pending);
    }
}

impl Clone for AsyncTaskManager {
    fn clone(&self) -> Self {
        Self {
            semaphore: self.semaphore.clone(),
            metrics: self.metrics.clone(),
            max_concurrent_tasks: self.max_concurrent_tasks,
            task_queue: self.task_queue.clone(),
        }
    }
}

/// Async rate limiter for controlling request rates
pub struct AsyncRateLimiter {
    semaphore: Arc<Semaphore>,
    interval: Duration,
    last_request: Arc<RwLock<Instant>>,
}

impl AsyncRateLimiter {
    /// Create a new async rate limiter
    #[must_use]
    pub fn new(max_requests_per_interval: usize, interval: Duration) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_requests_per_interval)),
            interval,
            last_request: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Wait for rate limit permission
    pub async fn wait(&self) {
        let _permit = self.semaphore.acquire().await;

        let now = Instant::now();
        let elapsed = {
            let last_request = self.last_request.read().await;
            now.duration_since(*last_request)
        };

        if elapsed < self.interval {
            let sleep_duration = self.interval - elapsed;
            debug!("Rate limiting: sleeping for {:?}", sleep_duration);
            tokio::time::sleep(sleep_duration).await;
        }

        {
            let mut last_request = self.last_request.write().await;
            *last_request = Instant::now();
        }
    }
}

/// Async connection pool manager
pub struct AsyncConnectionPool {
    max_connections: usize,
    active_connections: Arc<RwLock<usize>>,
    semaphore: Arc<Semaphore>,
}

impl AsyncConnectionPool {
    /// Create a new connection pool
    #[must_use]
    pub fn new(max_connections: usize) -> Self {
        Self {
            max_connections,
            active_connections: Arc::new(RwLock::new(0)),
            semaphore: Arc::new(Semaphore::new(max_connections)),
        }
    }

    /// Acquire a connection
    pub async fn acquire(&self) -> ConnectionGuard {
        let _permit = self.semaphore.acquire().await;

        {
            let mut active = self.active_connections.write().await;
            *active += 1;
        }

        ConnectionGuard { pool: self.clone() }
    }

    /// Get current connection count
    pub async fn get_connection_count(&self) -> usize {
        let active = self.active_connections.read().await;
        *active
    }
}

impl Clone for AsyncConnectionPool {
    fn clone(&self) -> Self {
        Self {
            max_connections: self.max_connections,
            active_connections: self.active_connections.clone(),
            semaphore: self.semaphore.clone(),
        }
    }
}

/// Connection guard that automatically releases the connection
pub struct ConnectionGuard {
    pool: AsyncConnectionPool,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let pool = self.pool.clone();
        tokio::spawn(async move {
            let mut active = pool.active_connections.write().await;
            *active = active.saturating_sub(1);
        });
    }
}

/// Async batch processor for efficient bulk operations
pub struct AsyncBatchProcessor<T> {
    batch_size: usize,
    batch_timeout: Duration,
    pending_items: Arc<RwLock<Vec<T>>>,
    last_batch_time: Arc<RwLock<Instant>>,
}

impl<T> AsyncBatchProcessor<T>
where
    T: Send + Sync + 'static,
{
    /// Create a new batch processor
    #[must_use]
    pub fn new(batch_size: usize, batch_timeout: Duration) -> Self {
        Self {
            batch_size,
            batch_timeout,
            pending_items: Arc::new(RwLock::new(Vec::new())),
            last_batch_time: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Add an item to the batch
    pub async fn add_item(&self, item: T) {
        let mut items = self.pending_items.write().await;
        items.push(item);
    }

    /// Process items when batch is ready
    pub async fn process_batch<F, Fut, R>(&self, processor: F) -> Vec<R>
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Vec<R>> + Send,
        R: Send + 'static,
    {
        let should_process = {
            let items = self.pending_items.read().await;
            let last_batch = self.last_batch_time.read().await;

            items.len() >= self.batch_size || last_batch.elapsed() >= self.batch_timeout
        };

        if should_process {
            let items = {
                let mut pending = self.pending_items.write().await;
                let items = pending.drain(..).collect::<Vec<_>>();
                *self.last_batch_time.write().await = Instant::now();
                items
            };

            if items.is_empty() {
                Vec::new()
            } else {
                processor(items).await
            }
        } else {
            Vec::new()
        }
    }
}

impl<T> Clone for AsyncBatchProcessor<T> {
    fn clone(&self) -> Self {
        Self {
            batch_size: self.batch_size,
            batch_timeout: self.batch_timeout,
            pending_items: self.pending_items.clone(),
            last_batch_time: self.last_batch_time.clone(),
        }
    }
}

static GLOBAL_TASK_MANAGER: std::sync::LazyLock<AsyncTaskManager> =
    std::sync::LazyLock::new(|| {
        AsyncTaskManager::new(
            50, // Max 50 concurrent tasks
            crate::performance::get_global_metrics(),
        )
    });

/// Get the global task manager
#[must_use]
pub fn get_global_task_manager() -> &'static AsyncTaskManager {
    &GLOBAL_TASK_MANAGER
}
