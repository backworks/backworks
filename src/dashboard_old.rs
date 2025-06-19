use crate::config::DashboardConfig;
use crate::error::{BackworksResult, BackworksError};
use axum::{
    extract::{ws::WebSocket, ws::Message, WebSocketUpgrade},
    response::{Response, IntoResponse},
    routing::{get, Router},
    http::{StatusCode, header},
    Json,
};
use futures::{stream::StreamExt, SinkExt};
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
    match std::fs::read_to_string("studio/dist/index.html") {
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
    
    tracing::info!("Serving static file request for path: {}", path);
    
    // Check if this is a static file by extension or path
    let is_static_file = path.ends_with(".css") || path.ends_with(".js") || 
                        path.ends_with(".wasm") || path.ends_with(".png") || 
                        path.ends_with(".jpg") || path.ends_with(".jpeg") || 
                        path.ends_with(".svg") || path.ends_with(".ico") ||
                        path.starts_with("/build/") || path.starts_with("/assets/");
    
    if !is_static_file {
        tracing::info!("Path {} not recognized as static file", path);
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }
    
    // Map paths to studio/dist/ directory
    let file_path = format!("studio/dist{}", path);
    
    tracing::info!("Mapped path {} to file_path: {}", path, file_path);
    
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
            tracing::info!("Successfully served file: {}", file_path);
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, content_type)],
                content,
            ).into_response()
        }
        Err(e) => {
            tracing::warn!("File not found: {} - {}", file_path, e);
            (StatusCode::NOT_FOUND, "File not found").into_response()
        }
    }
}
            }),
        };
        
        let _ = self.event_sender.send(event);
        Ok(())
    }

    // Background task to update system metrics periodically
    pub fn start_system_monitoring(&self) {
        let metrics = self.metrics.clone();
        let system_metrics = self.system_metrics.clone();
        let event_sender = self.event_sender.clone();
        let start_time = self.start_time;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                // Update system metrics
                let mut system_metrics_guard = system_metrics.write().await;
                system_metrics_guard.cpu_usage = get_cpu_usage();
                system_metrics_guard.memory_usage = get_memory_usage();
                system_metrics_guard.uptime = (chrono::Utc::now() - start_time).num_seconds() as u64;
                
                // Calculate derived metrics
                let metrics_guard = metrics.read().await;
                let total_requests: u64 = metrics_guard.values().map(|m| m.request_count).sum();
                
                system_metrics_guard.total_requests = total_requests;
                drop(metrics_guard);
                
                // Send system metrics update
                let event = DashboardEvent {
                    event_type: "system_metrics".to_string(),
                    timestamp: chrono::Utc::now(),
                    data: serde_json::to_value(&*system_metrics_guard).unwrap_or_default(),
                };
                
                let _ = event_sender.send(event);
                drop(system_metrics_guard);
            }
        });
    }

    // Enhanced metrics APIs
    pub async fn get_performance_summary(&self) -> BackworksResult<serde_json::Value> {
        let metrics = self.metrics.read().await;
        let system_metrics = self.system_metrics.read().await;
        
        let total_requests: u64 = metrics.values().map(|m| m.request_count).sum();
        let total_errors: u64 = metrics.values()
            .map(|m| (m.error_rate * m.request_count as f64) as u64)
            .sum();
        let avg_response_time: f64 = if metrics.is_empty() {
            0.0
        } else {
            metrics.values().map(|m| m.avg_response_time).sum::<f64>() / metrics.len() as f64
        };
        
        // Calculate performance grades
        let endpoint_grades: Vec<_> = metrics.values()
            .map(|m| {
                serde_json::json!({
                    "endpoint": format!("{} {}", m.method, m.path),
                    "grade": calculate_performance_grade(m.avg_response_time, 200),
                    "requests": m.request_count,
                    "avg_response_time": m.avg_response_time,
                    "error_rate": m.error_rate
                })
            })
            .collect();
        
        Ok(serde_json::json!({
            "summary": {
                "total_requests": total_requests,
                "total_errors": total_errors,
                "error_rate": if total_requests > 0 { total_errors as f64 / total_requests as f64 } else { 0.0 },
                "avg_response_time": avg_response_time,
                "overall_grade": calculate_performance_grade(avg_response_time, 200),
                "uptime": system_metrics.uptime,
                "cpu_usage": system_metrics.cpu_usage,
                "memory_usage": system_metrics.memory_usage
            },
            "endpoints": endpoint_grades,
            "recommendations": generate_performance_recommendations(&metrics, avg_response_time, total_errors as f64 / total_requests as f64)
        }))
    }

    pub async fn update_architecture(
        &self,
        nodes: Vec<FlowNode>,
        edges: Vec<FlowEdge>,
    ) -> BackworksResult<()> {
        let mut architecture = self.architecture.write().await;
        architecture.nodes = nodes;
        architecture.edges = edges;
        architecture.timestamp = chrono::Utc::now();
        
        // Send architecture update event
        let event = DashboardEvent {
            event_type: "architecture_update".to_string(),
            timestamp: chrono::Utc::now(),
            data: serde_json::to_value(&*architecture).unwrap_or_default(),
        };
        
        let _ = self.event_sender.send(event);
        Ok(())
    }
}

