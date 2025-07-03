//! Circuit breaker implementation for proxy fault tolerance

use crate::error::ProxyResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

/// Circuit breaker state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CircuitBreakerState {
    /// Circuit is closed - requests flow normally
    Closed,
    
    /// Circuit is open - requests are blocked
    Open,
    
    /// Circuit is half-open - limited requests allowed to test recovery
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    
    /// Time to wait before attempting recovery
    pub recovery_timeout: Duration,
    
    /// Minimum number of requests before evaluating failure rate
    pub request_volume_threshold: u32,
    
    /// Success rate threshold for closing circuit (0.0 to 1.0)
    pub success_rate_threshold: f64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            request_volume_threshold: 20,
            success_rate_threshold: 0.5,
        }
    }
}

/// Circuit breaker metrics
#[derive(Debug, Clone)]
struct CircuitBreakerMetrics {
    /// Total requests in current window
    request_count: u32,
    
    /// Failed requests in current window
    failure_count: u32,
    
    /// Successful requests in current window
    success_count: u32,
    
    /// Last failure time
    last_failure_time: Option<Instant>,
    
    /// Last success time
    last_success_time: Option<Instant>,
    
    /// Window start time
    window_start: Instant,
}

impl CircuitBreakerMetrics {
    fn new() -> Self {
        Self {
            request_count: 0,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            last_success_time: None,
            window_start: Instant::now(),
        }
    }

    fn reset(&mut self) {
        self.request_count = 0;
        self.failure_count = 0;
        self.success_count = 0;
        self.window_start = Instant::now();
    }

    fn record_success(&mut self) {
        self.request_count += 1;
        self.success_count += 1;
        self.last_success_time = Some(Instant::now());
    }

    fn record_failure(&mut self) {
        self.request_count += 1;
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());
    }

    fn failure_rate(&self) -> f64 {
        if self.request_count == 0 {
            0.0
        } else {
            self.failure_count as f64 / self.request_count as f64
        }
    }

    fn success_rate(&self) -> f64 {
        if self.request_count == 0 {
            0.0
        } else {
            self.success_count as f64 / self.request_count as f64
        }
    }
}

