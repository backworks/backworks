//! Metrics collection and monitoring for proxy operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Proxy operation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyMetrics {
    /// Target name
    pub target_name: String,
    
    /// Total number of requests
    pub total_requests: u64,
    
    /// Number of successful requests (2xx responses)
    pub successful_requests: u64,
    
    /// Number of failed requests (4xx, 5xx responses)
    pub failed_requests: u64,
    
    /// Number of requests that timed out
    pub timeout_requests: u64,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    
    /// Minimum response time in milliseconds
    pub min_response_time_ms: u64,
    
    /// Maximum response time in milliseconds
    pub max_response_time_ms: u64,
    
    /// 95th percentile response time
    pub p95_response_time_ms: u64,
    
    /// 99th percentile response time
    pub p99_response_time_ms: u64,
    
    /// Requests per second (over the last minute)
    pub requests_per_second: f64,
    
    /// Current active connections
    pub active_connections: u32,
    
    /// Circuit breaker state
    pub circuit_breaker_state: String,
    
    /// Health status
    pub is_healthy: bool,
    
    /// Last health check timestamp
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Metrics collection start time
    pub metrics_start_time: chrono::DateTime<chrono::Utc>,
    
    /// Last metrics update time
    pub last_update_time: chrono::DateTime<chrono::Utc>,
}

impl ProxyMetrics {
    pub fn new(target_name: String) -> Self {
        let now = chrono::Utc::now();
        
        Self {
            target_name,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            timeout_requests: 0,
            avg_response_time_ms: 0.0,
            min_response_time_ms: 0,
            max_response_time_ms: 0,
            p95_response_time_ms: 0,
            p99_response_time_ms: 0,
            requests_per_second: 0.0,
            active_connections: 0,
            circuit_breaker_state: "Closed".to_string(),
            is_healthy: true,
            last_health_check: None,
            metrics_start_time: now,
            last_update_time: now,
        }
    }

    /// Calculate success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    /// Calculate error rate as a percentage
    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.failed_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    /// Calculate uptime duration
    pub fn uptime(&self) -> Duration {
        let now = chrono::Utc::now();
        (now - self.metrics_start_time).to_std().unwrap_or(Duration::ZERO)
    }
}

/// Request timing information
#[derive(Debug, Clone)]
struct RequestTiming {
    /// Request start time
    start_time: Instant,
    
    /// Request duration
    duration: Duration,
    
    /// HTTP status code
    #[allow(dead_code)]
    status_code: u16,
    
    /// Whether the request was successful
    success: bool,
    
    /// Whether the request timed out
    timeout: bool,
}

/// Metrics collector for a single target
#[derive(Debug)]
struct TargetMetricsCollector {
    /// Target name
    #[allow(dead_code)]
    target_name: String,
    
    /// Current metrics
    metrics: ProxyMetrics,
    
    /// Recent request timings (for percentile calculations)
    recent_timings: Vec<RequestTiming>,
    
    /// Request count in the current minute
    requests_this_minute: Vec<Instant>,
    
    /// Maximum number of recent timings to keep
    max_recent_timings: usize,
}

impl TargetMetricsCollector {
    fn new(target_name: String) -> Self {
        Self {
            target_name: target_name.clone(),
            metrics: ProxyMetrics::new(target_name),
            recent_timings: Vec::new(),
            requests_this_minute: Vec::new(),
            max_recent_timings: 1000, // Keep last 1000 requests for percentiles
        }
    }

