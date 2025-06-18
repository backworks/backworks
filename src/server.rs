use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, put, delete, any},
    response::Json,
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap, Method, Request},
    body::Body,
    middleware,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
};
use serde_json::Value;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tracing::{info, debug, error};

use crate::config::{BackworksConfig, ExecutionMode};
use crate::database::DatabaseManager;
use crate::runtime::RuntimeManager;
use crate::mock::MockHandler;
use crate::proxy::ProxyHandler;
use crate::plugin::PluginManager;
use crate::error::{BackworksError, Result};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<BackworksConfig>,
    pub plugin_manager: PluginManager,
    pub database_manager: Option<DatabaseManager>,
    pub runtime_manager: RuntimeManager,
    pub mock_handler: MockHandler,
    pub proxy_handler: Option<ProxyHandler>,
}

pub struct BackworksServer {
    state: AppState,
}

impl BackworksServer {
    pub fn new(
        config: Arc<BackworksConfig>,
        database_manager: Option<DatabaseManager>,
        plugin_manager: PluginManager,
    ) -> Result<Self> {
        let mock_handler = MockHandler::new(config.clone());
        
        // Initialize runtime manager
        let runtime_config = crate::runtime::RuntimeManagerConfig::default();
        let runtime_manager = RuntimeManager::new(runtime_config);
        
        let proxy_handler = if matches!(config.mode, ExecutionMode::Proxy) {
            // Create a default proxy config for now
            let proxy_config = crate::config::ProxyConfig {
                target: "http://localhost:8081".to_string(),
                targets: None,
                strip_prefix: None,
                timeout: Some(30),
                transform_request: None,
                transform_response: None,
                health_checks: Some(false),
                load_balancing: None,
                headers: None,
                capture: None, // Capture can be enabled per endpoint
            };
            Some(ProxyHandler::new(proxy_config))
        } else {
            None
        };
        
        // Remove standalone capture handler - it's now integrated into proxy
        
        let state = AppState {
            config,
            plugin_manager,
            database_manager,
            runtime_manager,
            mock_handler,
            proxy_handler,
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
        if let Some(ref monitoring) = self.state.config.monitoring {
            if let Some(ref metrics) = monitoring.metrics {
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
        
        if let Some(ref security) = self.state.config.security {
            if let Some(ref cors_config) = security.cors {
                if cors_config.enabled.unwrap_or(false) {
                    if let Some(ref origins) = cors_config.origins {
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
                    
                    if let Some(ref methods) = cors_config.methods {
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
) -> impl Fn(State<AppState>, Path<HashMap<String, String>>, Query<HashMap<String, String>>, HeaderMap, Option<axum::extract::Json<Value>>) -> std::pin::Pin<Box<dyn std::future::Future<Output = axum::response::Result<(StatusCode, Json<Value>)>> + Send>> + Clone + Send + Sync + 'static {
    move |state, path, query, headers, body| {
        let method = method.clone();
        let endpoint_name = endpoint_name.clone();
        
        Box::pin(async move {
            handle_endpoint_request(state, method, endpoint_name, path, query, headers, body).await
        })
    }
}

// Main endpoint request handler
async fn handle_endpoint_request(
    State(state): State<AppState>,
    method: String,
    endpoint_name: String,
    Path(path_params): Path<HashMap<String, String>>,
    Query(query_params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::extract::Json<Value>>,
) -> axum::response::Result<(StatusCode, Json<Value>)> {
    debug!("Handling {} request to endpoint: {}", method, endpoint_name);
    
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
        path_params,
        query_params,
        headers: headers.clone(),
        body: body.map(|b| b.0),
    };

    // Serialize request data for handlers that need string representation
    let request_data_json = serde_json::to_string(&request_data)
        .map_err(|e| BackworksError::Json(e))?;
    
    let result = match mode {
        ExecutionMode::Mock => {
            let mock_result = state.mock_handler.handle_request(&endpoint_name, endpoint_config, &request_data).await?;
            Ok(serde_json::to_string(&mock_result).map_err(|e| BackworksError::Json(e))?)
        }
        ExecutionMode::Runtime => {
            if let Some(ref runtime_config) = endpoint_config.runtime {
                state.runtime_manager.handle_request(runtime_config, &request_data_json).await
            } else {
                // Fallback to mock if no runtime configured
                let mock_result = state.mock_handler.handle_request(&endpoint_name, endpoint_config, &request_data).await?;
                Ok(serde_json::to_string(&mock_result).map_err(|e| BackworksError::Json(e))?)
            }
        }
        ExecutionMode::Database => {
            if let Some(ref db_manager) = state.database_manager {
                if let Some(ref db_config) = endpoint_config.database {
                    // Convert EndpointDatabaseConfig to DatabaseConfig for now
                    let full_db_config = crate::config::DatabaseConfig {
                        db_type: "sqlite".to_string(), // Default type
                        connection_string: None,
                        connection_string_env: Some("DATABASE_URL".to_string()),
                        pool: None,
                        databases: None,
                    };
                    db_manager.handle_request(&method, &full_db_config, &request_data_json).await
                } else {
                    Err(BackworksError::config("Database mode requires database configuration"))
                }
            } else {
                Err(BackworksError::config("Database mode requires database manager"))
            }
        }
        ExecutionMode::Proxy => {
            if let Some(ref proxy_handler) = state.proxy_handler {
                if let Some(_proxy_config) = &endpoint_config.proxy {
                    // Convert RequestData to HTTP Request<Body>
                    let method = request_data.method.parse::<Method>()
                        .map_err(|e| BackworksError::Proxy(format!("Invalid HTTP method: {}", e)))?;
                    
                    let mut request_builder = Request::builder()
                        .method(method)
                        .uri(format!("{}?{}", 
                            request_data.path_params.get("path").unwrap_or(&"/".to_string()),
                            serde_urlencoded::to_string(&request_data.query_params).unwrap_or_default()
                        ));
                    
                    // Add headers
                    for (key, value) in &request_data.headers {
                        request_builder = request_builder.header(key, value);
                    }
                    
                    // Create request body
                    let body_bytes = if let Some(ref body_value) = request_data.body {
                        serde_json::to_vec(body_value).unwrap_or_default()
                    } else {
                        Vec::new()
                    };
                    
                    let request = request_builder
                        .body(Body::from(body_bytes))
                        .map_err(|e| BackworksError::Proxy(format!("Failed to build request: {}", e)))?;
                    
                    // Proxy the request and convert response to string
                    match proxy_handler.handle_request(request).await {
                        Ok(response) => {
                            let status = response.status();
                            let headers = response.headers().clone();
                            let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await
                                .map_err(|e| BackworksError::Proxy(format!("Failed to read proxy response: {}", e)))?;
                            
                            // For now, return a JSON response with the proxied data
                            let response_json = serde_json::json!({
                                "status": status.as_u16(),
                                "headers": headers.iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect::<std::collections::HashMap<_, _>>(),
                                "body": String::from_utf8_lossy(&body_bytes)
                            });
                            
                            Ok(response_json.to_string())
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    Err(BackworksError::config("Proxy mode requires proxy configuration"))
                }
            } else {
                Err(BackworksError::config("Proxy mode requires proxy handler"))
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
            // Parse the JSON string response to a Value for proper JSON response
            let json_value: serde_json::Value = serde_json::from_str(&response)
                .unwrap_or_else(|_| serde_json::json!({"response": response}));
            Ok((StatusCode::OK, Json(json_value)))
        },
        Err(e) => {
            error!("Request handling error: {}", e);
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()}))
            ))
        }
    }
}

// Health check endpoint
async fn health_check() -> Json<Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// Metrics endpoint
async fn metrics_handler() -> String {
    // Simulate metrics collection
    format!(
        "# HELP backworks_requests_total Total number of requests\n\
         # TYPE backworks_requests_total counter\n\
         backworks_requests_total {}\n",
        42 // Simulated request count
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestData {
    pub method: String,
    pub path_params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    #[serde(skip)] // HeaderMap doesn't implement Serialize
    pub headers: HeaderMap,
    pub body: Option<Value>,
}