#[derive(Clone)]
struct DashboardState {
    metrics: Arc<RwLock<HashMap<String, EndpointMetrics>>>,
    system_metrics: Arc<RwLock<SystemMetrics>>,
    architecture: Arc<RwLock<ArchitectureDiagram>>,
    event_sender: broadcast::Sender<DashboardEvent>,
}

async fn serve_full_dashboard() -> Response {
    // Try to serve the Leptos dashboard HTML file
    match std::fs::read_to_string("dash/dist/index.html") {
        Ok(content) => {
            axum::response::Response::builder()
                .header("content-type", "text/html")
                .body(content.into())
                .unwrap()
        },
        Err(_) => {
            // Try alternative path
            match std::fs::read_to_string("../dash/dist/index.html") {
                Ok(content) => {
                    axum::response::Response::builder()
                        .header("content-type", "text/html")
                        .body(content.into())
                        .unwrap()
                },
                Err(_) => {
                    axum::response::Response::builder()
                        .status(404)
                        .header("content-type", "text/html")
                        .body(r#"
<html>
<body style="font-family: Arial; background: #1a1a1a; color: #fff; padding: 40px;">
    <h1>Leptos Dashboard Not Found</h1>
    <p>Could not locate dash/dist/index.html</p>
    <p>Please build the Leptos dashboard first:</p>
    <pre style="background: #333; padding: 10px; border-radius: 4px;">cd dash && trunk build</pre>
    <p><a href="/" style="color: #4a9eff;">Back to Status Page</a></p>
</body>
</html>
                        "#.into())
                        .unwrap()
                }
            }
        }
    }
}

#[derive(Serialize)]
struct SimpleSystemInfo {
    uptime: String,
    total_requests: u64,
    active_connections: u32,
    cpu_usage: f32,
    memory_usage: u64,
    status: String,
}

#[derive(Serialize)]
struct SimpleEndpointMetric {
    method: String,
    path: String,
    request_count: u64,
    avg_response_time: f64,
    last_accessed: String,
}

// Simple API endpoints for the new dashboard
async fn get_simple_system_info(
    axum::extract::State(state): axum::extract::State<DashboardState>
) -> impl IntoResponse {
    let system_metrics = state.system_metrics.read().await;
    let uptime_seconds = system_metrics.uptime;
    
    let uptime_str = if uptime_seconds > 86400 { // More than a day
        let days = uptime_seconds / 86400;
        let hours = (uptime_seconds % 86400) / 3600;
        let minutes = (uptime_seconds % 3600) / 60;
        format!("{}d {}h {}m", days, hours, minutes)
    } else if uptime_seconds > 3600 { // More than an hour
        let hours = uptime_seconds / 3600;
        let minutes = (uptime_seconds % 3600) / 60;
        format!("{}h {}m", hours, minutes)
    } else { // Less than an hour
        let minutes = uptime_seconds / 60;
        let seconds = uptime_seconds % 60;
        format!("{}m {}s", minutes, seconds)
    };
    
    let system_info = SimpleSystemInfo {
        uptime: uptime_str,
        total_requests: system_metrics.total_requests,
        active_connections: system_metrics.active_connections,
        cpu_usage: system_metrics.cpu_usage as f32,
        memory_usage: system_metrics.memory_usage,
        status: "Running".to_string(),
    };
    
    Json(system_info)
}

async fn get_simple_api_metrics(
    axum::extract::State(state): axum::extract::State<DashboardState>
) -> impl IntoResponse {
    let metrics = state.metrics.read().await;
    let mut endpoint_metrics = Vec::new();
    
    for (_, metric) in metrics.iter() {
        let last_accessed = {
            let duration = chrono::Utc::now() - metric.last_request;
            if duration.num_minutes() < 1 {
                "Just now".to_string()
            } else if duration.num_minutes() < 60 {
                format!("{} minutes ago", duration.num_minutes())
            } else if duration.num_hours() < 24 {
                format!("{} hours ago", duration.num_hours())
            } else {
                format!("{} days ago", duration.num_days())
            }
        };
        
        endpoint_metrics.push(SimpleEndpointMetric {
            method: metric.method.clone(),
            path: metric.path.clone(),
            request_count: metric.request_count,
            avg_response_time: metric.avg_response_time,
            last_accessed,
        });
    }
    
    // Sort by request count (most popular first)
    endpoint_metrics.sort_by(|a, b| b.request_count.cmp(&a.request_count));
    
    Json(endpoint_metrics)
}

async fn serve_simple_dashboard() -> Response {
    // Serve the Qwik dashboard HTML file
    match std::fs::read_to_string("studio/dist/index.html") {
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

async fn get_metrics(
    axum::extract::State(state): axum::extract::State<DashboardState>,
) -> Json<Vec<EndpointMetrics>> {
    let metrics = state.metrics.read().await;
    Json(metrics.values().cloned().collect())
}

async fn get_system_metrics(
    axum::extract::State(state): axum::extract::State<DashboardState>,
) -> Json<SystemMetrics> {
    let metrics = state.system_metrics.read().await;
    Json(metrics.clone())
}

async fn get_architecture(
    axum::extract::State(state): axum::extract::State<DashboardState>,
) -> Json<ArchitectureDiagram> {
    let architecture = state.architecture.read().await;
    Json(architecture.clone())
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<DashboardState>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: DashboardState) {
    let mut event_receiver = state.event_sender.subscribe();
    let (mut sender, mut receiver) = socket.split();
    
    // Send initial data
    let metrics = state.metrics.read().await;
    let system_metrics = state.system_metrics.read().await;
    let architecture = state.architecture.read().await;
    
    let initial_data = serde_json::json!({
        "type": "initial",
        "metrics": metrics.values().cloned().collect::<Vec<_>>(),
        "system_metrics": *system_metrics,
        "architecture": *architecture
    });
    
    if sender.send(Message::Text(initial_data.to_string())).await.is_err() {
        return;
    }
    
    drop(metrics);
    drop(system_metrics);
    drop(architecture);
    
    // Handle incoming messages and send real-time updates
    let sender_task = tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            let message = serde_json::to_string(&event).unwrap();
            if sender.send(Message::Text(message)).await.is_err() {
                break;
            }
        }
    });
    
    let receiver_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Close(_) => break,
                    _ => {
                        // Handle other message types if needed
                    }
                }
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = sender_task => {},
        _ = receiver_task => {},
    }
}

