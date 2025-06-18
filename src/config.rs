use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::error::{BackworksError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackworksConfig {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    
    #[serde(default)]
    pub mode: ExecutionMode,
    
    pub endpoints: HashMap<String, EndpointConfig>,
    
    #[serde(default)]
    pub server: ServerConfig,
    
    #[serde(default)]
    pub ai: AIConfig,
    
    pub dashboard: Option<DashboardConfig>,
    pub database: Option<DatabaseConfig>,
    pub apis: Option<HashMap<String, ExternalAPIConfig>>,
    pub cache: Option<CacheConfig>,
    pub security: Option<SecurityConfig>,
    pub monitoring: Option<MonitoringConfig>,
    
    #[serde(default)]
    pub global_headers: HashMap<String, String>,
    
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ExecutionMode {
    #[default]
    #[serde(rename = "mock")]
    Mock,
    #[serde(rename = "capture")]
    Capture,
    #[serde(rename = "runtime")]
    Runtime,
    #[serde(rename = "database")]
    Database,
    #[serde(rename = "proxy")]
    Proxy,
    #[serde(rename = "hybrid")]
    Hybrid,
    #[serde(rename = "evolving")]
    Evolving,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
        }
    }
}

fn default_port() -> u16 { 8080 }
fn default_host() -> String { "0.0.0.0".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub path: String,
    
    #[serde(default = "default_methods")]
    pub methods: Vec<String>,
    
    pub description: Option<String>,
    
    #[serde(default)]
    pub mode: Option<ExecutionMode>,
    
    // Mock configuration
    pub mock: Option<MockConfig>,
    pub mock_responses: Option<HashMap<String, MockResponse>>,
    
    // Runtime configuration  
    pub runtime: Option<RuntimeConfig>,
    
    // Database configuration
    pub database: Option<EndpointDatabaseConfig>,
    
    // Proxy configuration
    pub proxy: Option<ProxyConfig>,
    
    // Capture configuration
    pub capture: Option<CaptureConfig>,
    
    // AI configuration
    pub ai_enhanced: Option<bool>,
    pub ai_suggestions: Option<AIEndpointSuggestions>,
    
    // API access
    pub apis: Option<Vec<String>>,
    
    // Parameter validation
    pub parameters: Option<Vec<ParameterConfig>>,
    pub validation: Option<ValidationConfig>,
    
    // Monitoring
    pub monitoring: Option<EndpointMonitoringConfig>,
}

