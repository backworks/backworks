//! Plugin resilience and error isolation
//! 
//! Provides circuit breakers, resource limits, and performance monitoring
//! to ensure plugin failures don't affect the core system.

use crate::error::BackworksResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::timeout;

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Failing, blocking requests
    HalfOpen, // Testing if service recovered
}

/// Circuit breaker for individual plugins
pub struct PluginCircuitBreaker {
    plugin_name: String,
    state: Arc<RwLock<CircuitBreakerState>>,
    failure_threshold: usize,
    recovery_timeout: Duration,
    failure_count: AtomicUsize,
    last_failure_time: AtomicU64,
    success_count_in_half_open: AtomicUsize,
}

impl PluginCircuitBreaker {
    pub fn new(plugin_name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            plugin_name,
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            failure_threshold: config.failure_threshold,
            recovery_timeout: config.recovery_timeout,
            failure_count: AtomicUsize::new(0),
            last_failure_time: AtomicU64::new(0),
            success_count_in_half_open: AtomicUsize::new(0),
        }
    }

    pub async fn execute<F, T>(&self, operation: F) -> CircuitBreakerResult<T>
    where
        F: std::future::Future<Output = BackworksResult<T>> + Send,
    {
        let state = self.state.read().await.clone();
        
        match state {
            CircuitBreakerState::Closed => self.execute_closed(operation).await,
            CircuitBreakerState::Open => {
                if self.should_attempt_reset().await {
                    self.transition_to_half_open().await;
                    self.execute_half_open(operation).await
                } else {
                    Err(CircuitBreakerError::Open(self.plugin_name.clone()))
                }
            }
            CircuitBreakerState::HalfOpen => self.execute_half_open(operation).await,
        }
    }

    async fn execute_closed<F, T>(&self, operation: F) -> CircuitBreakerResult<T>
    where
        F: std::future::Future<Output = BackworksResult<T>> + Send,
    {
        match operation.await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(err) => {
                self.record_failure().await;
                Err(CircuitBreakerError::PluginError(err))
            }
        }
    }

    async fn execute_half_open<F, T>(&self, operation: F) -> CircuitBreakerResult<T>
    where
        F: std::future::Future<Output = BackworksResult<T>> + Send,
    {
        match operation.await {
            Ok(result) => {
                let success_count = self.success_count_in_half_open.fetch_add(1, Ordering::Relaxed);
                if success_count >= 2 { // Require multiple successes to close
                    self.transition_to_closed().await;
                }
                Ok(result)
            }
            Err(err) => {
                self.transition_to_open().await;
                Err(CircuitBreakerError::PluginError(err))
            }
        }
    }

    async fn record_failure(&self) {
        let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed);
        self.last_failure_time.store(
            Instant::now().elapsed().as_millis() as u64,
            Ordering::Relaxed,
        );

        if failure_count >= self.failure_threshold {
            self.transition_to_open().await;
        }
    }

    async fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
    }

    async fn should_attempt_reset(&self) -> bool {
        let last_failure = self.last_failure_time.load(Ordering::Relaxed);
        let now = Instant::now().elapsed().as_millis() as u64;
        (now - last_failure) > self.recovery_timeout.as_millis() as u64
    }

    async fn transition_to_open(&self) {
        *self.state.write().await = CircuitBreakerState::Open;
        tracing::warn!("🔴 Circuit breaker OPEN for plugin: {}", self.plugin_name);
    }

    async fn transition_to_half_open(&self) {
        *self.state.write().await = CircuitBreakerState::HalfOpen;
        self.success_count_in_half_open.store(0, Ordering::Relaxed);
        tracing::info!("🟡 Circuit breaker HALF-OPEN for plugin: {}", self.plugin_name);
    }

    async fn transition_to_closed(&self) {
        *self.state.write().await = CircuitBreakerState::Closed;
        self.failure_count.store(0, Ordering::Relaxed);
        tracing::info!("🟢 Circuit breaker CLOSED for plugin: {}", self.plugin_name);
    }

    pub async fn get_state(&self) -> CircuitBreakerState {
        self.state.read().await.clone()
    }

    pub fn get_failure_count(&self) -> usize {
        self.failure_count.load(Ordering::Relaxed)
    }
}

