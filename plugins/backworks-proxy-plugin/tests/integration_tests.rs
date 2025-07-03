use backworks_proxy_plugin::ProxyPlugin;
use backworks::plugin::{Plugin, PluginConfig};
use serde_json::json;

#[tokio::test]
async fn test_proxy_plugin_initialization() {
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
    
    assert_eq!(plugin.name(), "proxy");
    assert_eq!(plugin.version(), "0.1.0");
    assert!(!plugin.description().is_empty());
}

#[tokio::test]
async fn test_proxy_plugin_lifecycle() {
    let mut plugin = ProxyPlugin::new();
    
    let config = PluginConfig {
        enabled: true,
        config: json!({
            "health_checks": true,
            "health_check_interval": 30,
            "timeout": 30,
            "max_connections": 100
        }),
    };
    
    // Initialize
    plugin.initialize(&config).await.unwrap();
    
    // Start
    let result = plugin.start().await;
    assert!(result.is_ok());
    
    // Health check
    let health = plugin.health_check().await;
    assert!(health.is_ok());
    
    // Stop
    let result = plugin.stop().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_proxy_plugin_metadata() {
    let plugin = ProxyPlugin::new();
    let metadata = plugin.metadata();
    
    assert_eq!(metadata.get("type"), Some(&"proxy".to_string()));
    assert_eq!(metadata.get("version"), Some(&"0.1.0".to_string()));
    assert!(metadata.contains_key("health_checks"));
}

#[tokio::test]
async fn test_proxy_plugin_config_validation() {
    let mut plugin = ProxyPlugin::new();
    
    // Test with valid config
    let valid_config = PluginConfig {
        enabled: true,
        config: json!({
            "health_checks": true,
            "timeout": 30
        }),
    };
    
    let result = plugin.initialize(&valid_config).await;
    assert!(result.is_ok());
    
    // Test metadata reflects config
    let metadata = plugin.metadata();
    assert_eq!(metadata.get("health_checks"), Some(&"true".to_string()));
}

#[tokio::test]
async fn test_proxy_plugin_default_config() {
    let mut plugin = ProxyPlugin::new();
    
    // Use empty config to test defaults
    let config = PluginConfig {
        enabled: true,
        config: json!({}),
    };
    
    let result = plugin.initialize(&config).await;
    assert!(result.is_ok());
    
    // Should work with default configuration
    let health = plugin.health_check().await;
    assert!(health.is_ok());
}
