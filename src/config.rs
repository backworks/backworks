//! Configuration structures and utilities for Backworks
//! 
//! This module contains all configuration structures used throughout Backworks,
//! including deprecated structs kept for backward compatibility.

#![allow(deprecated)] // Allow deprecated MockConfig and MockResponse for backward compatibility

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::error::{BackworksError, Result};
use crate::plugin::PluginConfig;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    #[serde(rename = "runtime")]
    Runtime,
    #[serde(rename = "database")]
    Database,
    #[default]
    #[serde(rename = "plugin")]
    Plugin,
}





#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTarget {
    pub enabled: bool,
    pub profile: String,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProfiles {
    pub profiles: HashMap<String, SecurityProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProfile {
    pub strip_secrets: bool,
    pub enable_debug: bool,
    pub verbose_logging: bool,
    pub obfuscate_internals: Option<bool>,
    pub enable_rate_limiting: Option<bool>,
    pub require_https: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: String,
    pub output: String,
}

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
    
    // Plugin configurations (replaces individual feature configs like AI)
    #[serde(default)]
    pub plugins: HashMap<String, PluginConfig>,
    
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

// ExecutionMode enum is defined above

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
    
    // Mock configuration (removed)
    // Removed mock and mock_responses fields
    
    // Runtime configuration  
    pub runtime: Option<RuntimeConfig>,
    
    // Database configuration
    pub database: Option<EndpointDatabaseConfig>,
    
    // Capture configuration
    pub capture: Option<CaptureConfig>,
    
    // Plugin configuration
    pub plugin: Option<String>,
    
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

#[deprecated(since = "0.2.0", note = "Mock mode is deprecated, use runtime or plugin mode instead")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockConfig {
    pub data: Option<serde_json::Value>,
    pub ai_generated: Option<bool>,
    pub based_on_patterns: Option<bool>,
    pub count: Option<usize>,
    pub patterns: Option<Vec<String>>,
}

#[deprecated(since = "0.2.0", note = "Mock mode is deprecated, use runtime or plugin mode instead")]
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
pub struct TransformConfig {
    // Header transformations
    pub add_headers: Option<HashMap<String, String>>,
    pub remove_headers: Option<Vec<String>>,
    pub header_mapping: Option<HashMap<String, String>>,
    
    // Status code transformations
    pub status_code_mapping: Option<HashMap<u16, u16>>,
    pub force_status_code: Option<u16>,
    
    // Body transformations
    pub body_transform: Option<BodyTransform>,
    
    // Path and query transformations
    pub path_rewrite: Option<PathRewrite>,
    pub query_transform: Option<QueryTransform>,
    
    // Content type conversions
    pub content_conversion: Option<ContentConversion>,
    
    // Response filtering and formatting
    pub response_filter: Option<ResponseFilter>,
    
    // Template-based transformations
    pub template: Option<TemplateTransform>,
    
