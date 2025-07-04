use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

use crate::config::BackworksConfig;
use crate::server::BackworksServer;
use crate::dashboard::Dashboard;
use crate::runtime::RuntimeManager;
use crate::plugin::PluginManager;
use crate::error::Result;

pub struct BackworksEngine {
    config: Arc<BackworksConfig>,
    server: BackworksServer,
    dashboard: Option<Arc<Dashboard>>,
    #[allow(dead_code)] // TODO: Will be used when runtime features are implemented
    runtime_manager: RuntimeManager,
    plugin_manager: PluginManager,
}

impl BackworksEngine {
    pub async fn new(config: BackworksConfig) -> Result<Self> {
        let config = Arc::new(config);
        
        info!("🎯 Initializing Backworks Engine");
        info!("   Name: {}", config.name);
        info!("   Mode: {:?}", config.mode);
        info!("   Endpoints: {}", config.endpoints.len());
        
        // Initialize plugin manager
        let plugin_manager = PluginManager::new();
        
        // Initialize plugins from configuration
        info!("🔌 Initializing plugins from configuration...");
        
        // Load external plugins from discovery configuration
        if let Err(e) = plugin_manager.initialize_from_discovery(&config.plugin_discovery).await {
            error!("Failed to initialize plugins from discovery: {}", e);
        }
        
        // Load plugins specified in configuration
        for (plugin_name, plugin_config) in &config.plugins {
            if plugin_config.enabled {
                info!("🔌 Loading plugin: {}", plugin_name);
                if let Err(e) = plugin_manager.register_plugin_from_config(plugin_name, plugin_config, None).await {
                    error!("Failed to load plugin {}: {}", plugin_name, e);
                }
            }
        }
        
        info!("🔌 Plugin initialization completed");
        
        // Initialize runtime manager
        info!("⚡ Initializing runtime manager...");
        let runtime_config = crate::runtime::RuntimeManagerConfig::default(); // Create empty config for now
        let runtime_manager = RuntimeManager::new(runtime_config);
        
        // Initialize dashboard if enabled
        let dashboard = if let Some(ref dashboard_config) = &config.dashboard {
            if dashboard_config.enabled {
                info!("🎨 Initializing dashboard on port {}...", dashboard_config.port);
                Some(Arc::new(Dashboard::new(dashboard_config.clone())))
            } else {
                None
            }
        } else {
            None
        };
        
        // Initialize main server
        info!("🚀 Initializing API server on {}:{}...", config.server.host, config.server.port);
        let server = BackworksServer::new(
            config.clone(),
            plugin_manager.clone(),
            dashboard.clone(),
        )?;
        
        Ok(Self {
            config,
            server,
            dashboard,
            runtime_manager,
            plugin_manager,
        })
    }
    
    pub async fn start(self) -> Result<()> {
        info!("🚀 Starting Backworks Engine...");
        
        // Print startup information
        self.print_startup_info();
        
        // Start dashboard if enabled
        let dashboard_handle = if let Some(dashboard) = self.dashboard.clone() {
            Some(tokio::spawn(async move {
                if let Err(e) = dashboard.start().await {
                    error!("Dashboard error: {}", e);
                }
            }))
        } else {
            None
        };
        
        // Start main server
        let server_handle = tokio::spawn({
            let server = self.server;
            async move {
                if let Err(e) = server.start().await {
                    error!("Server error: {}", e);
                }
            }
        });
        
        // Wait for shutdown signal
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("🛑 Shutdown signal received");
            }
            _ = server_handle => {
                error!("Server unexpectedly stopped");
            }
        }
        
        // Graceful shutdown
        info!("🔄 Shutting down...");
        
        // Shutdown plugins
        if let Err(e) = self.plugin_manager.shutdown_all().await {
            error!("Plugin shutdown error: {}", e);
        }
        
        if let Some(handle) = dashboard_handle {
            handle.abort();
        }
        
        info!("✅ Backworks shutdown complete");
        Ok(())
    }
    
    fn print_startup_info(&self) {
        println!("\n🎉 Backworks is running!");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 API: {}", self.config.name);
        println!("🌐 Server: http://{}:{}", self.config.server.host, self.config.server.port);
        
        if let Some(ref dashboard) = &self.config.dashboard {
            if dashboard.enabled {
                println!("🎨 Dashboard: http://localhost:{}", dashboard.port);
            }
        }
        
        println!("📊 Mode: {:?}", self.config.mode);
        
        // Show enabled plugins
        let plugin_count = self.config.plugins.iter().filter(|(_, config)| config.enabled).count();
        if plugin_count > 0 {
            println!("🔌 Plugins: {} enabled", plugin_count);
            for (name, plugin_config) in &self.config.plugins {
                if plugin_config.enabled {
                    println!("   └─ {}", name);
                }
            }
        }
        
        // Database functionality is now handled by plugins
        if self.config.database.is_some() {
            println!("🗄️  Database: Configuration present (handled by plugins)");
        }
        
        println!("📑 Endpoints:");
        for (name, endpoint) in &self.config.endpoints {
            println!("   {} {} -> {}", 
                endpoint.methods.join("|"), 
                endpoint.path,
                name
            );
        }
        
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Press Ctrl+C to stop\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ServerConfig, EndpointConfig, ExecutionMode};
    use std::collections::HashMap;
    
    fn create_test_config() -> BackworksConfig {
        let mut endpoints = HashMap::new();
        endpoints.insert("test".to_string(), EndpointConfig {
            path: "/test".to_string(),
            methods: vec!["GET".to_string()],
            description: None,
            mode: None,
            // mock and mock_responses fields removed (deprecated)
            runtime: None,
            database: None,
            capture: None,
            ai_enhanced: None,
            ai_suggestions: None,
            apis: None,
            parameters: None,
            validation: None,
            monitoring: None,
            plugin: None,
        });
        
        BackworksConfig {
            name: "test_api".to_string(),
            description: None,
            version: None,
            mode: ExecutionMode::Runtime,
            endpoints,
            server: ServerConfig::default(),
            plugins: HashMap::new(), // Replaced ai field with plugins
            plugin_discovery: crate::config::PluginDiscoveryConfig::default(),
            dashboard: None,
            database: None,
            apis: None,
            cache: None,
            security: None,
            monitoring: None,
            global_headers: HashMap::new(),
            logging: Default::default(),
        }
    }
    
    #[tokio::test]
    async fn test_engine_creation() {
        let config = create_test_config();
        let engine = BackworksEngine::new(config).await;
        assert!(engine.is_ok());
    }
}
