use crate::config::DashboardConfig;
use crate::error::{BackworksResult, BackworksError};
use axum::{
    response::{Response, IntoResponse},
    routing::{get, Router},
    http::{StatusCode, header},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetrics {
    pub path: String,
    pub method: String,
    pub request_count: u64,
    pub avg_response_time: f64,
    pub error_rate: f64,
    pub last_request: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub uptime: u64,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub active_connections: u32,
    pub total_requests: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone)]
pub struct DashboardState {
    pub metrics: Arc<RwLock<HashMap<String, EndpointMetrics>>>,
    pub system_metrics: Arc<RwLock<SystemMetrics>>,
    pub event_sender: broadcast::Sender<String>,
}

pub struct Dashboard {
    config: DashboardConfig,
    metrics: Arc<RwLock<HashMap<String, EndpointMetrics>>>,
    system_metrics: Arc<RwLock<SystemMetrics>>,
    event_sender: broadcast::Sender<String>,
    start_time: chrono::DateTime<chrono::Utc>,
}

impl Dashboard {
    pub fn new(config: DashboardConfig) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            config,
            metrics: Arc::new(RwLock::new(HashMap::new())),
            system_metrics: Arc::new(RwLock::new(SystemMetrics {
                uptime: 0,
                memory_usage: 0,
                cpu_usage: 0.0,
                active_connections: 0,
                total_requests: 0,
                error_count: 0,
            })),
            event_sender,
            start_time: chrono::Utc::now(),
        }
    }

    pub fn router(&self) -> Router {
        let dashboard_state = DashboardState {
            metrics: self.metrics.clone(),
            system_metrics: self.system_metrics.clone(),
            event_sender: self.event_sender.clone(),
        };

        Router::new()
            .route("/", get(serve_qwik_dashboard))
            .route("/api/system", get(get_system_info))
            .route("/api/metrics", get(get_api_metrics))
            .route("/build/*file", get(serve_static_files))
            .route("/assets/*file", get(serve_static_files))
            .fallback(serve_static_files)
            .with_state(dashboard_state)
    }

    pub async fn start(&self) -> BackworksResult<()> {
        tracing::info!("Starting dashboard on port {}", self.config.port);
        
        let app = self.router();
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.config.port))
            .await
            .map_err(|e| BackworksError::Config(format!("Failed to bind dashboard to port {}: {}", self.config.port, e)))?;
            
        tracing::info!("Dashboard server listening on http://0.0.0.0:{}", self.config.port);
        
        axum::serve(listener, app)
            .await
            .map_err(|e| BackworksError::Config(format!("Dashboard server error: {}", e)))?;
        
        Ok(())
    }

    pub async fn record_request(
        &self,
        method: &str,
        path: &str,
        response_time: f64,
        status_code: u16,
    ) -> BackworksResult<()> {
        let key = format!("{} {}", method, path);
        let mut metrics = self.metrics.write().await;
        
        if let Some(endpoint_metrics) = metrics.get_mut(&key) {
            endpoint_metrics.request_count += 1;
            endpoint_metrics.avg_response_time = 
                (endpoint_metrics.avg_response_time + response_time) / 2.0;
            
            if status_code >= 400 {
                endpoint_metrics.error_rate = 
                    (endpoint_metrics.error_rate + 1.0) / endpoint_metrics.request_count as f64;
            }
            
            endpoint_metrics.last_request = chrono::Utc::now();
        } else {
            metrics.insert(key.clone(), EndpointMetrics {
                path: path.to_string(),
                method: method.to_string(),
                request_count: 1,
                avg_response_time: response_time,
                error_rate: if status_code >= 400 { 1.0 } else { 0.0 },
                last_request: chrono::Utc::now(),
            });
        }

        // Update system metrics
        let mut system_metrics = self.system_metrics.write().await;
        system_metrics.total_requests += 1;
        if status_code >= 400 {
            system_metrics.error_count += 1;
        }
        
        Ok(())
    }
}

// Route handlers for the Qwik dashboard
async fn serve_qwik_dashboard() -> Response {
    match std::fs::read_to_string("dashboard/dist/index.html") {
        Ok(content) => {
            axum::response::Response::builder()
                .header("content-type", "text/html")
                .body(content.into())
                .unwrap()
        },
        Err(e) => {
            tracing::error!("Failed to load Qwik dashboard: {}. Please run 'npm run build' in the dashboard directory.", e);
            axum::response::Response::builder()
                .status(503)
                .header("content-type", "text/html")
                .body("<h1>Dashboard Unavailable</h1><p>Please build the dashboard first: <code>cd dashboard && npm run build</code></p>".into())
                .unwrap()
        }
    }
}

async fn get_system_info(
    axum::extract::State(state): axum::extract::State<DashboardState>,
) -> Json<serde_json::Value> {
    let system_metrics = state.system_metrics.read().await;
    
    // Calculate uptime (simplified)
    let uptime_secs = chrono::Utc::now().timestamp() - (chrono::Utc::now().timestamp() - 300); // Mock for now
    let uptime = format!("{}m {}s", uptime_secs / 60, uptime_secs % 60);
    
    Json(serde_json::json!({
        "uptime": uptime,
        "total_requests": system_metrics.total_requests,
        "active_connections": system_metrics.active_connections,
        "cpu_usage": system_metrics.cpu_usage,
        "memory_usage": system_metrics.memory_usage,
        "status": "Running"
    }))
}

async fn get_api_metrics(
    axum::extract::State(state): axum::extract::State<DashboardState>,
) -> Json<Vec<serde_json::Value>> {
    let metrics = state.metrics.read().await;
    let endpoint_metrics = metrics.values().map(|m| {
        let last_accessed = match chrono::Utc::now().signed_duration_since(m.last_request).num_minutes() {
            0 => "Just now".to_string(),
            n if n < 60 => format!("{} minutes ago", n),
            n => format!("{} hours ago", n / 60),
        };
        
        serde_json::json!({
            "method": m.method,
            "path": m.path,
            "request_count": m.request_count,
            "avg_response_time": m.avg_response_time,
            "last_accessed": last_accessed
        })
    }).collect();
    
    Json(endpoint_metrics)
}

async fn serve_static_files(
    uri: axum::http::Uri,
) -> impl IntoResponse {
    let path = uri.path();
    
    // Check if this is a static file by extension or path
    let is_static_file = path.ends_with(".css") || path.ends_with(".js") || 
                        path.ends_with(".wasm") || path.ends_with(".png") || 
                        path.ends_with(".jpg") || path.ends_with(".jpeg") || 
                        path.ends_with(".svg") || path.ends_with(".ico") ||
                        path.starts_with("/build/") || path.starts_with("/assets/");
    
    if !is_static_file {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }
    
    // Map paths to dashboard/dist/ directory
    let file_path = format!("dashboard/dist{}", path);
    
    // Determine content type
    let content_type = match std::path::Path::new(&file_path)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        Some("js") => "application/javascript",
        Some("css") => "text/css",
        Some("html") => "text/html",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    };
    
    // Serve the file
    match std::fs::read(&file_path) {
        Ok(content) => {
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, content_type)],
                content,
            ).into_response()
        }
        Err(_) => {
            (StatusCode::NOT_FOUND, "File not found").into_response()
        }
    }
}
