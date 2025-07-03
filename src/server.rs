use std::sync::Arc;
use std::collections::HashMap;
use axum::{
    Router,
    routing::{get, post, put, delete, any},
    response::Json,
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap, Method},
    middleware,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
};
use serde_json::Value;
use serde::{Serialize, Deserialize};
use tracing::{info, debug, error};

use crate::config::{BackworksConfig, ExecutionMode};
use crate::runtime::RuntimeManager;
use crate::plugin::PluginManager;
use crate::dashboard::Dashboard;
use crate::error::{BackworksError, Result};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<BackworksConfig>,
    pub plugin_manager: PluginManager,
    pub runtime_manager: RuntimeManager,
    pub dashboard: Option<Arc<Dashboard>>,
}

pub struct BackworksServer {
    state: AppState,
}

impl BackworksServer {
    pub fn new(
        config: Arc<BackworksConfig>,
        plugin_manager: PluginManager,
        dashboard: Option<Arc<Dashboard>>,
    ) -> Result<Self> {
        // Initialize runtime manager
        let runtime_config = crate::runtime::RuntimeManagerConfig::default();
        let runtime_manager = RuntimeManager::new(runtime_config);
        
        let state = AppState {
            config,
            plugin_manager,
            runtime_manager,
            dashboard,
        };
        
        Ok(Self { state })
    }
    
    pub async fn start(self) -> Result<()> {
        let app = self.create_app();
        
        let listener = tokio::net::TcpListener::bind(
            format!("{}:{}", self.state.config.server.host, self.state.config.server.port)
        ).await?;
        
        info!("🌐 API server listening on {}", listener.local_addr()?);
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }
    
    fn create_app(&self) -> Router {
        let mut app = Router::new();
        
        // Add global middleware
        app = app.layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(self.create_cors_layer())
                .layer(middleware::from_fn_with_state(
                    self.state.clone(),
                    request_middleware,
                ))
        );
        
        // Add health check endpoint
        app = app.route("/health", get(health_check));
        
        // Add metrics endpoint if monitoring is enabled
        if let Some(ref monitoring) = &self.state.config.monitoring {
            if let Some(ref metrics) = &monitoring.metrics {
                if metrics.enabled.unwrap_or(false) {
                    let endpoint = metrics.export_endpoint.as_deref().unwrap_or("/metrics");
                    app = app.route(endpoint, get(metrics_handler));
                }
            }
        }
        
        // Add dynamic endpoints based on configuration
        for (name, endpoint_config) in &self.state.config.endpoints {
            let path = &endpoint_config.path;
            debug!("Registering endpoint: {} -> {}", name, path);
            
            // Create handler for each HTTP method
            for method in &endpoint_config.methods {
                let handler = create_endpoint_handler(method.clone(), name.clone());
                
                app = match method.as_str() {
                    "GET" => app.route(path, get(handler)),
                    "POST" => app.route(path, post(handler)),
                    "PUT" => app.route(path, put(handler)),
                    "DELETE" => app.route(path, delete(handler)),
                    "PATCH" => app.route(path, axum::routing::patch(handler)),
                    _ => app.route(path, any(handler)),
                };
            }
        }
        
        app.with_state(self.state.clone())
    }
    
    fn create_cors_layer(&self) -> CorsLayer {
        let mut cors = CorsLayer::new();
        
        if let Some(ref security) = &self.state.config.security {
            if let Some(ref cors_config) = &security.cors {
                if cors_config.enabled.unwrap_or(false) {
                    if let Some(ref origins) = &cors_config.origins {
                        for origin in origins {
                            // Parse as HeaderValue and create AllowOrigin
                            if let Ok(header_value) = origin.parse::<http::HeaderValue>() {
                                let allow_origin = tower_http::cors::AllowOrigin::exact(header_value);
                                cors = cors.allow_origin(allow_origin);
                            }
                        }
                    } else {
                        cors = cors.allow_origin(Any);
                    }
                    
                    if let Some(ref methods) = &cors_config.methods {
                        let parsed_methods: Vec<Method> = methods
                            .iter()
                            .filter_map(|m| m.parse().ok())
                            .collect();
                        cors = cors.allow_methods(parsed_methods);
                    }
                    
                    if let Some(ref headers) = cors_config.headers {
                        for header in headers {
                            cors = cors.allow_headers([header.parse().unwrap()]);
                        }
                    }
                    
                    if cors_config.credentials.unwrap_or(false) {
                        cors = cors.allow_credentials(true);
                    }
                }
            }
        }
        
        cors
    }
}

