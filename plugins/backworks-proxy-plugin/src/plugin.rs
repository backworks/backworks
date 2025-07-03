//! Plugin implementation for the Backworks proxy plugin

use crate::proxy::ProxyManager;
use backworks::plugin::{Plugin, PluginConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use async_trait::async_trait;

/// Configuration for the proxy plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyPluginConfig {
    /// Enable health checking
    pub health_checks: Option<bool>,
    
    /// Health check interval in seconds
    pub health_check_interval: Option<u64>,
    
    /// Connection timeout in seconds
    pub timeout: Option<u64>,
    
    /// Maximum connections per target
    pub max_connections: Option<u32>,
    
    /// Enable metrics collection
    pub metrics: Option<MetricsConfig>,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable Prometheus metrics
    pub prometheus: Option<bool>,
    
    /// Metrics endpoint path
    pub endpoint: Option<String>,
    
    /// Collection interval in seconds
    pub interval: Option<u64>,
}

impl Default for ProxyPluginConfig {
    fn default() -> Self {
        Self {
            health_checks: Some(true),
            health_check_interval: Some(30),
            timeout: Some(30),
            max_connections: Some(100),
            metrics: Some(MetricsConfig {
                prometheus: Some(true),
                endpoint: Some("/metrics".to_string()),
                interval: Some(10),
            }),
        }
    }
}

/// The main proxy plugin implementation
pub struct ProxyPlugin {
    config: ProxyPluginConfig,
    proxy_manager: Option<ProxyManager>,
}

impl ProxyPlugin {
    pub fn new() -> Self {
        Self {
            config: ProxyPluginConfig::default(),
            proxy_manager: None,
        }
    }
    
    pub fn with_config(config: ProxyPluginConfig) -> Self {
        Self {
            config,
            proxy_manager: None,
        }
    }
}

impl Default for ProxyPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for ProxyPlugin {
    fn name(&self) -> &str {
        "proxy"
    }
    
    fn version(&self) -> &str {
        "0.1.0"
    }
    
    fn description(&self) -> &str {
        "HTTP proxy and load balancing plugin for Backworks"
    }
    
    async fn initialize(&mut self, config: &PluginConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Parse the plugin configuration from the provided config
        if let Ok(plugin_config) = serde_json::from_value::<ProxyPluginConfig>(config.config.clone()) {
            self.config = plugin_config;
        }
        
        // Create a default proxy configuration for initialization
        let proxy_config = crate::proxy::ProxyConfig {
            targets: Vec::new(), // Will be configured later
            load_balancing: crate::load_balancer::LoadBalancingAlgorithm::RoundRobin,
            health_checks: Some(crate::health_check::HealthCheckConfig {
                enabled: self.config.health_checks.unwrap_or(true),
                interval: Duration::from_secs(self.config.health_check_interval.unwrap_or(30)),
                timeout: Duration::from_secs(10),
                healthy_threshold: 2,
                unhealthy_threshold: 3,
            }),
            circuit_breaker: None, // Can be configured later
            request_transform: None,
            response_transform: None,
            headers: None,
            timeout: Some(Duration::from_secs(self.config.timeout.unwrap_or(30))),
        };
        
        // Initialize the proxy manager with configuration
        let proxy_manager = ProxyManager::new(proxy_config).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        self.proxy_manager = Some(proxy_manager);
        
        tracing::info!("Proxy plugin initialized successfully");
        Ok(())
    }
    
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref mut manager) = self.proxy_manager {
            // Start health checking if enabled
            if self.config.health_checks.unwrap_or(false) {
                manager.start_health_checking().await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            }
            
            tracing::info!("Proxy plugin started successfully");
        }
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref mut manager) = self.proxy_manager {
            manager.stop().await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        }
        
        tracing::info!("Proxy plugin stopped successfully");
        Ok(())
    }
    
    async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref manager) = self.proxy_manager {
            Ok(manager.is_healthy().await)
        } else {
            Ok(false)
        }
    }
    
    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "proxy".to_string());
        metadata.insert("version".to_string(), self.version().to_string());
        metadata.insert("health_checks".to_string(), 
                        self.config.health_checks.unwrap_or(false).to_string());
        metadata
    }
}
