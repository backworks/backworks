use crate::config::DashboardConfig;
use crate::error::{BackworksResult, BackworksError};
use axum::{
    extract::{ws::WebSocket, ws::Message, WebSocketUpgrade},
    response::{Response, IntoResponse},
    routing::{get, Router},
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    pub node_type: String, // "endpoint", "handler", "database", "external"
    pub label: String,
    pub position: Position,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    pub edge_type: String, // "request", "response", "data_flow"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureDiagram {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardEvent {
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}

pub struct Dashboard {
    config: DashboardConfig,
    metrics: Arc<RwLock<HashMap<String, EndpointMetrics>>>,
    system_metrics: Arc<RwLock<SystemMetrics>>,
    architecture: Arc<RwLock<ArchitectureDiagram>>,
    event_sender: broadcast::Sender<DashboardEvent>,
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
            architecture: Arc::new(RwLock::new(ArchitectureDiagram {
                nodes: Vec::new(),
                edges: Vec::new(),
                timestamp: chrono::Utc::now(),
            })),
            event_sender,
            start_time: chrono::Utc::now(),
        }
    }

    pub fn router(&self) -> Router {
        let dashboard_state = DashboardState {
            metrics: self.metrics.clone(),
            system_metrics: self.system_metrics.clone(),
            architecture: self.architecture.clone(),
            event_sender: self.event_sender.clone(),
        };

        Router::new()
            .route("/", get(serve_dashboard))
            .route("/dashboard", get(serve_full_dashboard))
            .route("/api/metrics", get(get_metrics))
            .route("/api/system", get(get_system_metrics))
            .route("/api/architecture", get(get_architecture))
            .route("/ws", get(websocket_handler))
            .with_state(dashboard_state)
    }

    pub async fn start(&self) -> BackworksResult<()> {
        tracing::info!("Starting dashboard on port {}", self.config.port);
        
        // Start background tasks
        self.start_metrics_collector().await;
        self.start_architecture_updater().await;
        
        // Create and start the dashboard HTTP server
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

        // Send real-time event
        let event = DashboardEvent {
            event_type: "request".to_string(),
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({
                "method": method,
                "path": path,
                "response_time": response_time,
                "status_code": status_code
            }),
        };
        
        let _ = self.event_sender.send(event);
        
        Ok(())
    }

    pub async fn update_architecture(&self, nodes: Vec<FlowNode>, edges: Vec<FlowEdge>) -> BackworksResult<()> {
        let mut architecture = self.architecture.write().await;
        architecture.nodes = nodes;
        architecture.edges = edges;
        architecture.timestamp = chrono::Utc::now();

        // Send real-time event
        let event = DashboardEvent {
            event_type: "architecture_update".to_string(),
            timestamp: chrono::Utc::now(),
            data: serde_json::to_value(&*architecture)?,
        };
        
        let _ = self.event_sender.send(event);
        
        Ok(())
    }

    async fn start_metrics_collector(&self) {
        let system_metrics = self.system_metrics.clone();
        let start_time = self.start_time;
        let event_sender = self.event_sender.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                let mut metrics = system_metrics.write().await;
                metrics.uptime = (chrono::Utc::now() - start_time).num_seconds() as u64;
                
                // In a real implementation, these would collect actual system metrics
                metrics.memory_usage = get_memory_usage();
                metrics.cpu_usage = get_cpu_usage();
                
                let event = DashboardEvent {
                    event_type: "system_metrics".to_string(),
                    timestamp: chrono::Utc::now(),
                    data: serde_json::to_value(&*metrics).unwrap(),
                };
                
                let _ = event_sender.send(event);
            }
        });
    }

    async fn start_architecture_updater(&self) {
        let architecture = self.architecture.clone();
        let event_sender = self.event_sender.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Auto-update architecture based on active endpoints and handlers
                let nodes = generate_architecture_nodes().await;
                let edges = generate_architecture_edges(&nodes).await;
                
                {
                    let mut arch = architecture.write().await;
                    arch.nodes = nodes;
                    arch.edges = edges;
                    arch.timestamp = chrono::Utc::now();
                    
                    let event = DashboardEvent {
                        event_type: "architecture_update".to_string(),
                        timestamp: chrono::Utc::now(),
                        data: serde_json::to_value(&*arch).unwrap(),
                    };
                    
                    let _ = event_sender.send(event);
                }
            }
        });
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
    // Try to serve the full dashboard HTML file
    match std::fs::read_to_string("dashboard/index.html") {
        Ok(content) => {
            axum::response::Response::builder()
                .header("content-type", "text/html")
                .body(content.into())
                .unwrap()
        },
        Err(_) => {
            // Try alternative path
            match std::fs::read_to_string("../dashboard/index.html") {
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
    <h1>Dashboard Not Found</h1>
    <p>Could not locate dashboard/index.html</p>
    <p>Please ensure the dashboard files are in the correct location.</p>
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

async fn serve_dashboard() -> Response {
    // Try to serve the dashboard file
    match std::fs::read_to_string("dashboard/index.html") {
        Ok(content) => axum::response::Html(content).into_response(),
        Err(_) => {
            // Fallback to a basic dashboard message
            let fallback_html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Backworks Dashboard</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; background: #1a1a1a; color: #fff; }
        .container { max-width: 800px; margin: 0 auto; text-align: center; }
        .status { background: #333; padding: 20px; border-radius: 8px; margin: 20px 0; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Backworks Dashboard</h1>
        <div class="status">
            <h2>Dashboard is Running</h2>
            <p>The Backworks proxy is active and monitoring API traffic.</p>
            <p>Dashboard UI is available at: <code>/dashboard/index.html</code></p>
        </div>
        <h3>API Endpoints:</h3>
        <ul style="text-align: left; max-width: 400px; margin: 0 auto;">
            <li><a href="/api/metrics" style="color: #4a9eff;">/api/metrics</a> - Endpoint metrics</li>
            <li><a href="/api/system" style="color: #4a9eff;">/api/system</a> - System metrics</li>
            <li><a href="/api/architecture" style="color: #4a9eff;">/api/architecture</a> - Architecture diagram</li>
            <li><strong>/ws</strong> - WebSocket for real-time updates</li>
        </ul>
    </div>
</body>
</html>
            "#;
            axum::response::Html(fallback_html).into_response()
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