// Mock system metrics functions - in a real implementation these would use system APIs
fn get_memory_usage() -> u64 {
    // This would use system APIs to get actual memory usage
    1024 * 1024 * 512 // 512 MB mock value
}

fn get_cpu_usage() -> f64 {
    // This would use system APIs to get actual CPU usage
    15.5 // 15.5% mock value
}

async fn generate_architecture_nodes() -> Vec<FlowNode> {
    // This would analyze the current configuration and active handlers
    // to generate an architecture diagram
    vec![
        FlowNode {
            id: "api_gateway".to_string(),
            node_type: "endpoint".to_string(),
            label: "API Gateway".to_string(),
            position: Position { x: 100.0, y: 100.0 },
            metadata: HashMap::new(),
        },
        FlowNode {
            id: "mock_handler".to_string(),
            node_type: "handler".to_string(),
            label: "Mock Handler".to_string(),
            position: Position { x: 300.0, y: 100.0 },
            metadata: HashMap::new(),
        },
    ]
}

async fn generate_architecture_edges(nodes: &[FlowNode]) -> Vec<FlowEdge> {
    if nodes.len() >= 2 {
        vec![
            FlowEdge {
                id: "gateway_to_mock".to_string(),
                source: "api_gateway".to_string(),
                target: "mock_handler".to_string(),
                label: Some("HTTP Request".to_string()),
                edge_type: "request".to_string(),
            }
        ]
    } else {
        Vec::new()
    }
}

