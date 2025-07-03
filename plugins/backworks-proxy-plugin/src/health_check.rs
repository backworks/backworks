//! Health checking system for proxy targets

use crate::error::ProxyResult;
use crate::load_balancer::ProxyTarget;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Whether health checks are enabled
    pub enabled: bool,
    
    /// Interval between health checks
    pub interval: Duration,
    
    /// Timeout for health check requests
    pub timeout: Duration,
    
    /// Number of consecutive successes to mark as healthy
    pub healthy_threshold: u32,
    
    /// Number of consecutive failures to mark as unhealthy
    pub unhealthy_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            healthy_threshold: 3,
            unhealthy_threshold: 2,
        }
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Target name
    pub target_name: String,
    
    /// Whether the target is healthy
    pub healthy: bool,
    
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
    
    /// HTTP status code (if applicable)
    pub status_code: Option<u16>,
    
    /// Error message (if failed)
    pub error: Option<String>,
    
    /// Timestamp of the check
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Health check statistics for a target
#[derive(Debug, Clone)]
struct TargetHealthStats {
    /// Consecutive successful checks
    consecutive_successes: u32,
    
    /// Consecutive failed checks
    consecutive_failures: u32,
    
    /// Current health status
    healthy: bool,
    
    /// Last check time
    last_check: Option<Instant>,
    
    /// Recent check results (limited history)
    recent_results: Vec<HealthCheckResult>,
}

impl TargetHealthStats {
    fn new() -> Self {
        Self {
            consecutive_successes: 0,
            consecutive_failures: 0,
            healthy: true, // Assume healthy initially
            last_check: None,
            recent_results: Vec::new(),
        }
    }

    fn record_success(&mut self, result: HealthCheckResult) {
        self.consecutive_successes += 1;
        self.consecutive_failures = 0;
        self.last_check = Some(Instant::now());
        
        // Keep only recent results (last 10)
        self.recent_results.push(result);
        if self.recent_results.len() > 10 {
            self.recent_results.remove(0);
        }
    }

    fn record_failure(&mut self, result: HealthCheckResult) {
        self.consecutive_failures += 1;
        self.consecutive_successes = 0;
        self.last_check = Some(Instant::now());
        
        // Keep only recent results (last 10)
        self.recent_results.push(result);
        if self.recent_results.len() > 10 {
            self.recent_results.remove(0);
        }
    }

    fn should_mark_healthy(&self, threshold: u32) -> bool {
        !self.healthy && self.consecutive_successes >= threshold
    }

    fn should_mark_unhealthy(&self, threshold: u32) -> bool {
        self.healthy && self.consecutive_failures >= threshold
    }
}

/// Health checker implementation
pub struct HealthChecker {
    config: HealthCheckConfig,
    client: reqwest::Client,
    target_stats: Arc<RwLock<HashMap<String, TargetHealthStats>>>,
    health_change_callback: Option<Arc<dyn Fn(&str, bool) + Send + Sync>>,
}

impl HealthChecker {
    pub fn new(config: HealthCheckConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap();

        Self {
            config,
            client,
            target_stats: Arc::new(RwLock::new(HashMap::new())),
            health_change_callback: None,
        }
    }

