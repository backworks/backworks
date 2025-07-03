//! # Backworks Proxy Plugin
//! 
//! HTTP proxy and load balancing plugin for Backworks.
//! 
//! This plugin provides comprehensive proxy functionality including:
//! - Multiple load balancing algorithms (Round Robin, Weighted, IP Hash, Least Connections)
//! - Circuit breaker patterns for fault tolerance
//! - Health checking and automatic failover
//! - Request/response transformations
//! - Metrics collection and monitoring
//! - Capture integration for debugging

pub mod plugin;
pub mod proxy;
pub mod load_balancer;
pub mod circuit_breaker;
pub mod health_check;
pub mod transformations;
pub mod metrics;
pub mod error;

// Re-export main types
pub use plugin::{ProxyPlugin, ProxyPluginConfig, MetricsConfig};
pub use proxy::ProxyManager;
pub use load_balancer::{LoadBalancer, LoadBalancingAlgorithm};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
pub use health_check::{HealthChecker, HealthCheckConfig};
pub use transformations::{RequestTransformer, ResponseTransformer};
pub use metrics::ProxyMetrics;
pub use error::{ProxyError, ProxyResult};

#[cfg(test)]
mod tests {
    use super::*;
    use backworks::plugin::{Plugin, PluginConfig};
    use serde_json::json;

    #[tokio::test]
    async fn test_plugin_name_and_version() {
        let plugin = ProxyPlugin::new();
        assert_eq!(plugin.name(), "proxy");
        assert_eq!(plugin.version(), env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_plugin_initialization() {
        let mut plugin = ProxyPlugin::new();
        
        let config = PluginConfig {
            enabled: true,
            config: json!({
                "health_checks": true,
                "health_check_interval": 30,
                "timeout": 30,
                "max_connections": 100,
                "metrics": {
                    "prometheus": true,
                    "endpoint": "/metrics",
                    "interval": 10
                }
            }),
        };

        let result = plugin.initialize(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_plugin_lifecycle() {
        let mut plugin = ProxyPlugin::new();
        
        let config = PluginConfig {
            enabled: true,
            config: json!({
                "health_checks": true,
                "timeout": 30
            }),
        };
        
        // Initialize
        let result = plugin.initialize(&config).await;
        assert!(result.is_ok());
        
        // Start
        let result = plugin.start().await;
        assert!(result.is_ok());
        
        // Stop
        let result = plugin.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let plugin = ProxyPlugin::new();
        let health = plugin.health_check().await;
        assert!(health.is_ok());
    }
}
