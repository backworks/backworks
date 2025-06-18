//! AI Enhancement Plugin for Backworks
//! 
//! Provides AI-powered features like smart response generation,
//! pattern detection, and configuration suggestions.

use crate::error::BackworksResult;
use crate::plugin::{BackworksPlugin, PluginHealth, HealthStatus};
use axum::{http::Request, response::Response};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

/// AI Enhancement Plugin
pub struct AIPlugin {
    config: std::sync::Arc<std::sync::RwLock<AIPluginConfig>>,
    initialized: std::sync::atomic::AtomicBool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AIPluginConfig {
    #[serde(default)]
    pub features: Vec<String>,
    
    #[serde(default = "default_model")]
    pub model: String,
    
    #[serde(default = "default_context_window")]
    pub context_window: usize,
    
    #[serde(default)]
    pub api_key: Option<String>,
    
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    
    #[serde(default)]
    pub cache_responses: bool,
    
    #[serde(default)]
    pub smart_responses: SmartResponsesConfig,
    
    #[serde(default)]
    pub pattern_detection: PatternDetectionConfig,
    
    #[serde(default)]
    pub config_generation: ConfigGenerationConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmartResponsesConfig {
    #[serde(default)]
    pub enabled: bool,
    
    #[serde(default = "default_creativity")]
    pub creativity: f32,
    
    #[serde(default)]
    pub consistent_personas: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatternDetectionConfig {
    #[serde(default)]
    pub enabled: bool,
    
    #[serde(default = "default_min_requests")]
    pub min_requests: usize,
    
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigGenerationConfig {
    #[serde(default)]
    pub enabled: bool,
    
    #[serde(default)]
    pub auto_suggest: bool,
    
    #[serde(default)]
    pub suggest_optimizations: bool,
}

fn default_model() -> String { "gpt-3.5-turbo".to_string() }
fn default_context_window() -> usize { 4000 }
fn default_timeout() -> u64 { 30 }
fn default_creativity() -> f32 { 0.7 }
fn default_min_requests() -> usize { 10 }
fn default_confidence_threshold() -> f32 { 0.8 }

impl Default for AIPluginConfig {
    fn default() -> Self {
        Self {
            features: vec!["smart_responses".to_string()],
            model: default_model(),
            context_window: default_context_window(),
            api_key: None,
            timeout_seconds: default_timeout(),
            cache_responses: false,
            smart_responses: SmartResponsesConfig::default(),
            pattern_detection: PatternDetectionConfig::default(),
            config_generation: ConfigGenerationConfig::default(),
        }
    }
}

impl Default for SmartResponsesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            creativity: default_creativity(),
            consistent_personas: true,
        }
    }
}

impl Default for PatternDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_requests: default_min_requests(),
            confidence_threshold: default_confidence_threshold(),
        }
    }
}

impl Default for ConfigGenerationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_suggest: false,
            suggest_optimizations: false,
        }
    }
}

impl AIPlugin {
    pub fn new() -> Self {
        Self {
            config: std::sync::Arc::new(std::sync::RwLock::new(AIPluginConfig::default())),
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

#[async_trait::async_trait]
impl BackworksPlugin for AIPlugin {
    fn name(&self) -> &str {
        "ai"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "AI enhancement plugin providing smart responses, pattern detection, and config generation"
    }
    
    async fn initialize(&self, config: &Value) -> BackworksResult<()> {
        tracing::info!("🤖 Initializing AI Plugin");
        
        if let Ok(ai_config) = serde_json::from_value::<AIPluginConfig>(config.clone()) {
            *self.config.write().unwrap() = ai_config;
            self.initialized.store(true, std::sync::atomic::Ordering::Relaxed);
            tracing::info!("✅ AI Plugin initialized successfully");
            Ok(())
        } else {
            Err(crate::error::BackworksError::PluginConfigInvalid(
                "Invalid AI plugin configuration".to_string()
            ))
        }
    }
    
    async fn shutdown(&self) -> BackworksResult<()> {
        tracing::info!("🤖 Shutting down AI Plugin");
        self.initialized.store(false, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    
    async fn health_check(&self) -> BackworksResult<PluginHealth> {
        let is_initialized = self.initialized.load(std::sync::atomic::Ordering::Relaxed);
        
        if is_initialized {
            Ok(PluginHealth {
                status: HealthStatus::Healthy,
                message: "AI Plugin is running normally".to_string(),
                details: {
                    let mut details = HashMap::new();
                    let config = self.config.read().unwrap();
                    details.insert("model".to_string(), serde_json::Value::String(config.model.clone()));
                    details.insert("features".to_string(), serde_json::Value::Array(
                        config.features.iter().map(|f| serde_json::Value::String(f.clone())).collect()
                    ));
                    details
                },
            })
        } else {
            Ok(PluginHealth {
                status: HealthStatus::Unhealthy,
                message: "AI Plugin not initialized".to_string(),
                details: HashMap::new(),
            })
        }
    }
    
    fn max_execution_time(&self) -> Duration {
        Duration::from_millis(500) // AI operations can take longer
    }
    
    fn is_critical(&self) -> bool {
        false // AI plugin is not critical - system should work without it
    }
    
    async fn before_request(&self, request: &mut Request<axum::body::Body>) -> BackworksResult<()> {
        if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }
        
        let config = self.config.read().unwrap();
        if !config.smart_responses.enabled {
            return Ok(());
        }
        
        // Add AI processing header to indicate AI enhancement is active
        request.headers_mut().insert(
            "X-Backworks-AI-Processing",
            "enabled".parse().unwrap()
        );
        
        tracing::debug!("🤖 AI before_request hook: Added AI processing headers");
        Ok(())
    }
    
    async fn after_response(&self, response: &mut Response<axum::body::Body>) -> BackworksResult<()> {
        if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }
        
        let config = self.config.read().unwrap();
        if !config.smart_responses.enabled {
            return Ok(());
        }
        
        // Add AI enhancement headers
        response.headers_mut().insert(
            "X-Backworks-AI-Enhanced",
            "true".parse().unwrap()
        );
        response.headers_mut().insert(
            "X-Backworks-AI-Model",
            config.model.parse().unwrap()
        );
        
        tracing::debug!("🤖 AI after_response hook: Added AI enhancement headers");
        Ok(())
    }
    
    async fn on_config_reload(&self, config: &Value) -> BackworksResult<()> {
        tracing::info!("🔄 AI Plugin config reload requested");
        
        if let Ok(new_config) = serde_json::from_value::<AIPluginConfig>(config.clone()) {
            *self.config.write().unwrap() = new_config;
            tracing::info!("✅ AI Plugin config reloaded successfully");
            Ok(())
        } else {
            Err(crate::error::BackworksError::PluginConfigInvalid(
                "Invalid AI plugin configuration for reload".to_string()
            ))
        }
    }
}