    // Script-based transformations (JavaScript/Lua)
    pub script: Option<ScriptTransform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyTransform {
    // JSON transformations
    pub json_path_mapping: Option<HashMap<String, String>>, // "$.user.name" -> "$.userName"
    pub json_field_addition: Option<HashMap<String, serde_json::Value>>,
    pub json_field_removal: Option<Vec<String>>,
    pub json_field_renaming: Option<HashMap<String, String>>,
    
    // String transformations
    pub string_replace: Option<Vec<StringReplace>>,
    pub string_template: Option<String>, // Handlebars-style template
    
    // Format conversions
    pub input_format: Option<ContentFormat>,
    pub output_format: Option<ContentFormat>,
    
    // Custom transformation script
    pub transform_script: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringReplace {
    pub pattern: String,
    pub replacement: String,
    pub is_regex: Option<bool>,
    pub case_sensitive: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentFormat {
    Json,
    Xml,
    Yaml,
    Csv,
    PlainText,
    FormData,
    Base64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRewrite {
    pub strip_prefix: Option<String>,
    pub add_prefix: Option<String>,
    pub pattern_replace: Option<Vec<PathReplace>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathReplace {
    pub pattern: String, // regex pattern
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTransform {
    pub add_params: Option<HashMap<String, String>>,
    pub remove_params: Option<Vec<String>>,
    pub rename_params: Option<HashMap<String, String>>,
    pub default_values: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentConversion {
    pub from: ContentFormat,
    pub to: ContentFormat,
    pub options: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFilter {
    pub include_fields: Option<Vec<String>>, // JSONPath expressions
    pub exclude_fields: Option<Vec<String>>, // JSONPath expressions
    pub field_filters: Option<HashMap<String, FieldFilter>>,
    pub pagination: Option<PaginationTransform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldFilter {
    pub condition: String, // "gt:100", "contains:test", "regex:^[A-Z]"
    pub action: FilterAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterAction {
    Include,
    Exclude,
    Transform(String), // transformation expression
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationTransform {
    pub page_param: String,
    pub size_param: String,
    pub total_field: Option<String>,
    pub data_field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateTransform {
    pub engine: TemplateEngine,
    pub request_template: Option<String>,
    pub response_template: Option<String>,
    pub variables: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateEngine {
    Handlebars,
    Mustache,
    Jinja2,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptTransform {
    pub language: ScriptLanguage,
    pub request_script: Option<String>,
    pub response_script: Option<String>,
    pub timeout: Option<u64>,
    pub sandbox: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptLanguage {
    JavaScript,
    Lua,
    Python,
    Wasm,
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
    load_yaml_config(path).await
}

/// Load YAML configuration with support for both old and new formats
pub async fn load_yaml_config(path: &PathBuf) -> Result<BackworksConfig> {
    let content = tokio::fs::read_to_string(path).await?;
    
    // Try new array-based format first
    if let Ok(new_config) = serde_yaml::from_str::<NewBlueprintConfig>(&content) {
        let config = new_config.to_backworks_config();
        validate_config(&config)?;
        Ok(config)
    } else {
        // Fallback to legacy HashMap format
        let config: BackworksConfig = serde_yaml::from_str(&content)?;
        validate_config(&config)?;
        Ok(config)
    }
}

pub fn validate_config(config: &BackworksConfig) -> Result<()> {
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
    
    // Validate plugin configurations
    for (plugin_name, plugin_config) in &config.plugins {
        if plugin_config.enabled {
            // Basic validation - each plugin can have its own validation logic
            match plugin_name.as_str() {
                "ai" => {
                    // AI plugin specific validation could go here
                    // For now, just ensure the config is valid JSON
                },
                _ => {
                    // Unknown plugins are allowed for extensibility
                }
            }
        }
    }
    
    Ok(())
}



/// Detect project structure and load appropriate configuration - YAML-only approach
pub fn load_project_config(path: Option<PathBuf>) -> Result<BackworksConfig> {
    let current_dir = std::env::current_dir()
        .map_err(|e| BackworksError::config(format!("Cannot get current directory: {}", e)))?;
    
    if let Some(config_path) = path {
        // Explicit config file provided - try to load as YAML
        let config = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                load_yaml_config(&config_path).await
            })
        })?;
        Ok(config)
    } else {
        // Auto-detect project structure - look for YAML files only
        
        // Try backworks.yaml first (new preferred format)
        let backworks_file = current_dir.join("backworks.yaml");
        if backworks_file.exists() {
            let config = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    load_yaml_config(&backworks_file).await
                })
            })?;
            Ok(config)
        } else {
            // Try main.yaml in current directory
            let main_file = current_dir.join("main.yaml");
            if main_file.exists() {
                let config = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        load_yaml_config(&main_file).await
                    })
                })?;
                Ok(config)
            } else {
                // Try blueprints/main.yaml (project structure)
                let blueprint_file = current_dir.join("blueprints/main.yaml");
                if blueprint_file.exists() {
                    let config = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            load_yaml_config(&blueprint_file).await
                        })
                    })?;
                    Ok(config)
                } else {
                    // Legacy blueprint.yaml
                    let legacy_file = current_dir.join("blueprint.yaml");
                    if legacy_file.exists() {
                        let config = tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(async {
                                load_yaml_config(&legacy_file).await
                            })
                        })?;
                        Ok(config)
                    } else {
                        return Err(BackworksError::config(
                            "No configuration found. Expected 'backworks.yaml', 'main.yaml', 'blueprints/main.yaml' or 'blueprint.yaml'".to_string()
                        ));
                    }
                }
            }
        }
    }
}