impl Clone for PluginCircuitBreaker {
    fn clone(&self) -> Self {
        Self {
            plugin_name: self.plugin_name.clone(),
            state: self.state.clone(),
            failure_threshold: self.failure_threshold,
            recovery_timeout: self.recovery_timeout,
            failure_count: AtomicUsize::new(self.failure_count.load(std::sync::atomic::Ordering::Relaxed)),
            last_failure_time: AtomicU64::new(self.last_failure_time.load(std::sync::atomic::Ordering::Relaxed)),
            success_count_in_half_open: AtomicUsize::new(self.success_count_in_half_open.load(std::sync::atomic::Ordering::Relaxed)),
        }
    }
}

/// Configuration for circuit breaker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: usize,
    
    #[serde(default = "default_recovery_timeout")]
    pub recovery_timeout: Duration,
    
    #[serde(default = "default_timeout")]
    pub timeout: Duration,
}

fn default_failure_threshold() -> usize { 5 }
fn default_recovery_timeout() -> Duration { Duration::from_secs(30) }
fn default_timeout() -> Duration { Duration::from_millis(100) }

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: default_failure_threshold(),
            recovery_timeout: default_recovery_timeout(),
            timeout: default_timeout(),
        }
    }
}

/// Resource limits for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResourceLimits {
    #[serde(default)]
    pub max_memory_mb: Option<usize>,
    
    #[serde(default)]
    pub max_execution_time: Option<Duration>,
    
    #[serde(default)]
    pub max_concurrent_operations: Option<usize>,
}

impl Default for PluginResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: Some(100), // 100MB default limit
            max_execution_time: Some(Duration::from_millis(100)), // 100ms default
            max_concurrent_operations: Some(10), // 10 concurrent ops
        }
    }
}

/// Plugin performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetrics {
    pub plugin_name: String,
    pub total_invocations: u64,
    pub successful_invocations: u64,
    pub failed_invocations: u64,
    pub average_execution_time_ms: f64,
    pub p95_execution_time_ms: f64,
    pub current_memory_usage_mb: f64,
    pub circuit_breaker_state: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Plugin executor with resilience features
pub struct ResilientPluginExecutor {
    circuit_breakers: Arc<RwLock<HashMap<String, PluginCircuitBreaker>>>,
    resource_limits: Arc<RwLock<HashMap<String, PluginResourceLimits>>>,
    metrics: Arc<RwLock<HashMap<String, PluginMetrics>>>,
}