    fn record_request(&mut self, timing: RequestTiming) {
        // Update basic counters
        self.metrics.total_requests += 1;
        self.metrics.last_update_time = chrono::Utc::now();

        if timing.success {
            self.metrics.successful_requests += 1;
        } else {
            self.metrics.failed_requests += 1;
        }

        if timing.timeout {
            self.metrics.timeout_requests += 1;
        }

        // Update response time metrics
        let duration_ms = timing.duration.as_millis() as u64;
        
        if self.metrics.total_requests == 1 {
            // First request
            self.metrics.min_response_time_ms = duration_ms;
            self.metrics.max_response_time_ms = duration_ms;
            self.metrics.avg_response_time_ms = duration_ms as f64;
        } else {
            // Update min/max
            self.metrics.min_response_time_ms = self.metrics.min_response_time_ms.min(duration_ms);
            self.metrics.max_response_time_ms = self.metrics.max_response_time_ms.max(duration_ms);
            
            // Update rolling average
            let prev_avg = self.metrics.avg_response_time_ms;
            let count = self.metrics.total_requests as f64;
            self.metrics.avg_response_time_ms = ((prev_avg * (count - 1.0)) + duration_ms as f64) / count;
        }

        // Store timing for percentile calculations
        self.recent_timings.push(timing.clone());
        if self.recent_timings.len() > self.max_recent_timings {
            self.recent_timings.remove(0);
        }

        // Track requests for RPS calculation
        self.requests_this_minute.push(timing.start_time);
        self.cleanup_old_requests();
        
        // Update percentiles
        self.update_percentiles();
        
        // Update RPS
        self.update_requests_per_second();
    }

    fn cleanup_old_requests(&mut self) {
        let cutoff = Instant::now() - Duration::from_secs(60);
        self.requests_this_minute.retain(|&instant| instant > cutoff);
    }

    fn update_percentiles(&mut self) {
        if self.recent_timings.is_empty() {
            return;
        }

        let mut durations: Vec<u64> = self.recent_timings.iter()
            .map(|t| t.duration.as_millis() as u64)
            .collect();
        
        durations.sort_unstable();
        
        let len = durations.len();
        if len > 0 {
            let p95_index = ((len as f64) * 0.95) as usize;
            let p99_index = ((len as f64) * 0.99) as usize;
            
            self.metrics.p95_response_time_ms = durations[p95_index.min(len - 1)];
            self.metrics.p99_response_time_ms = durations[p99_index.min(len - 1)];
        }
    }

    fn update_requests_per_second(&mut self) {
        let requests_count = self.requests_this_minute.len() as f64;
        
        if requests_count > 0.0 {
            // Calculate RPS over the actual time window
            if let (Some(&first), Some(&last)) = (self.requests_this_minute.first(), self.requests_this_minute.last()) {
                let window_duration = last.duration_since(first).as_secs_f64();
                if window_duration > 0.0 {
                    self.metrics.requests_per_second = requests_count / window_duration.max(1.0);
                } else {
                    self.metrics.requests_per_second = requests_count; // All requests in same instant
                }
            }
        } else {
            self.metrics.requests_per_second = 0.0;
        }
    }

    fn update_health_status(&mut self, healthy: bool) {
        self.metrics.is_healthy = healthy;
        self.metrics.last_health_check = Some(chrono::Utc::now());
    }

    fn update_circuit_breaker_state(&mut self, state: &str) {
        self.metrics.circuit_breaker_state = state.to_string();
    }

    fn increment_active_connections(&mut self) {
        self.metrics.active_connections += 1;
    }

    fn decrement_active_connections(&mut self) {
        self.metrics.active_connections = self.metrics.active_connections.saturating_sub(1);
    }

    fn get_metrics(&self) -> ProxyMetrics {
        self.metrics.clone()
    }
}

/// Main metrics manager
#[derive(Debug)]
pub struct ProxyMetricsManager {
    /// Metrics collectors for each target
    collectors: Arc<RwLock<HashMap<String, TargetMetricsCollector>>>,
}