/// New blueprint format with array-based endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBlueprintConfig {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    
    // Array-based endpoints (new format)
    pub endpoints: Vec<NewEndpointConfig>,
    
    // Global settings
    #[serde(default)]
    pub server: ServerConfig,
    
    #[serde(default)]
    pub dashboard: Option<DashboardConfig>,
    
    #[serde(default)]
    pub plugins: HashMap<String, PluginConfig>,
    
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// New endpoint configuration for array-based format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEndpointConfig {
    pub path: String,
    
    // Single method or array of methods
    #[serde(alias = "methods")]
    pub method: MethodSpec,
    
    pub description: Option<String>,
    
    // Handler (JavaScript function or file path)
    pub handler: Option<String>,
    
    // Runtime configuration for handlers
    pub runtime: Option<RuntimeConfig>,
    
    // Middleware
    #[serde(default)]
    pub middleware: Vec<String>,
}

/// Method specification - supports both single method and array
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MethodSpec {
    Single(String),
    Multiple(Vec<String>),
}

impl MethodSpec {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            MethodSpec::Single(method) => vec![method.clone()],
            MethodSpec::Multiple(methods) => methods.clone(),
        }
    }
}

impl NewBlueprintConfig {
    /// Convert new blueprint format to legacy BackworksConfig format
    pub fn to_backworks_config(self) -> BackworksConfig {
        let mut endpoints = HashMap::new();
        
        // Convert array-based endpoints to map-based endpoints
        for (index, endpoint) in self.endpoints.into_iter().enumerate() {
            let endpoint_name = if let Some(last_segment) = endpoint.path.split('/').last() {
                if last_segment.is_empty() || last_segment.starts_with('{') {
                    format!("endpoint_{}", index)
                } else {
                    last_segment.replace('{', "").replace('}', "")
                }
            } else {
                format!("endpoint_{}", index)
            };
            
            // Create runtime config for handlers
            let runtime = if let Some(handler) = endpoint.handler {
                Some(RuntimeConfig {
                    language: "javascript".to_string(),
                    handler,
                    timeout: None,
                    memory_limit: None,
                    environment: None,
                    requirements: None,
                    working_dir: None,
                })
            } else {
                endpoint.runtime
            };
            
            let legacy_endpoint = EndpointConfig {
                path: endpoint.path,
                methods: endpoint.method.to_vec(),
                description: endpoint.description,
                mode: Some(ExecutionMode::Runtime),
                runtime,
                database: None,
                capture: None,
                plugin: None,
                ai_enhanced: None,
                ai_suggestions: None,
                apis: None,
                parameters: None,
                validation: None,
                monitoring: None,
            };
            
            endpoints.insert(endpoint_name, legacy_endpoint);
        }
        
        // Determine global mode based on endpoints
        let has_runtime = endpoints.values().any(|e| e.runtime.is_some());
        
        let global_mode = if has_runtime {
            ExecutionMode::Runtime
        } else {
            ExecutionMode::Plugin
        };
        
        BackworksConfig {
            name: self.name,
            description: self.description,
            version: self.version,
            mode: global_mode,
            endpoints,
            server: self.server,
            plugins: self.plugins,
            dashboard: self.dashboard,
            // Set defaults for other fields
            database: None,
            apis: None,
            cache: None,
            security: None,
            monitoring: None,
            global_headers: HashMap::new(),
            logging: self.logging,
        }
    }
}

/// Load configuration supporting both new and legacy blueprint formats
pub async fn load_blueprint_config(path: &PathBuf) -> Result<BackworksConfig> {
    let content = tokio::fs::read_to_string(path).await
        .map_err(|e| BackworksError::config(format!("Failed to read blueprint file: {}", e)))?;
    
    // Try new array-based format first
    if let Ok(new_config) = serde_yaml::from_str::<NewBlueprintConfig>(&content) {
        let config = new_config.to_backworks_config();
        validate_config(&config)?;
        Ok(config)
    } else {
        // Fallback to legacy format
        let config: BackworksConfig = serde_yaml::from_str(&content)
            .map_err(|e| BackworksError::config(format!("Failed to parse blueprint: {}", e)))?;
        validate_config(&config)?;
        Ok(config)
    }
}