// Performance analysis helper functions
fn calculate_performance_grade(response_time: f64, status_code: u16) -> &'static str {
    if status_code >= 500 {
        "F"
    } else if status_code >= 400 {
        "D"
    } else if response_time > 2000.0 {
        "D"
    } else if response_time > 1000.0 {
        "C"
    } else if response_time > 500.0 {
        "B"
    } else if response_time > 200.0 {
        "A"
    } else {
        "A+"
    }
}

fn calculate_trend_indicator(response_time: f64) -> &'static str {
    // This would typically compare against historical data
    // For now, we'll use response time thresholds
    if response_time > 1000.0 {
        "deteriorating"
    } else if response_time > 500.0 {
        "stable"
    } else {
        "improving"
    }
}

fn generate_performance_recommendations(
    metrics: &std::collections::HashMap<String, EndpointMetrics>,
    avg_response_time: f64,
    error_rate: f64,
) -> Vec<serde_json::Value> {
    let mut recommendations = Vec::new();
    
    if avg_response_time > 1000.0 {
        recommendations.push(serde_json::json!({
            "type": "performance",
            "severity": "high",
            "title": "High Average Response Time",
            "description": format!("Average response time is {:.1}ms. Consider optimizing your endpoints.", avg_response_time),
            "action": "Review slow endpoints and implement caching or optimization strategies"
        }));
    }
    
    if error_rate > 0.05 {
        recommendations.push(serde_json::json!({
            "type": "reliability",
            "severity": "medium",
            "title": "High Error Rate",
            "description": format!("Error rate is {:.1}%. Consider improving error handling.", error_rate * 100.0),
            "action": "Review error logs and implement better error handling and monitoring"
        }));
    }
    
    // Find slow endpoints
    let slow_endpoints: Vec<_> = metrics.values()
        .filter(|m| m.avg_response_time > 2000.0)
        .collect();
    
    if !slow_endpoints.is_empty() {
        recommendations.push(serde_json::json!({
            "type": "optimization",
            "severity": "medium",
            "title": "Slow Endpoints Detected",
            "description": format!("{} endpoints have response times > 2s", slow_endpoints.len()),
            "action": "Optimize slow endpoints or implement caching strategies"
        }));
    }
    
    recommendations
}

// Remove duplicate StreamExt import

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dashboard_config() -> DashboardConfig {
        DashboardConfig {
            enabled: true,
            port: 3001,
            features: Some(vec!["metrics".to_string(), "api".to_string()]),
            real_time: None,
            visualization: None,
            access: None,
        }
    }

    #[tokio::test]
    async fn test_dashboard_creation() {
        let config = create_test_dashboard_config();
        let dashboard = Dashboard::new(config);
        
        assert!(dashboard.start().await.is_ok());
    }

    #[tokio::test]
    async fn test_record_request() {
        let config = create_test_dashboard_config();
        let dashboard = Dashboard::new(config);
        
        dashboard.record_request("GET", "/users", 100.0, 200).await.unwrap();
        dashboard.record_request("GET", "/users", 150.0, 200).await.unwrap();
        
        let metrics = dashboard.metrics.read().await;
        let endpoint_metrics = metrics.get("GET /users").unwrap();
        
        assert_eq!(endpoint_metrics.request_count, 2);
        assert_eq!(endpoint_metrics.avg_response_time, 125.0);
        assert_eq!(endpoint_metrics.error_rate, 0.0);
    }

    #[tokio::test]
    async fn test_architecture_update() {
        let config = create_test_dashboard_config();
        let dashboard = Dashboard::new(config);
        
        let nodes = vec![
            FlowNode {
                id: "test_node".to_string(),
                node_type: "endpoint".to_string(),
                label: "Test Endpoint".to_string(),
                position: Position { x: 0.0, y: 0.0 },
                metadata: HashMap::new(),
            }
        ];
        
        let edges = vec![
            FlowEdge {
                id: "test_edge".to_string(),
                source: "source".to_string(),
                target: "target".to_string(),
                label: None,
                edge_type: "request".to_string(),
            }
        ];
        
        dashboard.update_architecture(nodes, edges).await.unwrap();
        
        let architecture = dashboard.architecture.read().await;
        assert_eq!(architecture.nodes.len(), 1);
        assert_eq!(architecture.edges.len(), 1);
    }
}