impl ProxyMetricsManager {
    pub fn new() -> Self {
        Self {
            collectors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a target for metrics collection
    pub async fn add_target(&self, target_name: String) {
        let mut collectors = self.collectors.write().await;
        collectors.insert(target_name.clone(), TargetMetricsCollector::new(target_name));
    }

    /// Remove a target from metrics collection
    pub async fn remove_target(&self, target_name: &str) {
        let mut collectors = self.collectors.write().await;
        collectors.remove(target_name);
    }

    /// Record a request completion
    pub async fn record_request_completion(
        &self,
        target_name: &str,
        start_time: Instant,
        status_code: u16,
        timeout: bool,
    ) {
        let mut collectors = self.collectors.write().await;
        
        if let Some(collector) = collectors.get_mut(target_name) {
            let duration = start_time.elapsed();
            let success = (200..300).contains(&status_code) && !timeout;
            
            let timing = RequestTiming {
                start_time,
                duration,
                status_code,
                success,
                timeout,
            };
            
            collector.record_request(timing);
        }
    }

    /// Record the start of a request (for active connection tracking)
    pub async fn record_request_start(&self, target_name: &str) {
        let mut collectors = self.collectors.write().await;
        
        if let Some(collector) = collectors.get_mut(target_name) {
            collector.increment_active_connections();
        }
    }

    /// Record the end of a request (for active connection tracking)
    pub async fn record_request_end(&self, target_name: &str) {
        let mut collectors = self.collectors.write().await;
        
        if let Some(collector) = collectors.get_mut(target_name) {
            collector.decrement_active_connections();
        }
    }

    /// Update health status for a target
    pub async fn update_health_status(&self, target_name: &str, healthy: bool) {
        let mut collectors = self.collectors.write().await;
        
        if let Some(collector) = collectors.get_mut(target_name) {
            collector.update_health_status(healthy);
        }
    }

    /// Update circuit breaker state for a target
    pub async fn update_circuit_breaker_state(&self, target_name: &str, state: &str) {
        let mut collectors = self.collectors.write().await;
        
        if let Some(collector) = collectors.get_mut(target_name) {
            collector.update_circuit_breaker_state(state);
        }
    }

    /// Get metrics for a specific target
    pub async fn get_target_metrics(&self, target_name: &str) -> Option<ProxyMetrics> {
        let collectors = self.collectors.read().await;
        collectors.get(target_name).map(|c| c.get_metrics())
    }

    /// Get metrics for all targets
    pub async fn get_all_metrics(&self) -> HashMap<String, ProxyMetrics> {
        let collectors = self.collectors.read().await;
        collectors.iter().map(|(name, collector)| {
            (name.clone(), collector.get_metrics())
        }).collect()
    }

    /// Get aggregated metrics across all targets
    pub async fn get_aggregated_metrics(&self) -> ProxyMetrics {
        let collectors = self.collectors.read().await;
        
        if collectors.is_empty() {
            return ProxyMetrics::new("aggregated".to_string());
        }

        let mut aggregated = ProxyMetrics::new("aggregated".to_string());
        let mut total_avg_weighted_by_requests = 0.0;
        let mut response_times = Vec::new();
        let mut earliest_start_time = chrono::Utc::now();
        let mut latest_update_time = chrono::DateTime::<chrono::Utc>::MIN_UTC;

        for (_, collector) in collectors.iter() {
            let metrics = &collector.metrics;
            
            aggregated.total_requests += metrics.total_requests;
            aggregated.successful_requests += metrics.successful_requests;
            aggregated.failed_requests += metrics.failed_requests;
            aggregated.timeout_requests += metrics.timeout_requests;
            aggregated.active_connections += metrics.active_connections;
            
            // Weighted average for response time
            if metrics.total_requests > 0 {
                total_avg_weighted_by_requests += metrics.avg_response_time_ms * metrics.total_requests as f64;
            }
            
            // Collect response times for percentiles
            response_times.extend(collector.recent_timings.iter().map(|t| t.duration.as_millis() as u64));
            
            // Track time ranges
            if metrics.metrics_start_time < earliest_start_time {
                earliest_start_time = metrics.metrics_start_time;
            }
            if metrics.last_update_time > latest_update_time {
                latest_update_time = metrics.last_update_time;
            }
            
            // Min/max response times
            if aggregated.total_requests > 0 {
                if aggregated.min_response_time_ms == 0 || metrics.min_response_time_ms < aggregated.min_response_time_ms {
                    aggregated.min_response_time_ms = metrics.min_response_time_ms;
                }
                if metrics.max_response_time_ms > aggregated.max_response_time_ms {
                    aggregated.max_response_time_ms = metrics.max_response_time_ms;
                }
            }
        }

        // Calculate weighted average response time
        if aggregated.total_requests > 0 {
            aggregated.avg_response_time_ms = total_avg_weighted_by_requests / aggregated.total_requests as f64;
        }

        // Calculate aggregated percentiles
        if !response_times.is_empty() {
            response_times.sort_unstable();
            let len = response_times.len();
            let p95_index = ((len as f64) * 0.95) as usize;
            let p99_index = ((len as f64) * 0.99) as usize;
            
            aggregated.p95_response_time_ms = response_times[p95_index.min(len - 1)];
            aggregated.p99_response_time_ms = response_times[p99_index.min(len - 1)];
        }

        // Calculate aggregated RPS
        let time_window = (latest_update_time - earliest_start_time).num_seconds() as f64;
        if time_window > 0.0 {
            aggregated.requests_per_second = aggregated.total_requests as f64 / time_window;
        }

        aggregated.metrics_start_time = earliest_start_time;
        aggregated.last_update_time = latest_update_time;

        aggregated
    }

    /// Reset metrics for a target
    pub async fn reset_target_metrics(&self, target_name: &str) {
        let mut collectors = self.collectors.write().await;
        
        if let Some(collector) = collectors.get_mut(target_name) {
            *collector = TargetMetricsCollector::new(target_name.to_string());
        }
    }

    /// Reset all metrics
    pub async fn reset_all_metrics(&self) {
        let mut collectors = self.collectors.write().await;
        
        for (target_name, collector) in collectors.iter_mut() {
            *collector = TargetMetricsCollector::new(target_name.clone());
        }
    }
}

impl Default for ProxyMetricsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn test_metrics_manager_creation() {
        let manager = ProxyMetricsManager::new();
        
        manager.add_target("test-target".to_string()).await;
        
        let metrics = manager.get_target_metrics("test-target").await;
        assert!(metrics.is_some());
        
        let metrics = metrics.unwrap();
        assert_eq!(metrics.target_name, "test-target");
        assert_eq!(metrics.total_requests, 0);
    }

    #[tokio::test]
    async fn test_request_recording() {
        let manager = ProxyMetricsManager::new();
        manager.add_target("test-target".to_string()).await;
        
        let start_time = Instant::now();
        
        // Simulate a successful request
        manager.record_request_start("test-target").await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        manager.record_request_completion("test-target", start_time, 200, false).await;
        manager.record_request_end("test-target").await;
        
        let metrics = manager.get_target_metrics("test-target").await.unwrap();
        
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.failed_requests, 0);
        assert!(metrics.avg_response_time_ms >= 10.0);
    }