// Middleware for request processing and plugin hooks
async fn request_middleware(
    State(state): State<AppState>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let start_time = std::time::Instant::now();
    
    // Call before_request hooks on all plugins
    if let Err(e) = state.plugin_manager.before_request(&mut request).await {
        error!("Plugin before_request hook failed: {}", e);
    }
    
    // Process request through middleware chain
    let mut response = next.run(request).await;
    
    // Call after_response hooks on all plugins
    if let Err(e) = state.plugin_manager.after_response(&mut response).await {
        error!("Plugin after_response hook failed: {}", e);
    }
    
    let duration = start_time.elapsed();
    debug!("Request processed in {:?}", duration);
    
    response
}

// Create handler function for specific endpoint and method
fn create_endpoint_handler(
    method: String,
    endpoint_name: String,
) -> impl Fn(State<AppState>, axum::extract::OriginalUri, Path<HashMap<String, String>>, Query<HashMap<String, String>>, HeaderMap, Option<axum::extract::Json<Value>>) -> std::pin::Pin<Box<dyn std::future::Future<Output = axum::response::Result<(StatusCode, Json<Value>)>> + Send>> + Clone + Send + Sync + 'static {
    move |state, original_uri, path, query, headers, body| {
        let method = method.clone();
        let endpoint_name = endpoint_name.clone();
        
        Box::pin(async move {
            handle_endpoint_request(state, original_uri, method, endpoint_name, path, query, headers, body).await
        })
    }
}