// Helper function to get performance summary from DashboardState
async fn get_performance_summary_from_state(state: &DashboardState) -> BackworksResult<serde_json::Value> {
    let metrics = state.metrics.read().await;
    let system_metrics = state.system_metrics.read().await;
    
    let total_requests: u64 = metrics.values().map(|m| m.request_count).sum();
    let total_errors: u64 = metrics.values()
        .map(|m| (m.error_rate * m.request_count as f64) as u64)
        .sum();
    let avg_response_time: f64 = if metrics.is_empty() {
        0.0
    } else {
        metrics.values().map(|m| m.avg_response_time).sum::<f64>() / metrics.len() as f64
    };
    
    // Calculate performance grades
    let endpoint_grades: Vec<_> = metrics.values()
        .map(|m| {
            serde_json::json!({
                "endpoint": format!("{} {}", m.method, m.path),
                "grade": calculate_performance_grade(m.avg_response_time, 200),
                "requests": m.request_count,
                "avg_response_time": m.avg_response_time,
                "error_rate": m.error_rate
            })
        })
        .collect();
    
    Ok(serde_json::json!({
        "summary": {
            "total_requests": total_requests,
            "total_errors": total_errors,
            "error_rate": if total_requests > 0 { total_errors as f64 / total_requests as f64 } else { 0.0 },
            "avg_response_time": avg_response_time,
            "overall_grade": calculate_performance_grade(avg_response_time, 200),
            "uptime": system_metrics.uptime,
            "cpu_usage": system_metrics.cpu_usage,
            "memory_usage": system_metrics.memory_usage
        },
        "endpoints": endpoint_grades,
        "recommendations": generate_performance_recommendations(&metrics, avg_response_time, total_errors as f64 / total_requests.max(1) as f64)
    }))
}

async fn serve_static_files(
    uri: axum::http::Uri,
) -> impl IntoResponse {
    let path = uri.path();
    
    tracing::info!("Serving static file request for path: {}", path);
    
    // Check if this is a static file by extension
    let is_static_file = path.ends_with(".css") || path.ends_with(".js") || 
                        path.ends_with(".wasm") || path.ends_with(".png") || 
                        path.ends_with(".jpg") || path.ends_with(".jpeg") || 
                        path.ends_with(".svg") || path.ends_with(".ico");
    
    if !is_static_file && !path.starts_with("/pkg/") && !path.starts_with("/assets/") && !path.starts_with("/build/") {
        tracing::info!("Path {} not recognized as static file", path);
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }
    
    // Map various request paths to the actual file location in studio/dist/
    let file_path = if path.starts_with("/pkg/") {
        // Remove /pkg/ prefix and look in studio/dist/
        let filename = path.strip_prefix("/pkg/").unwrap_or("");
        format!("studio/dist/{}", filename)
    } else if path.starts_with("/assets/") {
        // Assets path
        format!("studio/dist{}", path)
    } else if path.starts_with("/build/") {
        // Qwik build assets
        format!("studio/dist{}", path)
    } else {
        // For root-level requests (CSS, JS, WASM files), look directly in studio/dist/
        let filename = path.strip_prefix("/").unwrap_or("");
        format!("studio/dist/{}", filename)
    };
    
    tracing::info!("Mapped path {} to file_path: {}", path, file_path);
    
    // Determine content type based on file extension
    let content_type = match std::path::Path::new(&file_path)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        Some("js") => "application/javascript",
        Some("wasm") => "application/wasm",
        Some("css") => "text/css",
        Some("html") => "text/html",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    };
    
    // Try to read and serve the file
    match std::fs::read(&file_path) {
        Ok(content) => {
            tracing::debug!("Successfully served file: {}", file_path);
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, content_type)],
                content,
            ).into_response()
        }
        Err(e) => {
            tracing::debug!("Failed to read file {}: {}", file_path, e);
            // Try alternative path
            let alt_path = format!("../{}", file_path);
            tracing::debug!("Trying alternative path: {}", alt_path);
            match std::fs::read(&alt_path) {
                Ok(content) => {
                    tracing::debug!("Successfully served file from alternative path: {}", alt_path);
                    (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, content_type)],
                        content,
                    ).into_response()
                }
                Err(e2) => {
                    tracing::warn!("File not found at {} or {}: {} / {}", file_path, alt_path, e, e2);
                    (StatusCode::NOT_FOUND, "File not found").into_response()
                }
            }
        }
    }
}