impl ResilientPluginExecutor {
    pub fn new() -> Self {
        Self {
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            resource_limits: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_plugin(&self, plugin_name: String, config: ResilientPluginConfig) {
        // Register circuit breaker
        let circuit_breaker = PluginCircuitBreaker::new(
            plugin_name.clone(),
            config.circuit_breaker.unwrap_or_default(),
        );
        self.circuit_breakers.write().await.insert(plugin_name.clone(), circuit_breaker);

        // Register resource limits
        self.resource_limits.write().await.insert(
            plugin_name.clone(),
            config.resource_limits.unwrap_or_default(),
        );

        // Initialize metrics
        let metrics = PluginMetrics {
            plugin_name: plugin_name.clone(),
            total_invocations: 0,
            successful_invocations: 0,
            failed_invocations: 0,
            average_execution_time_ms: 0.0,
            p95_execution_time_ms: 0.0,
            current_memory_usage_mb: 0.0,
            circuit_breaker_state: "Closed".to_string(),
            last_updated: chrono::Utc::now(),
        };
        self.metrics.write().await.insert(plugin_name, metrics);
    }

    pub async fn execute_with_resilience<F, T>(
        &self,
        plugin_name: &str,
        operation: F,
    ) -> ResilientExecutionResult<T>
    where
        F: std::future::Future<Output = BackworksResult<T>> + Send,
    {
        let start_time = Instant::now();

        // Get circuit breaker
        let circuit_breaker = {
            let breakers = self.circuit_breakers.read().await;
            breakers.get(plugin_name).cloned()
        };

        let circuit_breaker = match circuit_breaker {
            Some(cb) => cb,
            None => {
                return Err(ResilientExecutionError::PluginNotRegistered(plugin_name.to_string()));
            }
        };

        // Get resource limits
        let limits = {
            let limits_map = self.resource_limits.read().await;
            limits_map.get(plugin_name).cloned().unwrap_or_default()
        };

        // Apply timeout if specified
        let operation_with_timeout = async {
            if let Some(max_time) = limits.max_execution_time {
                timeout(max_time, operation).await
                    .map_err(|_| crate::error::BackworksError::PluginTimeout(plugin_name.to_string()))?
            } else {
                operation.await
            }
        };

        // Execute through circuit breaker
        let result = circuit_breaker.execute(operation_with_timeout).await;

        // Record metrics
        let execution_time = start_time.elapsed();
        self.update_metrics(plugin_name, &result, execution_time).await;

        result.map_err(ResilientExecutionError::CircuitBreakerError)
    }

    async fn update_metrics<T>(
        &self,
        plugin_name: &str,
        result: &CircuitBreakerResult<T>,
        execution_time: Duration,
    ) {
        let mut metrics_map = self.metrics.write().await;
        if let Some(metrics) = metrics_map.get_mut(plugin_name) {
            metrics.total_invocations += 1;
            
            match result {
                Ok(_) => metrics.successful_invocations += 1,
                Err(_) => metrics.failed_invocations += 1,
            }

            // Update average execution time (simple moving average)
            let new_time_ms = execution_time.as_millis() as f64;
            metrics.average_execution_time_ms = 
                (metrics.average_execution_time_ms * (metrics.total_invocations - 1) as f64 + new_time_ms) 
                / metrics.total_invocations as f64;

            metrics.last_updated = chrono::Utc::now();

            // Update circuit breaker state
            if let Some(cb) = self.circuit_breakers.read().await.get(plugin_name) {
                metrics.circuit_breaker_state = match cb.get_state().await {
                    CircuitBreakerState::Closed => "Closed".to_string(),
                    CircuitBreakerState::Open => "Open".to_string(),
                    CircuitBreakerState::HalfOpen => "HalfOpen".to_string(),
                };
            }
        }
    }

    pub async fn get_plugin_metrics(&self, plugin_name: &str) -> Option<PluginMetrics> {
        self.metrics.read().await.get(plugin_name).cloned()
    }

    pub async fn get_all_metrics(&self) -> HashMap<String, PluginMetrics> {
        self.metrics.read().await.clone()
    }
}

/// Configuration for resilient plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilientPluginConfig {
    #[serde(default)]
    pub circuit_breaker: Option<CircuitBreakerConfig>,
    
    #[serde(default)]
    pub resource_limits: Option<PluginResourceLimits>,
    
    #[serde(default)]
    pub is_critical: bool,
}

impl Default for ResilientPluginConfig {
    fn default() -> Self {
        Self {
            circuit_breaker: Some(CircuitBreakerConfig::default()),
            resource_limits: Some(PluginResourceLimits::default()),
            is_critical: false,
        }
    }
}

/// Circuit breaker result type
pub type CircuitBreakerResult<T> = Result<T, CircuitBreakerError>;

/// Circuit breaker errors
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open for plugin: {0}")]
    Open(String),
    
    #[error("Plugin error: {0}")]
    PluginError(#[from] crate::error::BackworksError),
}

/// Resilient execution result type
pub type ResilientExecutionResult<T> = Result<T, ResilientExecutionError>;

/// Resilient execution errors
#[derive(Debug, thiserror::Error)]
pub enum ResilientExecutionError {
    #[error("Plugin not registered: {0}")]
    PluginNotRegistered(String),
    
    #[error("Circuit breaker error: {0}")]
    CircuitBreakerError(CircuitBreakerError),
}

impl From<ResilientExecutionError> for crate::error::BackworksError {
    fn from(err: ResilientExecutionError) -> Self {
        crate::error::BackworksError::plugin(err.to_string())
    }
}
