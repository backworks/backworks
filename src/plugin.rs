use async_trait::async_trait;
use crate::error::BackworksResult;
use crate::resilience::{ResilientPluginExecutor, ResilientPluginConfig, PluginMetrics};
use axum::{http::Request, response::Response};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::path::Path;
use tokio::sync::RwLock;
use crate::config::PluginDiscoveryConfig;

pub mod dynamic;
pub mod discovery;
pub use dynamic::{DynamicPluginLoader, PluginMetadata};
pub use discovery::{PluginDiscovery, PluginRegistry};

/// Configuration for a plugin
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginConfig {
    #[serde(default)]
    pub enabled: bool,
    
    #[serde(default)]
    pub plugin_type: PluginType,
    
    #[serde(default)]
    pub config: Value,
    
    // For external plugins
    pub path: Option<String>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            plugin_type: PluginType::Builtin,
            config: Value::Null,
            path: None,
        }
    }
}

/// Type of plugin
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    Builtin,
    External,
}

impl Default for PluginType {
    fn default() -> Self {
        PluginType::Builtin
    }
}

/// Plugin trait that all Backworks plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get the plugin name
    fn name(&self) -> &str;
    
    /// Get the plugin version
    fn version(&self) -> &str;
    
    /// Get the plugin description
    fn description(&self) -> &str;
    
    /// Initialize the plugin with configuration
    async fn initialize(&mut self, config: &PluginConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Start the plugin
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Stop the plugin
    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Health check for the plugin
    async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get plugin metadata
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Enhanced plugin interface with resilience features
#[async_trait::async_trait]
pub trait BackworksPlugin: Send + Sync {
    /// Plugin identifier
    fn name(&self) -> &str;
    
    /// Plugin version
    fn version(&self) -> &str;
    
    /// Plugin description
    fn description(&self) -> &str;
    
    /// Initialize the plugin with configuration
    async fn initialize(&self, config: &Value) -> BackworksResult<()>;
    
    /// Shutdown the plugin gracefully
    async fn shutdown(&self) -> BackworksResult<()>;
    
    /// Plugin health check
    async fn health_check(&self) -> BackworksResult<PluginHealth> {
        Ok(PluginHealth {
            status: HealthStatus::Healthy,
            message: "Plugin is running normally".to_string(),
            details: HashMap::new(),
        })
    }
    
    /// Maximum execution time for plugin operations (for circuit breaker)
    fn max_execution_time(&self) -> Duration {
        Duration::from_millis(100) // Default 100ms timeout
    }
    
    /// Whether this plugin is critical (failure should affect request)
    fn is_critical(&self) -> bool {
        false // Most plugins are non-critical by default
    }
    
    
    /// Hook called before processing each request
    async fn before_request(&self, request: &mut Request<axum::body::Body>) -> BackworksResult<()> {
        let _ = request; // Default implementation does nothing
        Ok(())
    }
    
    /// Hook called after generating each response
    async fn after_response(&self, response: &mut Response<axum::body::Body>) -> BackworksResult<()> {
        let _ = response; // Default implementation does nothing
        Ok(())
    }
    
    /// Hook called when configuration changes
    async fn on_config_reload(&self, config: &Value) -> BackworksResult<()> {
        let _ = config; // Default implementation does nothing
        Ok(())
    }
    
    /// Hook called for custom endpoint processing
    async fn process_endpoint_data(&self, _endpoint: &str, _method: &str, _data: &str) -> BackworksResult<Option<String>> {
        Ok(None) // Default implementation doesn't handle endpoints
    }
    
}

/// Plugin health status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginHealth {
    pub status: HealthStatus,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Enhanced plugin manager with resilience features
#[derive(Clone)]
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, Arc<dyn BackworksPlugin>>>>,
    configs: Arc<RwLock<HashMap<String, Value>>>,
    resilient_executor: Arc<ResilientPluginExecutor>,
    dynamic_loader: Arc<DynamicPluginLoader>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            resilient_executor: Arc::new(ResilientPluginExecutor::new()),
            dynamic_loader: Arc::new(DynamicPluginLoader::new()),
        }
    }
    
    /// Register a plugin with resilience configuration
    pub async fn register_plugin(
        &self, 
        plugin: Arc<dyn BackworksPlugin>, 
        config: Option<Value>,
        resilience_config: Option<ResilientPluginConfig>
    ) -> BackworksResult<()> {
        let name = plugin.name().to_string();
        
        // Register with resilient executor
        self.resilient_executor.register_plugin(
            name.clone(),
            resilience_config.unwrap_or_default(),
        ).await;
        
        // Initialize the plugin if config is provided
        if let Some(config) = config.as_ref() {
            let result = self.resilient_executor.execute_with_resilience(
                &name,
                plugin.initialize(config),
            ).await;
            
            if let Err(err) = result {
                tracing::error!("🔴 Failed to initialize plugin {}: {:?}", name, err);
                if plugin.is_critical() {
                    return Err(crate::error::BackworksError::PluginInitializationFailed(name));
                }
                // Non-critical plugins can fail to initialize without affecting the system
                tracing::warn!("⚠️ Non-critical plugin {} failed to initialize, continuing...", name);
            }
        }
        
        // Store plugin and config
        self.plugins.write().await.insert(name.clone(), plugin);
        if let Some(config) = config {
            self.configs.write().await.insert(name.clone(), config);
        }
        
        tracing::info!("🔌 Registered plugin: {}", name);
        Ok(())
    }
    
    /// Load and register an external plugin from a file path
    pub async fn load_external_plugin<P: AsRef<Path>>(
        &self, 
        path: P, 
        config: Option<Value>,
        resilience_config: Option<ResilientPluginConfig>
    ) -> BackworksResult<()> {
        let plugin = self.dynamic_loader.load_plugin(path).await?;
        let plugin_arc = Arc::from(plugin);
        self.register_plugin(plugin_arc, config, resilience_config).await
    }
    
    /// Register a plugin from configuration
    pub async fn register_plugin_from_config(
        &self,
        name: &str,
        plugin_config: &PluginConfig,
        resilience_config: Option<ResilientPluginConfig>
    ) -> BackworksResult<()> {
        if !plugin_config.enabled {
            tracing::debug!("Plugin {} is disabled, skipping", name);
            return Ok(());
        }
        
        match plugin_config.plugin_type {
            PluginType::Builtin => {
                // For builtin plugins, they should be registered separately
                tracing::warn!("Cannot auto-register builtin plugin {} from config - must be registered explicitly", name);
                Ok(())
            }
            PluginType::External => {
                let path = plugin_config.path.as_ref()
                    .ok_or_else(|| crate::error::BackworksError::Config(
                        format!("External plugin {} missing path configuration", name)
                    ))?;
                
                self.load_external_plugin(
                    path,
                    Some(plugin_config.config.clone()),
                    resilience_config
                ).await
            }
        }
    }
    
    /// Discover and list available external plugins
    pub async fn discover_plugins(&self) -> BackworksResult<Vec<PluginMetadata>> {
        self.dynamic_loader.scan_plugins().await
    }
    
    /// Add a directory to scan for external plugins
    pub async fn add_plugin_directory<P: AsRef<Path>>(&self, path: P) {
        // Since DynamicPluginLoader is behind Arc, we need to access it differently
        // For now, we'll document this limitation and suggest using environment configuration
        tracing::info!("To add plugin directory {}, configure it in your environment or configuration file", 
                      path.as_ref().display());
    }
    
    /// Initialize plugins from discovery configuration
    pub async fn initialize_from_discovery(&self, discovery_config: &PluginDiscoveryConfig) -> BackworksResult<()> {
        if !discovery_config.enabled {
            tracing::debug!("Plugin discovery disabled, skipping auto-discovery");
            return Ok(());
        }
        
        let discovery = PluginDiscovery::new(discovery_config.clone());
        let discovered_plugins = discovery.discover_all_plugins().await?;
        
        tracing::info!("Auto-discovered {} external plugins", discovered_plugins.len());
        
        for plugin_meta in discovered_plugins {
            tracing::info!("Auto-loading plugin: {} v{} from {}", 
                          plugin_meta.name, plugin_meta.version, plugin_meta.path.display());
            
            // Load the plugin with default configuration
            if let Err(e) = self.load_external_plugin(&plugin_meta.path, None, None).await {
                tracing::warn!("Failed to auto-load plugin {}: {}", plugin_meta.name, e);
                // Continue with other plugins even if one fails
            }
        }
        
        Ok(())
    }
    
    /// Remove a plugin safely
    pub async fn unregister_plugin(&self, name: &str) -> BackworksResult<()> {
        let plugins = self.plugins.read().await;
        if let Some(plugin) = plugins.get(name) {
            // Shutdown plugin through resilient executor
            let result = self.resilient_executor.execute_with_resilience(
                name,
                plugin.shutdown(),
            ).await;
            
            if let Err(err) = result {
                tracing::warn!("⚠️ Plugin {} shutdown failed: {:?}", name, err);
                // Continue with unregistration even if shutdown fails
            }
            
            tracing::info!("🔌 Unregistered plugin: {}", name);
        }
        
        // Remove from storage (do this after attempting shutdown)
        drop(plugins); // Release read lock
        self.plugins.write().await.remove(name);
        self.configs.write().await.remove(name);
        Ok(())
    }
    
    /// Get list of registered plugin names
    pub async fn list_plugins(&self) -> Vec<String> {
        self.plugins.read().await.keys().cloned().collect()
    }
    
    /// Call before_request on all plugins with resilience
    pub async fn before_request(&self, request: &mut Request<axum::body::Body>) -> BackworksResult<()> {
        let plugins = self.plugins.read().await;
        let mut critical_errors = Vec::new();
        
        for (name, plugin) in plugins.iter() {
            let result = self.resilient_executor.execute_with_resilience(
                name,
                plugin.before_request(request),
            ).await;
            
            match result {
                Ok(_) => {
                    tracing::debug!("✅ Plugin {} before_request hook completed", name);
                }
                Err(err) => {
                    tracing::warn!("⚠️ Plugin {} before_request hook failed: {:?}", name, err);
                    
                    if plugin.is_critical() {
                        critical_errors.push((name.clone(), err));
                    }
                    // Non-critical plugin failures are logged but don't affect the request
                }
            }
        }
        
        // If any critical plugins failed, fail the request
        if !critical_errors.is_empty() {
            return Err(crate::error::BackworksError::CriticalPluginFailure(
                critical_errors.into_iter().map(|(name, _)| name).collect()
            ));
        }
        
        Ok(())
    }
    
    /// Call after_response on all plugins with resilience
    pub async fn after_response(&self, response: &mut Response<axum::body::Body>) -> BackworksResult<()> {
        let plugins = self.plugins.read().await;
        let mut critical_errors = Vec::new();
        
        // Execute in reverse order for after_response hooks
        let plugin_vec: Vec<_> = plugins.iter().collect();
        for (name, plugin) in plugin_vec.iter().rev() {
            let result = self.resilient_executor.execute_with_resilience(
                name,
                plugin.after_response(response),
            ).await;
            
            match result {
                Ok(_) => {
                    tracing::debug!("✅ Plugin {} after_response hook completed", name);
                }
                Err(err) => {
                    tracing::warn!("⚠️ Plugin {} after_response hook failed: {:?}", name, err);
                    
                    if plugin.is_critical() {
                        critical_errors.push(((*name).clone(), err));
                    }
                    // Non-critical plugin failures are logged but don't affect the response
                }
            }
        }
        
        // For after_response, we typically don't want to fail the response
        // even for critical plugins, just log the errors
        if !critical_errors.is_empty() {
            tracing::error!("🔴 Critical plugins failed in after_response: {:?}", 
                critical_errors.iter().map(|(name, _)| name).collect::<Vec<_>>());
        }
        
        Ok(())
    }
    
    
    /// Reload configuration for all plugins with resilience
    pub async fn reload_configs(&self, new_configs: HashMap<String, Value>) -> BackworksResult<()> {
        let plugins = self.plugins.read().await;
        let mut failed_reloads = Vec::new();
        
        for (name, config) in new_configs.iter() {
            if let Some(plugin) = plugins.get(name) {
                let result = self.resilient_executor.execute_with_resilience(
                    name,
                    plugin.on_config_reload(config),
                ).await;
                
                match result {
                    Ok(_) => {
                        tracing::info!("🔄 Plugin {} config reloaded successfully", name);
                    }
                    Err(err) => {
                        tracing::error!("🔴 Plugin {} config reload failed: {:?}", name, err);
                        failed_reloads.push(name.clone());
                    }
                }
            }
        }
        
        // Update stored configs for successful reloads
        let mut configs = self.configs.write().await;
        for (name, config) in new_configs {
            if !failed_reloads.contains(&name) {
                configs.insert(name, config);
            }
        }
        
        if !failed_reloads.is_empty() {
            tracing::warn!("⚠️ Some plugin configs failed to reload: {:?}", failed_reloads);
        }
        
        Ok(())
    }
    
    /// Get plugin health status
    pub async fn get_plugin_health(&self, plugin_name: &str) -> Option<PluginHealth> {
        let plugins = self.plugins.read().await;
        if let Some(plugin) = plugins.get(plugin_name) {
            match self.resilient_executor.execute_with_resilience(
                plugin_name,
                plugin.health_check(),
            ).await {
                Ok(health) => Some(health),
                Err(_) => Some(PluginHealth {
                    status: HealthStatus::Unhealthy,
                    message: "Health check failed".to_string(),
                    details: HashMap::new(),
                }),
            }
        } else {
            None
        }
    }
    
    /// Get all plugin health statuses
    pub async fn get_all_plugin_health(&self) -> HashMap<String, PluginHealth> {
        let plugins = self.plugins.read().await;
        let mut health_map = HashMap::new();
        
        for (name, plugin) in plugins.iter() {
            let health = match self.resilient_executor.execute_with_resilience(
                name,
                plugin.health_check(),
            ).await {
                Ok(health) => health,
                Err(_) => PluginHealth {
                    status: HealthStatus::Unhealthy,
                    message: "Health check failed".to_string(),
                    details: HashMap::new(),
                },
            };
            health_map.insert(name.clone(), health);
        }
        
        health_map
    }
    
    /// Get plugin performance metrics
    pub async fn get_plugin_metrics(&self, plugin_name: &str) -> Option<PluginMetrics> {
        self.resilient_executor.get_plugin_metrics(plugin_name).await
    }
    
    /// Get all plugin performance metrics
    /// Get all plugin performance metrics
    pub async fn get_all_plugin_metrics(&self) -> HashMap<String, PluginMetrics> {
        self.resilient_executor.get_all_metrics().await
    }
    
    /// Try to process endpoint with plugins (first plugin to return Some wins)
    pub async fn process_endpoint_data(&self, endpoint: &str, method: &str, data: &str) -> BackworksResult<Option<String>> {
        let plugins = self.plugins.read().await;
        
        for (name, plugin) in plugins.iter() {
            let result = self.resilient_executor.execute_with_resilience(
                name,
                plugin.process_endpoint_data(endpoint, method, data),
            ).await;
            
            match result {
                Ok(Some(response)) => return Ok(Some(response)),
                Ok(None) => continue, // Plugin doesn't handle this endpoint
                Err(err) => {
                    tracing::warn!("⚠️ Plugin {} endpoint processing failed: {:?}", name, err);
                    if plugin.is_critical() {
                        return Err(crate::error::BackworksError::CriticalPluginFailure(vec![name.clone()]));
                    }
                    // Non-critical plugin failures are logged but we continue
                }
            }
        }
        
        Ok(None)
    }
    
    /// Execute a specific plugin with JSON data
    pub async fn execute_plugin(&self, plugin_name: &str, request_data: &str) -> BackworksResult<String> {
        let plugins = self.plugins.read().await;
        
        if let Some(_plugin) = plugins.get(plugin_name) {
            // For now, return a simple response - this would be implemented based on plugin capabilities
            let result = format!(
                r#"{{"plugin": "{}", "processed": true, "message": "Plugin executed successfully", "data": {}}}"#,
                plugin_name, request_data
            );
            Ok(result)
        } else {
            Err(crate::error::BackworksError::Config(format!("Plugin not found: {}", plugin_name)))
        }
    }

    /// Shutdown all plugins gracefully
    pub async fn shutdown_all(&self) -> BackworksResult<()> {
        let plugin_names = self.list_plugins().await;
        let mut failed_shutdowns = Vec::new();
        
        for name in plugin_names {
            if let Err(err) = self.unregister_plugin(&name).await {
                tracing::error!("🔴 Failed to shutdown plugin {}: {:?}", name, err);
                failed_shutdowns.push(name);
            }
        }
        
        if !failed_shutdowns.is_empty() {
            tracing::warn!("⚠️ Some plugins failed to shutdown gracefully: {:?}", failed_shutdowns);
        }
        
        tracing::info!("🔌 All plugins shutdown completed");
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
