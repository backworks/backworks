use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

use crate::config::{BackworksConfig, ExecutionMode};
use crate::server::BackworksServer;
use crate::dashboard::Dashboard;
use crate::ai::AIEnhancer;
use crate::database::DatabaseManager;
use crate::runtime::{RuntimeManager, RuntimeManagerConfig};
use crate::error::{BackworksError, Result};

pub struct BackworksEngine {
    config: Arc<BackworksConfig>,
    server: BackworksServer,
    dashboard: Option<Dashboard>,
    ai_enhancer: Option<AIEnhancer>,
    database_manager: Option<DatabaseManager>,
    runtime_manager: RuntimeManager,
}

impl BackworksEngine {
    pub async fn new(config: BackworksConfig) -> Result<Self> {
        let config = Arc::new(config);
        
        info!("🎯 Initializing Backworks Engine");
        info!("   Name: {}", config.name);
        info!("   Mode: {:?}", config.mode);
        info!("   Endpoints: {}", config.endpoints.len());
        
        // Initialize AI enhancer if enabled
        let ai_enhancer = if config.ai.enabled {
            info!("🤖 Initializing AI enhancer...");
            Some(AIEnhancer::new(config.ai.clone()))
        } else {
            None
        };
        
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
            ai_enhancer.clone(),
            database_manager.clone(),
            runtime_manager.clone(),
        )?;
        
        Ok(Self {
            config,
            server,
            dashboard,
            ai_enhancer,
            database_manager,
            runtime_manager,
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
        
        if self.config.ai.enabled {
            println!("🤖 AI: Enabled");
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