fn default_methods() -> Vec<String> {
    vec!["GET".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockConfig {
    pub data: Option<serde_json::Value>,
    pub ai_generated: Option<bool>,
    pub based_on_patterns: Option<bool>,
    pub count: Option<usize>,
    pub patterns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockResponse {
    #[serde(default = "default_status")]
    pub status: u16,
    pub headers: Option<HashMap<String, String>>,
    pub body: serde_json::Value,
}

fn default_status() -> u16 { 200 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub language: String,
    pub handler: String,
    pub timeout: Option<u64>,
    pub memory_limit: Option<String>,
    pub environment: Option<HashMap<String, String>>,
    pub requirements: Option<String>,
    pub working_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerConfig {
    pub language: String,
    pub script: String,
    pub timeout: Option<u64>,
    pub environment: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointDatabaseConfig {
    pub table: Option<String>,
    pub auto_crud: Option<bool>,
    pub queries: Option<HashMap<String, String>>,
    pub transform: Option<DatabaseTransformConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTransformConfig {
    pub list: Option<String>,
    pub single: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub target: String,
    pub targets: Option<Vec<ProxyTarget>>,
    pub strip_prefix: Option<String>,
    pub timeout: Option<u64>,
    pub transform_request: Option<TransformConfig>,
    pub transform_response: Option<TransformConfig>,
    pub health_checks: Option<bool>,
    pub load_balancing: Option<LoadBalancingConfig>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyTarget {
    pub name: String,
    pub url: String,
    pub weight: Option<f64>,
    pub timeout: Option<std::time::Duration>,
    pub health_check: Option<HealthCheck>,
    pub retry_attempts: Option<u32>,
    pub circuit_breaker: Option<CircuitBreakerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub path: String,
    pub interval: std::time::Duration,
    pub timeout: std::time::Duration,
    pub healthy_threshold: u32,
    pub unhealthy_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: std::time::Duration,
    pub request_volume_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    pub algorithm: LoadBalancingAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    Weighted,
    IpHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    pub add_headers: Option<HashMap<String, String>>,
    pub remove_headers: Option<Vec<String>>,
    pub transform_body: Option<String>,
    pub status_code_mapping: Option<HashMap<u16, u16>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub analyze: Option<bool>,
    pub learn_schema: Option<bool>,
    pub enabled: Option<bool>,
    pub auto_start: Option<bool>,
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub methods: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIEndpointSuggestions {
    pub missing_fields: Option<Vec<FieldSuggestion>>,
    pub related_endpoints: Option<Vec<EndpointSuggestion>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSuggestion {
    pub name: String,
    pub confidence: f64,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSuggestion {
    pub path: String,
    pub confidence: f64,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: Option<bool>,
    pub minimum: Option<i64>,
    pub maximum: Option<i64>,
    pub max_length: Option<usize>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub create: Option<HashMap<String, serde_json::Value>>,
    pub update: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMonitoringConfig {
    pub display_name: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub category: Option<String>,
    pub critical: Option<bool>,
    pub expected_duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AIConfig {
    #[serde(default)]
    pub enabled: bool,
    pub features: Option<Vec<String>>,
    pub models: Option<HashMap<String, AIModelConfig>>,
    pub learning: Option<AILearningConfig>,
    pub endpoint_ai: Option<HashMap<String, AIEndpointConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelConfig {
    #[serde(rename = "type")]
    pub model_type: String,
    pub path: Option<String>,
    pub confidence_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AILearningConfig {
    pub enabled: Option<bool>,
    pub retention_days: Option<u64>,
    pub export_insights: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIEndpointConfig {
    pub generate_realistic_data: Option<bool>,
    pub analyze_usage_patterns: Option<bool>,
    pub predict_missing_fields: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    #[serde(default = "default_dashboard_port")]
    pub port: u16,
    
    #[serde(default)]
    pub enabled: bool,
    
    pub features: Option<Vec<String>>,
    pub real_time: Option<RealTimeConfig>,
    pub visualization: Option<VisualizationConfig>,
    pub access: Option<AccessConfig>,
}

fn default_dashboard_port() -> u16 { 3000 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeConfig {
    pub enabled: Option<bool>,
    pub update_frequency: Option<u64>,
    pub max_history: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub theme: Option<String>,
    pub layout: Option<String>,
    pub animations: Option<bool>,
    pub color_scheme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessConfig {
    pub public: Option<bool>,
    pub api_key_env: Option<String>,
    pub allowed_ips: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(rename = "type")]
    pub db_type: String,
    pub connection_string: Option<String>,
    pub connection_string_env: Option<String>,
    pub pool: Option<PoolConfig>,
    pub databases: Option<HashMap<String, DatabaseConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub min_connections: Option<u32>,
    pub max_connections: Option<u32>,
    pub connection_timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAPIConfig {
    pub base_url: String,
    pub authentication: Option<AuthenticationConfig>,
    pub headers: Option<HashMap<String, String>>,
    pub timeout: Option<u64>,
    pub rate_limit: Option<RateLimitConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    #[serde(rename = "type")]
    pub auth_type: String,
    pub token_env: Option<String>,
    pub client_id_env: Option<String>,
    pub client_secret_env: Option<String>,
    pub token_url: Option<String>,
    pub scope: Option<String>,
    pub username_env: Option<String>,
    pub password_env: Option<String>,
    pub key_env: Option<String>,
    pub location: Option<String>,
    pub parameter: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(rename = "type")]
    pub cache_type: String,
    pub connection_string: Option<String>,
    pub connection_string_env: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub cors: Option<CorsConfig>,
    pub rate_limiting: Option<SecurityRateLimitConfig>,
    pub authentication: Option<SecurityAuthConfig>,
    pub validation: Option<SecurityValidationConfig>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub enabled: Option<bool>,
    pub origins: Option<Vec<String>>,
    pub methods: Option<Vec<String>>,
    pub headers: Option<Vec<String>>,
    pub credentials: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRateLimitConfig {
    pub enabled: Option<bool>,
    pub requests_per_minute: Option<u64>,
    pub burst_size: Option<u64>,
    pub key_generator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuthConfig {
    #[serde(rename = "type")]
    pub auth_type: String,
    pub secret_env: Option<String>,
    pub algorithm: Option<String>,
    pub expiration: Option<u64>,
    pub validation: Option<AuthValidationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthValidationConfig {
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityValidationConfig {
    pub max_body_size: Option<String>,
    pub require_content_type: Option<bool>,
    pub validate_json: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics: Option<MetricsConfig>,
    pub logging: Option<MonitoringLoggingConfig>,
    pub health: Option<HealthConfig>,
    pub alerts: Option<AlertsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: Option<bool>,
    pub export_format: Option<String>,
    pub export_endpoint: Option<String>,
    pub custom: Option<Vec<CustomMetricConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetricConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub metric_type: String,
    pub description: String,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringLoggingConfig {
    pub level: Option<String>,
    pub format: Option<String>,
    pub output: Option<String>,
    pub file: Option<FileLoggingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLoggingConfig {
    pub path: String,
    pub max_size: Option<String>,
    pub max_files: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    pub enabled: Option<bool>,
    pub endpoint: Option<String>,
    pub checks: Option<Vec<HealthCheckConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub check_type: String,
    pub timeout: Option<u64>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsConfig {
    pub enabled: Option<bool>,
    pub channels: Option<HashMap<String, AlertChannelConfig>>,
    pub rules: Option<Vec<AlertRuleConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertChannelConfig {
    pub webhook_url_env: Option<String>,
    pub channel: Option<String>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub username_env: Option<String>,
    pub password_env: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRuleConfig {
    pub name: String,
    pub condition: String,
    pub duration: Option<String>,
    pub channels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub include_body: bool,
    #[serde(default)]
    pub include_headers: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

pub async fn load_config(path: &PathBuf) -> Result<BackworksConfig> {
    let content = tokio::fs::read_to_string(path).await?;
    let config: BackworksConfig = serde_yaml::from_str(&content)?;
    validate_config(&config)?;
    Ok(config)
}

fn validate_config(config: &BackworksConfig) -> Result<()> {
    // Basic validation
    if config.name.is_empty() {
        return Err(BackworksError::config("API name cannot be empty"));
    }
    
    if config.endpoints.is_empty() {
        return Err(BackworksError::config("At least one endpoint must be defined"));
    }
    
    // Validate endpoints
    for (name, endpoint) in &config.endpoints {
        if endpoint.path.is_empty() {
            return Err(BackworksError::config(format!("Endpoint '{}' path cannot be empty", name)));
        }
        
        if endpoint.methods.is_empty() {
            return Err(BackworksError::config(format!("Endpoint '{}' must have at least one HTTP method", name)));
        }
        
        // Validate HTTP methods
        for method in &endpoint.methods {
            match method.as_str() {
                "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" => {},
                _ => return Err(BackworksError::config(format!("Invalid HTTP method '{}' in endpoint '{}'", method, name))),
            }
        }
    }
    
    // Validate AI config if enabled
    if config.ai.enabled {
        if let Some(ref models) = config.ai.models {
            for (name, model) in models {
                match model.model_type.as_str() {
                    "onnx" | "candle" => {},
                    _ => return Err(BackworksError::config(format!("Invalid AI model type '{}' for model '{}'", model.model_type, name))),
                }
            }
        }
    }
    
    Ok(())
}
