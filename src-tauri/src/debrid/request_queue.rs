// Request queue for rate-limited API calls

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex as TokioMutex, Semaphore};
use tokio::time::sleep;
use anyhow::Result;
use tracing::debug;

/// Request queue that enforces rate limiting
pub struct RequestQueue {
    /// Minimum interval between requests
    min_interval: Duration,
    /// Last request time
    last_request: Arc<TokioMutex<Option<Instant>>>,
    /// Semaphore to ensure sequential processing
    semaphore: Arc<Semaphore>,
    /// Provider name for logging
    provider_name: String,
}

impl RequestQueue {
    /// Create a new request queue
    /// 
    /// # Arguments
    /// * `min_interval_ms` - Minimum milliseconds between requests
    /// * `provider_name` - Name of the provider (for logging)
    pub fn new(min_interval_ms: u64, provider_name: String) -> Self {
        Self {
            min_interval: Duration::from_millis(min_interval_ms),
            last_request: Arc::new(TokioMutex::new(None)),
            semaphore: Arc::new(Semaphore::new(1)), // Only one request at a time
            provider_name,
        }
    }

    /// Execute a request with rate limiting
    /// 
    /// # Arguments
    /// * `request_fn` - Async function that performs the request
    /// 
    /// # Returns
    /// Result from the request function
    pub async fn execute<F, T>(&self, request_fn: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        // Acquire semaphore to ensure sequential execution
        let _permit = self.semaphore.acquire().await.expect("Semaphore closed");
        
        // Check if we need to wait
        let mut last_req = self.last_request.lock().await;
        
        if let Some(last_time) = *last_req {
            let elapsed = last_time.elapsed();
            
            if elapsed < self.min_interval {
                let wait_time = self.min_interval - elapsed;
                debug!(
                    "[{}] Rate limiting: waiting {:?}",
                    self.provider_name, wait_time
                );
                drop(last_req); // Release lock before sleeping
                sleep(wait_time).await;
                last_req = self.last_request.lock().await;
            }
        }
        
        // Update last request time
        *last_req = Some(Instant::now());
        drop(last_req);
        
        // Execute the request
        debug!("[{}] Executing queued request", self.provider_name);
        request_fn.await
    }

    /// Execute multiple requests with proper rate limiting
    /// 
    /// # Arguments
    /// * `requests` - Vector of async request functions
    /// 
    /// # Returns
    /// Vector of results in the same order
    pub async fn execute_batch<F, T>(&self, requests: Vec<F>) -> Vec<Result<T>>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        let mut results = Vec::with_capacity(requests.len());
        
        for request in requests {
            let result = self.execute(request).await;
            results.push(result);
        }
        
        results
    }
}

/// Queue statistics
pub struct QueueStats {
    pub pending_requests: usize,
    pub last_request_time: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_rate_limiting() {
        let queue = RequestQueue::new(100, "test".to_string());
        let counter = Arc::new(AtomicUsize::new(0));
        
        let start = Instant::now();
        
        // Execute 3 requests
        for _ in 0..3 {
            let counter_clone = counter.clone();
            queue
                .execute(async move {
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                    Ok::<(), anyhow::Error>(())
                })
                .await
                .unwrap();
        }
        
        let elapsed = start.elapsed();
        
        // Should have taken at least 200ms (2 intervals between 3 requests)
        assert!(elapsed >= Duration::from_millis(200));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_sequential_execution() {
        let queue = RequestQueue::new(10, "test".to_string());
        let order = Arc::new(TokioMutex::new(Vec::new()));
        
        // Spawn multiple tasks that will queue up
        let mut handles = vec![];
        
        for i in 0..5 {
            let queue_clone = queue.clone();
            let order_clone = order.clone();
            
            let handle = tokio::spawn(async move {
                queue_clone
                    .execute(async move {
                        order_clone.lock().await.push(i);
                        Ok::<(), anyhow::Error>(())
                    })
                    .await
                    .unwrap();
            });
            
            handles.push(handle);
        }
        
        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Check that all requests were executed
        let final_order = order.lock().await;
        assert_eq!(final_order.len(), 5);
    }
}

// Implement Clone for RequestQueue
impl Clone for RequestQueue {
    fn clone(&self) -> Self {
        Self {
            min_interval: self.min_interval,
            last_request: self.last_request.clone(),
            semaphore: self.semaphore.clone(),
            provider_name: self.provider_name.clone(),
        }
    }
}
