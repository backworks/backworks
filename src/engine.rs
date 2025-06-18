use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

use crate::config::BackworksConfig;
use crate::server::BackworksServer;
use crate::dashboard::Dashboard;
use crate::database::DatabaseManager;
use crate::runtime::RuntimeManager;
use crate::plugin::PluginManager;
use crate::plugins::ai::AIPlugin;
use crate::resilience::ResilientPluginConfig;
use crate::error::Result;

pub struct BackworksEngine {
    config: Arc<BackworksConfig>,
    server: BackworksServer,
    dashboard: Option<Dashboard>,
    database_manager: Option<DatabaseManager>,
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
        
        // Initialize plugins based on configuration with resilience
        if let Some(ai_config) = config.plugins.get("ai") {
            if ai_config.enabled {
                info!("🔌 Loading AI Plugin with resilience...");
                let ai_plugin = Arc::new(AIPlugin::new());
                
                // Configure resilience settings for AI plugin
                let resilience_config = ResilientPluginConfig {
                    circuit_breaker: Some(crate::resilience::CircuitBreakerConfig {
                        failure_threshold: 3,
                        recovery_timeout: std::time::Duration::from_secs(60),
                        timeout: std::time::Duration::from_millis(500), // AI operations can take longer
                    }),
                    resource_limits: Some(crate::resilience::PluginResourceLimits {
                        max_memory_mb: Some(200), // AI needs more memory
                        max_execution_time: Some(std::time::Duration::from_millis(500)),
                        max_concurrent_operations: Some(5),
                    }),
                    is_critical: false, // AI plugin is not critical
                };
                
                plugin_manager.register_plugin(
                    ai_plugin, 
                    Some(ai_config.config.clone()),
                    Some(resilience_config)
                ).await?;
            }
        }
        
        // Initialize database manager if needed
        let database_manager = if config.database.is_some() || 
            config.endpoints.values().any(|e| e.database.is_some()) {
            info!("🗄️  Initializing database manager...");
            Some(DatabaseManager::new())
        } else {
            None
        };
        
        // Initialize runtime manager
        info!("⚡ Initializing runtime manager...");
        let runtime_config = crate::runtime::RuntimeManagerConfig::default(); // Create empty config for now
        let runtime_manager = RuntimeManager::new(runtime_config);
        
        // Initialize dashboard if enabled
        let dashboard = if let Some(ref dashboard_config) = config.dashboard {
            if dashboard_config.enabled {
                info!("🎨 Initializing dashboard on port {}...", dashboard_config.port);
                Some(Dashboard::new(dashboard_config.clone()))
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
            database_manager.clone(),
            plugin_manager.clone(),
        )?;
        
        Ok(Self {
            config,
            server,
            dashboard,
            database_manager,
            runtime_manager,
            plugin_manager,
        })
    }
    
    pub async fn start(self) -> Result<()> {
        info!("🚀 Starting Backworks Engine...");
        
        // Print startup information
        self.print_startup_info();
        
        // Start dashboard if enabled
        let dashboard_handle = if let Some(dashboard) = self.dashboard {
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
        
        if let Some(ref dashboard) = self.config.dashboard {
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
        
        if self.config.database.is_some() {
            println!("🗄️  Database: Connected");
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
    use crate::config::{ServerConfig, EndpointConfig, MockConfig};
    use std::collections::HashMap;
    
    fn create_test_config() -> BackworksConfig {
        let mut endpoints = HashMap::new();
        endpoints.insert("test".to_string(), EndpointConfig {
            path: "/test".to_string(),
            methods: vec!["GET".to_string()],
            description: None,
            mode: None,
            mock: Some(MockConfig {
                data: Some(serde_json::json!({"message": "test"})),
                ai_generated: None,
                based_on_patterns: None,
                count: None,
                patterns: None,
            }),
            mock_responses: None,
            runtime: None,
            database: None,
            proxy: None,
            capture: None,
            ai_enhanced: None,
            ai_suggestions: None,
            apis: None,
            parameters: None,
            validation: None,
            monitoring: None,
        });
        
        BackworksConfig {
            name: "test_api".to_string(),
            description: None,
            version: None,
            mode: ExecutionMode::Mock,
            endpoints,
            server: ServerConfig::default(),
            ai: Default::default(),
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