    #[tokio::test]
    async fn test_failed_request_recording() {
        let manager = ProxyMetricsManager::new();
        manager.add_target("test-target".to_string()).await;
        
        let start_time = Instant::now();
        
        // Simulate a failed request
        manager.record_request_completion("test-target", start_time, 500, false).await;
        
        let metrics = manager.get_target_metrics("test-target").await.unwrap();
        
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 0);
        assert_eq!(metrics.failed_requests, 1);
        assert_eq!(metrics.error_rate(), 100.0);
    }

    #[tokio::test]
    async fn test_timeout_request_recording() {
        let manager = ProxyMetricsManager::new();
        manager.add_target("test-target".to_string()).await;
        
        let start_time = Instant::now();
        
        // Simulate a timeout request
        manager.record_request_completion("test-target", start_time, 0, true).await;
        
        let metrics = manager.get_target_metrics("test-target").await.unwrap();
        
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.timeout_requests, 1);
        assert_eq!(metrics.failed_requests, 1);
    }

    #[tokio::test]
    async fn test_health_status_update() {
        let manager = ProxyMetricsManager::new();
        manager.add_target("test-target".to_string()).await;
        
        manager.update_health_status("test-target", false).await;
        
        let metrics = manager.get_target_metrics("test-target").await.unwrap();
        
        assert!(!metrics.is_healthy);
        assert!(metrics.last_health_check.is_some());
    }

    #[tokio::test]
    async fn test_aggregated_metrics() {
        let manager = ProxyMetricsManager::new();
        
        // Add multiple targets
        manager.add_target("target1".to_string()).await;
        manager.add_target("target2".to_string()).await;
        
        // Record some requests
        let start_time = Instant::now();
        manager.record_request_completion("target1", start_time, 200, false).await;
        manager.record_request_completion("target2", start_time, 200, false).await;
        manager.record_request_completion("target1", start_time, 500, false).await;
        
        let aggregated = manager.get_aggregated_metrics().await;
        
        assert_eq!(aggregated.total_requests, 3);
        assert_eq!(aggregated.successful_requests, 2);
        assert_eq!(aggregated.failed_requests, 1);
    }
}