// Main endpoint request handler
async fn handle_endpoint_request(
    State(state): State<AppState>,
    axum::extract::OriginalUri(original_uri): axum::extract::OriginalUri,
    method: String,
    endpoint_name: String,
    Path(path_params): Path<HashMap<String, String>>,
    Query(query_params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::extract::Json<Value>>,
) -> axum::response::Result<(StatusCode, Json<Value>)> {
    debug!("Handling {} request to endpoint: {}", method, endpoint_name);
    
    // Extract the original path from the original URI
    let original_path = original_uri.path().to_string();
    debug!("Original request path: {}", original_path);
    
    // Record request start time for dashboard metrics
    let start_time = std::time::Instant::now();
    
    let endpoint_config = match state.config.endpoints.get(&endpoint_name) {
        Some(config) => config,
        None => {
            return Ok((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Endpoint not found"}))
            ));
        }
    };
    
    // Determine execution mode for this endpoint
    let mode = endpoint_config.mode.as_ref().unwrap_or(&state.config.mode);
    
    let request_data = crate::server::RequestData {
        method: method.clone(),
        path: original_path.clone(),
        path_params,
        query_params,
        headers: headers.clone(),
        body: body.map(|b| b.0),
    };

    // Serialize request data for handlers that need string representation
    let request_data_json = serde_json::to_string(&request_data)
        .map_err(|e| BackworksError::Json(e))?;
    
    let result = match mode {
        ExecutionMode::Runtime => {
            if let Some(ref runtime_config) = endpoint_config.runtime {
                state.runtime_manager.handle_request(runtime_config, &request_data_json).await
            } else {
                Err(BackworksError::config("Runtime mode requires runtime configuration"))
            }
        }
        ExecutionMode::Database => {
            // Database mode now requires plugins to handle the actual database operations
            debug!("Database mode endpoint - delegating to plugins");
            
            // Let plugins handle database operations with simple data interface
            let data_str = serde_json::to_string(&request_data)
                .map_err(|e| BackworksError::plugin(format!("Failed to serialize request data: {}", e)))?;
            
            match state.plugin_manager.process_endpoint_data(&endpoint_name, &method, &data_str).await {
                Ok(Some(response)) => Ok(response),
                Ok(None) => Err(BackworksError::config("No plugin handled database endpoint")),
                Err(e) => Err(e),
            }
        }
        ExecutionMode::Plugin => {
            // Handle plugin-based execution
            if let Some(plugin_name) = &endpoint_config.plugin {
                let request_data_json = serde_json::to_string(&request_data)
                    .map_err(|e| BackworksError::Json(e))?;
                state.plugin_manager.execute_plugin(plugin_name, &request_data_json).await
            } else {
                Err(BackworksError::config("Plugin mode requires plugin name"))
            }
        }
    };
    
    match result {
        Ok(response) => {
            // Try to parse as structured response first
            if let Ok(structured_response) = serde_json::from_str::<serde_json::Value>(&response) {
                if let (Some(status), Some(body)) = (
                    structured_response.get("status").and_then(|s| s.as_u64()),
                    structured_response.get("body")
                ) {
                    // Structured response with status, headers, body
                    let status_code = StatusCode::from_u16(status as u16)
                        .unwrap_or(StatusCode::OK);
                    
                    let response_time = start_time.elapsed().as_millis() as f64;
                    if let Some(ref dashboard) = state.dashboard {
                        let path = format!("/{}", endpoint_name);
                        if let Err(e) = dashboard.record_request(&method, &path, response_time, status as u16).await {
                            error!("Failed to record request to dashboard: {}", e);
                        }
                    }
                    
                    return Ok((status_code, Json(body.clone())));
                }
            }
            
            // Fallback: treat as simple JSON response
            let json_value: serde_json::Value = serde_json::from_str(&response)
                .unwrap_or_else(|_| serde_json::json!({"response": response}));
            
            // Record successful request to dashboard
            let response_time = start_time.elapsed().as_millis() as f64;
            if let Some(ref dashboard) = state.dashboard {
                let path = format!("/{}", endpoint_name);
                if let Err(e) = dashboard.record_request(&method, &path, response_time, 200).await {
                    error!("Failed to record request to dashboard: {}", e);
                }
            }
            
            Ok((StatusCode::OK, Json(json_value)))
        },
        Err(e) => {
            error!("Request handling error: {}", e);
            
            // Record failed request to dashboard
            let response_time = start_time.elapsed().as_millis() as f64;
            if let Some(ref dashboard) = state.dashboard {
                let path = format!("/{}", endpoint_name);
                if let Err(dashboard_err) = dashboard.record_request(&method, &path, response_time, 500).await {
                    error!("Failed to record failed request to dashboard: {}", dashboard_err);
                }
            }
            
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()}))
            ))
        }
    }
}

// Health check endpoint
async fn health_check(State(state): State<AppState>) -> Json<Value> {
    let start_time = std::time::Instant::now();
    
    let response = Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    }));
    
    // Record health check request to dashboard
    let response_time = start_time.elapsed().as_millis() as f64;
    if let Some(ref dashboard) = state.dashboard {
        if let Err(e) = dashboard.record_request("GET", "/health", response_time, 200).await {
            error!("Failed to record health check to dashboard: {}", e);
        }
    }
    
    response
}

// Metrics endpoint
async fn metrics_handler(State(state): State<AppState>) -> String {
    let start_time = std::time::Instant::now();
    
    // Simulate metrics collection
    let response = format!(
        "# HELP backworks_requests_total Total number of requests\n\
         # TYPE backworks_requests_total counter\n\
         backworks_requests_total {}\n",
        42 // Simulated request count
    );
    
    // Record metrics request to dashboard
    let response_time = start_time.elapsed().as_millis() as f64;
    if let Some(ref dashboard) = state.dashboard {
        if let Err(e) = dashboard.record_request("GET", "/metrics", response_time, 200).await {
            error!("Failed to record metrics request to dashboard: {}", e);
        }
    }
    
    response
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestData {
    pub method: String,
    pub path: String, // Add original path
    pub path_params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    #[serde(skip)] // HeaderMap doesn't implement Serialize
    pub headers: HeaderMap,
    pub body: Option<Value>,
}