    /// Set callback for health status changes
    pub fn set_health_change_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str, bool) + Send + Sync + 'static,
    {
        self.health_change_callback = Some(Arc::new(callback));
    }

    /// Add a target for health checking
    pub async fn add_target(&self, target: &ProxyTarget) -> ProxyResult<()> {
        let mut stats = self.target_stats.write().await;
        stats.insert(target.name.clone(), TargetHealthStats::new());
        Ok(())
    }

    /// Remove a target from health checking
    pub async fn remove_target(&self, target_name: &str) -> ProxyResult<()> {
        let mut stats = self.target_stats.write().await;
        stats.remove(target_name);
        Ok(())
    }

    /// Start health checking for all targets
    pub async fn start_health_checking(&self, targets: Vec<ProxyTarget>) -> ProxyResult<()> {
        if !self.config.enabled {
            tracing::info!("Health checking disabled");
            return Ok(());
        }

        // Add all targets
        for target in &targets {
            self.add_target(target).await?;
        }

        // Start background health checking
        let checker = self.clone();
        let targets_clone = targets.clone();
        
        tokio::spawn(async move {
            checker.health_check_loop(targets_clone).await;
        });

        tracing::info!("Started health checking for {} targets", targets.len());
        Ok(())
    }

    /// Perform health check for a single target
    pub async fn check_target_health(&self, target: &ProxyTarget) -> HealthCheckResult {
        let start_time = Instant::now();
        let timestamp = chrono::Utc::now();

        // Build health check URL (assume /health endpoint if not specified)
        let health_url = if target.url.ends_with('/') {
            format!("{}health", target.url)
        } else {
            format!("{}/health", target.url)
        };

        match self.client.get(&health_url).send().await {
            Ok(response) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                let status_code = response.status().as_u16();
                
                let healthy = response.status().is_success();
                
                HealthCheckResult {
                    target_name: target.name.clone(),
                    healthy,
                    response_time_ms: Some(response_time),
                    status_code: Some(status_code),
                    error: if healthy { None } else { Some(format!("HTTP {}", status_code)) },
                    timestamp,
                }
            }
            Err(error) => {
                HealthCheckResult {
                    target_name: target.name.clone(),
                    healthy: false,
                    response_time_ms: None,
                    status_code: None,
                    error: Some(error.to_string()),
                    timestamp,
                }
            }
        }
    }

    /// Stop health checking
    pub async fn stop(&mut self) -> ProxyResult<()> {
        // Clear all target stats
        let mut stats = self.target_stats.write().await;
        stats.clear();
        
        tracing::info!("Health checker stopped");
        Ok(())
    }

    /// Check if the health checker is running
    pub async fn is_running(&self) -> bool {
        let stats = self.target_stats.read().await;
        !stats.is_empty()
    }

    /// Get health status for a target
    pub async fn get_target_health(&self, target_name: &str) -> Option<bool> {
        let stats = self.target_stats.read().await;
        stats.get(target_name).map(|s| s.healthy)
    }

    /// Get health statistics for all targets
    pub async fn get_all_health_stats(&self) -> HashMap<String, bool> {
        let stats = self.target_stats.read().await;
        stats.iter()
            .map(|(name, stat)| (name.clone(), stat.healthy))
            .collect()
    }

    /// Get recent health check results for a target
    pub async fn get_recent_results(&self, target_name: &str) -> Option<Vec<HealthCheckResult>> {
        let stats = self.target_stats.read().await;
        stats.get(target_name).map(|s| s.recent_results.clone())
    }

    /// Health check loop
    async fn health_check_loop(&self, targets: Vec<ProxyTarget>) {
        let mut interval = interval(self.config.interval);
        
        loop {
            interval.tick().await;
            
            // Check all targets concurrently
            let checks: Vec<_> = targets.iter()
                .map(|target| self.check_target_health(target))
                .collect();
            
            let results = futures::future::join_all(checks).await;
            
            // Process results
            for result in results {
                if let Err(e) = self.process_health_result(result).await {
                    tracing::error!("Error processing health check result: {}", e);
                }
            }
        }
    }

    /// Process a health check result and update target stats
    async fn process_health_result(&self, result: HealthCheckResult) -> ProxyResult<()> {
        let mut stats = self.target_stats.write().await;
        
        if let Some(target_stats) = stats.get_mut(&result.target_name) {
            let previous_health = target_stats.healthy;
            
            if result.healthy {
                target_stats.record_success(result.clone());
                
                // Check if we should mark as healthy
                if target_stats.should_mark_healthy(self.config.healthy_threshold) {
                    target_stats.healthy = true;
                    
                    if !previous_health {
                        tracing::info!("Target {} marked as healthy", result.target_name);
                        
                        // Notify callback
                        if let Some(ref callback) = self.health_change_callback {
                            callback(&result.target_name, true);
                        }
                    }
                }
            } else {
                target_stats.record_failure(result.clone());
                
                // Check if we should mark as unhealthy
                if target_stats.should_mark_unhealthy(self.config.unhealthy_threshold) {
                    target_stats.healthy = false;
                    
                    if previous_health {
                        tracing::warn!("Target {} marked as unhealthy: {}", 
                            result.target_name, 
                            result.error.unwrap_or_else(|| "Unknown error".to_string())
                        );
                        
                        // Notify callback
                        if let Some(ref callback) = self.health_change_callback {
                            callback(&result.target_name, false);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl Clone for HealthChecker {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: self.client.clone(),
            target_stats: Arc::clone(&self.target_stats),
            health_change_callback: self.health_change_callback.clone(),
        }
    }
}

impl std::fmt::Debug for HealthChecker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HealthChecker")
            .field("client", &"HttpClient")
            .field("config", &self.config)
            .field("target_stats", &self.target_stats)
            .field("health_change_callback", &"Option<Callback>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_creation() {
        let config = HealthCheckConfig::default();
        let checker = HealthChecker::new(config);
        
        let target = ProxyTarget::new("test".to_string(), "http://localhost:8080".to_string());
        
        assert!(checker.add_target(&target).await.is_ok());
        assert_eq!(checker.get_target_health("test").await, Some(true)); // Initially healthy
    }

    #[tokio::test]
    async fn test_health_stats_success_tracking() {
        let mut stats = TargetHealthStats::new();
        
        let result = HealthCheckResult {
            target_name: "test".to_string(),
            healthy: true,
            response_time_ms: Some(100),
            status_code: Some(200),
            error: None,
            timestamp: chrono::Utc::now(),
        };
        
        stats.record_success(result);
        
        assert_eq!(stats.consecutive_successes, 1);
        assert_eq!(stats.consecutive_failures, 0);
        assert_eq!(stats.recent_results.len(), 1);
    }

    #[tokio::test]
    async fn test_health_stats_failure_tracking() {
        let mut stats = TargetHealthStats::new();
        
        let result = HealthCheckResult {
            target_name: "test".to_string(),
            healthy: false,
            response_time_ms: None,
            status_code: Some(500),
            error: Some("Internal Server Error".to_string()),
            timestamp: chrono::Utc::now(),
        };
        
        stats.record_failure(result);
        
        assert_eq!(stats.consecutive_failures, 1);
        assert_eq!(stats.consecutive_successes, 0);
        assert_eq!(stats.recent_results.len(), 1);
    }

    #[tokio::test]
    async fn test_health_threshold_logic() {
        let mut stats = TargetHealthStats::new();
        stats.healthy = false; // Start unhealthy
        
        // Record 3 successes
        for _ in 0..3 {
            let result = HealthCheckResult {
                target_name: "test".to_string(),
                healthy: true,
                response_time_ms: Some(100),
                status_code: Some(200),
                error: None,
                timestamp: chrono::Utc::now(),
            };
            stats.record_success(result);
        }
        
        // Should mark as healthy with threshold 3
        assert!(stats.should_mark_healthy(3));
        assert!(!stats.should_mark_healthy(4));
    }
}