/// Circuit breaker implementation
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,
    metrics: Arc<RwLock<CircuitBreakerMetrics>>,
    last_state_change: Arc<RwLock<Instant>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            metrics: Arc::new(RwLock::new(CircuitBreakerMetrics::new())),
            last_state_change: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Check if a request should be allowed through
    pub async fn should_allow_request(&self) -> ProxyResult<bool> {
        let state = self.state.read().await;
        
        match *state {
            CircuitBreakerState::Closed => Ok(true),
            CircuitBreakerState::Open => {
                drop(state);
                self.check_recovery().await
            }
            CircuitBreakerState::HalfOpen => {
                // Allow limited requests in half-open state
                Ok(true)
            }
        }
    }

    /// Record a successful request
    pub async fn record_success(&self) -> ProxyResult<()> {
        let mut metrics = self.metrics.write().await;
        metrics.record_success();
        
        let state = self.state.read().await;
        match *state {
            CircuitBreakerState::HalfOpen => {
                drop(state);
                drop(metrics);
                // If we have enough successful requests, close the circuit
                if self.should_close_circuit().await? {
                    self.close_circuit().await?;
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    /// Record a failed request
    pub async fn record_failure(&self) -> ProxyResult<()> {
        let mut metrics = self.metrics.write().await;
        metrics.record_failure();
        
        let state = self.state.read().await;
        match *state {
            CircuitBreakerState::Closed => {
                drop(state);
                drop(metrics);
                // Check if we should open the circuit
                if self.should_open_circuit().await? {
                    self.open_circuit().await?;
                }
            }
            CircuitBreakerState::HalfOpen => {
                drop(state);
                drop(metrics);
                // Any failure in half-open state opens the circuit
                self.open_circuit().await?;
            }
            _ => {}
        }
        
        Ok(())
    }

    /// Get current circuit breaker state
    pub async fn get_state(&self) -> CircuitBreakerState {
        let state = self.state.read().await;
        state.clone()
    }

    /// Get circuit breaker metrics
    pub async fn get_metrics(&self) -> (u32, u32, f64) {
        let metrics = self.metrics.read().await;
        (metrics.request_count, metrics.failure_count, metrics.failure_rate())
    }

    /// Check if circuit should be opened
    async fn should_open_circuit(&self) -> ProxyResult<bool> {
        let metrics = self.metrics.read().await;
        
        // Need minimum request volume to make decision
        if metrics.request_count < self.config.request_volume_threshold {
            return Ok(false);
        }
        
        // Open if failure count exceeds threshold
        Ok(metrics.failure_count >= self.config.failure_threshold)
    }

    /// Check if circuit should be closed
    async fn should_close_circuit(&self) -> ProxyResult<bool> {
        let metrics = self.metrics.read().await;
        
        // Need minimum request volume to make decision
        if metrics.request_count < self.config.request_volume_threshold {
            return Ok(false);
        }
        
        // Close if success rate exceeds threshold
        Ok(metrics.success_rate() >= self.config.success_rate_threshold)
    }

    /// Check if circuit should transition to half-open
    async fn check_recovery(&self) -> ProxyResult<bool> {
        let last_change = self.last_state_change.read().await;
        let elapsed = last_change.elapsed();
        
        if elapsed >= self.config.recovery_timeout {
            drop(last_change);
            self.half_open_circuit().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Open the circuit
    async fn open_circuit(&self) -> ProxyResult<()> {
        let mut state = self.state.write().await;
        *state = CircuitBreakerState::Open;
        
        let mut last_change = self.last_state_change.write().await;
        *last_change = Instant::now();
        
        tracing::warn!("Circuit breaker opened due to high failure rate");
        Ok(())
    }

    /// Close the circuit
    async fn close_circuit(&self) -> ProxyResult<()> {
        let mut state = self.state.write().await;
        *state = CircuitBreakerState::Closed;
        
        let mut last_change = self.last_state_change.write().await;
        *last_change = Instant::now();
        
        // Reset metrics
        let mut metrics = self.metrics.write().await;
        metrics.reset();
        
        tracing::info!("Circuit breaker closed - service recovered");
        Ok(())
    }

    /// Set circuit to half-open
    async fn half_open_circuit(&self) -> ProxyResult<()> {
        let mut state = self.state.write().await;
        *state = CircuitBreakerState::HalfOpen;
        
        let mut last_change = self.last_state_change.write().await;
        *last_change = Instant::now();
        
        // Reset metrics for half-open evaluation
        let mut metrics = self.metrics.write().await;
        metrics.reset();
        
        tracing::info!("Circuit breaker half-open - testing recovery");
        Ok(())
    }

    /// Force circuit state (for testing)
    #[cfg(test)]
    pub async fn force_state(&self, state: CircuitBreakerState) {
        let mut current_state = self.state.write().await;
        *current_state = state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(1),
            request_volume_threshold: 5,
            success_rate_threshold: 0.5,
        };
        
        let cb = CircuitBreaker::new(config);
        
        // Initially closed
        assert_eq!(cb.get_state().await, CircuitBreakerState::Closed);
        assert!(cb.should_allow_request().await.unwrap());
        
        // Record some successful requests
        cb.record_success().await.unwrap();
        cb.record_success().await.unwrap();
        
        // Should still be closed
        assert_eq!(cb.get_state().await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_open_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(1),
            request_volume_threshold: 5,
            success_rate_threshold: 0.5,
        };
        
        let cb = CircuitBreaker::new(config);
        
        // Record enough failures to open circuit
        cb.record_failure().await.unwrap();
        cb.record_failure().await.unwrap();
        cb.record_failure().await.unwrap();
        cb.record_failure().await.unwrap();
        cb.record_failure().await.unwrap();
        
        // Circuit should be open
        assert_eq!(cb.get_state().await, CircuitBreakerState::Open);
        assert!(!cb.should_allow_request().await.unwrap());
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_millis(100),
            request_volume_threshold: 5,
            success_rate_threshold: 0.5,
        };
        
        let cb = CircuitBreaker::new(config);
        
        // Force circuit to open
        cb.force_state(CircuitBreakerState::Open).await;
        
        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should allow request and transition to half-open
        assert!(cb.should_allow_request().await.unwrap());
        assert_eq!(cb.get_state().await, CircuitBreakerState::HalfOpen);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_to_closed() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(1),
            request_volume_threshold: 5,
            success_rate_threshold: 0.8,
        };
        
        let cb = CircuitBreaker::new(config);
        
        // Force circuit to half-open
        cb.force_state(CircuitBreakerState::HalfOpen).await;
        
        // Record enough successful requests to close circuit
        for _ in 0..5 {
            cb.record_success().await.unwrap();
        }
        
        // Circuit should be closed
        assert_eq!(cb.get_state().await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_to_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(1),
            request_volume_threshold: 5,
            success_rate_threshold: 0.8,
        };
        
        let cb = CircuitBreaker::new(config);
        
        // Force circuit to half-open
        cb.force_state(CircuitBreakerState::HalfOpen).await;
        
        // Record a failure - should open circuit
        cb.record_failure().await.unwrap();
        
        // Circuit should be open
        assert_eq!(cb.get_state().await, CircuitBreakerState::Open);
    }
}
