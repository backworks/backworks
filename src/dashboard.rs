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
use std::path::{Path, PathBuf};
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
    #[allow(dead_code)] // TODO: Will be used for displaying uptime in dashboard
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

/// Find the studio directory by looking for it relative to the current working directory
/// or relative to the executable location
fn find_studio_path() -> BackworksResult<PathBuf> {
    // Try current directory + studio first
    let current_studio = Path::new("studio");
    if current_studio.exists() && current_studio.join("dist").exists() {
        return Ok(current_studio.to_path_buf());
    }
    
    // Try parent directory + studio (for when running from examples/)
    let parent_studio = Path::new("../studio");
    if parent_studio.exists() && parent_studio.join("dist").exists() {
        return Ok(parent_studio.to_path_buf());
    }
    
    // Try two levels up + studio (for deeper nesting)
    let grandparent_studio = Path::new("../../studio");
    if grandparent_studio.exists() && grandparent_studio.join("dist").exists() {
        return Ok(grandparent_studio.to_path_buf());
    }
    
    // Try to find it relative to the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // For cargo target/debug/backworks
            let exe_studio = exe_dir.parent().unwrap_or(exe_dir).parent().unwrap_or(exe_dir).join("studio");
            if exe_studio.exists() && exe_studio.join("dist").exists() {
                return Ok(exe_studio);
            }
        }
    }
    
    Err(BackworksError::Server("Studio directory not found. Please ensure the studio is built with 'cd studio && npm run build'".to_string()))
}

// Route handlers for the Qwik dashboard
async fn serve_qwik_dashboard() -> Response {
    let studio_path = match find_studio_path() {
        Ok(path) => path,
        Err(e) => {
            tracing::error!("Failed to find studio directory: {}", e);
            return axum::response::Response::builder()
                .status(503)
                .header("content-type", "text/html")
                .body("<h1>Studio Unavailable</h1><p>Please build the studio first: <code>cd studio && npm run build</code></p>".into())
                .unwrap();
        }
    };
    
    let index_path = studio_path.join("dist/index.html");
    match std::fs::read_to_string(&index_path) {
        Ok(content) => {
            axum::response::Response::builder()
                .header("content-type", "text/html")
                .body(content.into())
                .unwrap()
        },
        Err(e) => {
            tracing::error!("Failed to load studio dashboard from {:?}: {}. Please run 'npm run build' in the studio directory.", index_path, e);
            axum::response::Response::builder()
                .status(503)
                .header("content-type", "text/html")
                .body("<h1>Studio Unavailable</h1><p>Please build the studio first: <code>cd studio && npm run build</code></p>".into())
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
    
    // Find studio path and map to dist directory
    let studio_path = match find_studio_path() {
        Ok(path) => path,
        Err(_) => {
            return (StatusCode::SERVICE_UNAVAILABLE, "Studio not available").into_response();
        }
    };
    
    let file_path = studio_path.join("dist").join(path.strip_prefix("/").unwrap_or(path));
    
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
