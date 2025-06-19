use crate::config::{ProxyConfig, ProxyTarget, LoadBalancingAlgorithm, TransformConfig, BodyTransform};
use crate::capture::CaptureHandler;
use crate::error::{BackworksError, BackworksResult};
use axum::{body::Body, http::{Request, HeaderMap, StatusCode, HeaderName, HeaderValue}, response::Response};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyMetrics {
    pub target_name: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time: f64,
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
    pub is_healthy: bool,
    pub circuit_breaker_state: CircuitBreakerState,
}

#[derive(Debug)]
pub struct ProxyHandler {
    config: ProxyConfig,
    targets: Arc<RwLock<Vec<ProxyTarget>>>,
    metrics: Arc<RwLock<HashMap<String, ProxyMetrics>>>,
    client: Client,
    current_target_index: Arc<RwLock<usize>>,
    capture_handler: Option<CaptureHandler>,
}

impl Clone for ProxyHandler {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            targets: Arc::clone(&self.targets),
            metrics: Arc::clone(&self.metrics),
            client: self.client.clone(),
            current_target_index: Arc::clone(&self.current_target_index),
            capture_handler: self.capture_handler.clone(),
        }
    }
}

impl ProxyHandler {
    pub fn new(config: ProxyConfig) -> Self {
        tracing::info!("Creating ProxyHandler with config: transform_request={:?}, transform_response={:?}", 
                      config.transform_request.is_some(), config.transform_response.is_some());
        
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        // Initialize capture handler if capture is configured
        let capture_handler = if let Some(capture_config) = &config.capture {
            Some(CaptureHandler::new(capture_config.clone()))
        } else {
            None
        };

        Self {
            config,
            targets: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            client,
            current_target_index: Arc::new(RwLock::new(0)),
            capture_handler,
        }
    }

    pub async fn start(&self) -> BackworksResult<()> {
        tracing::info!("Starting proxy handler");
        
        // Initialize targets
        if let Some(targets) = &self.config.targets {
            for target in targets {
                self.add_target(target.clone()).await?;
            }
        } else {
            // Fallback to single target configuration if available
            if let Some(target_url) = &self.config.target {
                let single_target = ProxyTarget {
                    name: "default".to_string(),
                    url: target_url.clone(),
                    weight: Some(1.0),
                    timeout: self.config.timeout.map(std::time::Duration::from_secs),
                    health_check: None,
                    retry_attempts: Some(3),
                    circuit_breaker: None,
                };
                self.add_target(single_target).await?;
            } else {
                return Err(BackworksError::config("Proxy requires either 'target' or 'targets' configuration"));
            }
        }
        
        // Start health checks if enabled
        if self.config.health_checks.unwrap_or(false) {
            self.start_health_checks().await;
        }
        
        Ok(())
    }

    pub async fn add_target(&self, target: ProxyTarget) -> BackworksResult<()> {
        // Validate target URL
        Url::parse(&target.url)
            .map_err(|e| BackworksError::Config(format!("Invalid proxy target URL: {}", e)))?;
        
        let metrics = ProxyMetrics {
            target_name: target.name.clone(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time: 0.0,
            last_health_check: None,
            is_healthy: true,
            circuit_breaker_state: CircuitBreakerState::Closed,
        };
        
        let mut targets = self.targets.write().await;
        targets.push(target.clone());
        
        let mut metrics_map = self.metrics.write().await;
        metrics_map.insert(target.name.clone(), metrics);
        
        tracing::info!("Added proxy target: {} -> {}", target.name, target.url);
        Ok(())
    }

    pub async fn remove_target(&self, target_name: &str) -> BackworksResult<()> {
        let mut targets = self.targets.write().await;
        targets.retain(|t| t.name != target_name);
        
        let mut metrics = self.metrics.write().await;
        metrics.remove(target_name);
        
        tracing::info!("Removed proxy target: {}", target_name);
        Ok(())
    }

    pub async fn proxy_request(&self, mut request: Request<Body>) -> BackworksResult<Response<Body>> {
        let start_time = std::time::Instant::now();
        
        // Apply request transformations FIRST
        request = self.apply_request_transformation(request).await?;
        
        // Extract information from request AFTER transformation
        let method = request.method().as_str();
        let path = request.uri().path();
        let headers = request.headers().clone();
        
        // Select target based on load balancing strategy
        let target = self.select_target(method, path, &headers).await?;
        
        // Build the target URL using the TRANSFORMED request URI
        let target_url = self.build_target_url(&target, request.uri())?;
        
        // Update request URI to the final target URL
        let uri_parts = target_url.as_str().parse::<http::Uri>()
            .map_err(|e| BackworksError::Proxy(format!("Invalid target URI: {}", e)))?;
        *request.uri_mut() = uri_parts;
        
        // Add proxy headers
        self.add_proxy_headers(&mut request, &target).await;
        
        // Convert axum request to reqwest request
        let reqwest_request = self.convert_to_reqwest_request(request, &target_url).await?;
        
        // Execute the request
        let result = self.execute_request(reqwest_request, &target).await;
        let duration = start_time.elapsed();
        
        // Update metrics
        match &result {
            Ok(_) => self.update_metrics(&target.name, true, duration).await,
            Err(_) => self.update_metrics(&target.name, false, duration).await,
        }
        
        match result {
            Ok(mut response) => {
                // Apply response transformations
                response = self.apply_response_transformation(response).await?;
                Ok(response)
            },
            Err(e) => {
                // Return a 502 Bad Gateway error
                Ok(Response::builder()
                    .status(502)
                    .header("content-type", "application/json")
                    .body(Body::from(format!("{{\"error\": \"Proxy error: {}\"}}", e)))
                    .unwrap())
            }
        }
    }

    pub async fn get_metrics(&self) -> Vec<ProxyMetrics> {
        self.metrics.read().await.values().cloned().collect()
    }

    pub async fn get_targets(&self) -> Vec<ProxyTarget> {
        self.targets.read().await.clone()
    }

    pub async fn health_check(&self, target: &ProxyTarget) -> BackworksResult<bool> {
        if let Some(health_check) = &target.health_check {
            let health_url = format!("{}{}", target.url, health_check.path);
            
            let response = self.client
                .get(&health_url)
                .timeout(health_check.timeout)
                .send()
                .await;
            
            match response {
                Ok(resp) => Ok(resp.status().is_success()),
                Err(_) => Ok(false),
            }
        } else {
            // If no health check configured, assume healthy
            Ok(true)
        }
    }

    async fn select_target(&self, method: &str, path: &str, headers_map: &HeaderMap) -> BackworksResult<ProxyTarget> {
        let targets = self.targets.read().await;
        
        if targets.is_empty() {
            return Err(BackworksError::Proxy("No proxy targets available".to_string()));
        }
        
        // Filter healthy targets
        let healthy_targets: Vec<&ProxyTarget> = targets.iter()
            .filter(|target| {
                // Check if target is healthy based on metrics
                let metrics = futures::executor::block_on(self.metrics.read());
                metrics.get(&target.name)
                    .map(|m| m.is_healthy && matches!(m.circuit_breaker_state, CircuitBreakerState::Closed | CircuitBreakerState::HalfOpen))
                    .unwrap_or(true)
            })
            .collect();
        
        if healthy_targets.is_empty() {
            return Err(BackworksError::Proxy("No healthy proxy targets available".to_string()));
        }
        
        // Apply load balancing algorithm
        let selected_target = match self.config.load_balancing.as_ref().map(|lb| &lb.algorithm) {
            Some(LoadBalancingAlgorithm::RoundRobin) => {
                let mut index = self.current_target_index.write().await;
                let target = healthy_targets[*index % healthy_targets.len()];
                *index += 1;
                (*target).clone()
            }
            Some(LoadBalancingAlgorithm::Weighted) => {
                // Simple weighted round-robin implementation
                self.select_weighted_target(&healthy_targets).await
            }
            Some(LoadBalancingAlgorithm::IpHash) => {
                // Hash based on client IP (if available)
                self.select_by_ip_hash(&healthy_targets, headers_map).await
            }
            None => {
                // Default to round-robin
                let mut index = self.current_target_index.write().await;
                let target = healthy_targets[*index % healthy_targets.len()];
                *index += 1;
                (*target).clone()
            }
        };
        
        Ok(selected_target)
    }

    async fn select_weighted_target(&self, targets: &[&ProxyTarget]) -> ProxyTarget {
        let total_weight: f64 = targets.iter()
            .map(|t| t.weight.unwrap_or(1.0))
            .sum();
        
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut random_weight = rng.gen_range(0.0..total_weight);
        
        for target in targets {
            let weight = target.weight.unwrap_or(1.0);
            if random_weight < weight {
                return (*target).clone();
            }
            random_weight -= weight;
        }
        
        (*targets[0]).clone() // Fallback
    }

    async fn select_by_ip_hash(&self, targets: &[&ProxyTarget], headers_map: &HeaderMap) -> ProxyTarget {
        // Try to get client IP from headers or use a default
        let client_ip = headers_map
            .get("x-forwarded-for")
            .or_else(|| headers_map.get("x-real-ip"))
            .and_then(|v| v.to_str().ok())
            .unwrap_or("127.0.0.1");
        
        // Simple hash function
        let hash = client_ip.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        let index = hash as usize % targets.len();
        
        (*targets[index]).clone()
    }

    fn build_target_url(&self, target: &ProxyTarget, original_uri: &http::Uri) -> BackworksResult<Url> {
        let mut target_url = Url::parse(&target.url)
            .map_err(|e| BackworksError::Proxy(format!("Invalid target URL: {}", e)))?;
        
        // Append the original path and query
        target_url.set_path(original_uri.path());
        if let Some(query) = original_uri.query() {
            target_url.set_query(Some(query));
        }
        
        Ok(target_url)
    }

    async fn add_proxy_headers(&self, request: &mut Request<Body>, target: &ProxyTarget) {
        let headers = request.headers_mut();
        
        // Add X-Forwarded-* headers
        if !headers.contains_key("x-forwarded-for") {
            headers.insert("x-forwarded-for", "127.0.0.1".parse().unwrap());
        }
        
        if !headers.contains_key("x-forwarded-proto") {
            headers.insert("x-forwarded-proto", "http".parse().unwrap());
        }
        
        // Add custom headers from config
        if let Some(config_headers) = &self.config.headers {
            for (key, value) in config_headers {
                if let (Ok(header_name), Ok(header_value)) = (
                    key.parse::<http::HeaderName>(),
                    value.parse::<http::HeaderValue>()
                ) {
                    headers.insert(header_name, header_value);
                }
            }
        }
        
        // Update Host header to target
        if let Ok(target_url) = Url::parse(&target.url) {
            if let Some(host) = target_url.host_str() {
                let host_value = if let Some(port) = target_url.port() {
                    format!("{}:{}", host, port)
                } else {
                    host.to_string()
                };
                headers.insert("host", host_value.parse().unwrap());
            }
        }
    }

    async fn convert_to_reqwest_request(&self, request: Request<Body>, target_url: &Url) -> BackworksResult<reqwest::Request> {
        let method_str = request.method().as_str();
        let method = reqwest::Method::from_bytes(method_str.as_bytes())
            .map_err(|e| BackworksError::Proxy(format!("Invalid HTTP method: {}", e)))?;
        
        let headers = request.headers().clone();
        let body = axum::body::to_bytes(request.into_body(), usize::MAX).await
            .map_err(|e| BackworksError::Proxy(format!("Failed to read request body: {}", e)))?;
        
        let mut reqwest_request = self.client
            .request(method, target_url.clone())
            .body(body.to_vec());
        
        // Copy headers - convert between http crate versions
        for (name, value) in headers.iter() {
            let name_str = name.as_str();
            let value_bytes = value.as_bytes();
            if let Ok(value_str) = std::str::from_utf8(value_bytes) {
                reqwest_request = reqwest_request.header(name_str, value_str);
            }
        }
        
        reqwest_request.build()
            .map_err(|e| BackworksError::Proxy(format!("Failed to build request: {}", e)))
    }

    async fn execute_request(&self, request: reqwest::Request, target: &ProxyTarget) -> BackworksResult<Response<Body>> {
        let timeout = target.timeout.unwrap_or(Duration::from_secs(30));
        let max_retries = target.retry_attempts.unwrap_or(0);
        
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            if attempt > 0 {
                tracing::warn!("Retrying request to {} (attempt {})", target.name, attempt + 1);
                tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
            }
            
            let cloned_request = request.try_clone()
                .ok_or_else(|| BackworksError::Proxy("Failed to clone request for retry".to_string()))?;
            
            match self.client.execute(cloned_request).await {
                Ok(response) => {
                    return self.convert_from_reqwest_response(response).await;
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        continue;
                    }
                }
            }
        }
        
        Err(BackworksError::Proxy(format!(
            "Request failed after {} attempts: {}",
            max_retries + 1,
            last_error.unwrap()
        )))
    }

    async fn convert_from_reqwest_response(&self, response: reqwest::Response) -> BackworksResult<Response<Body>> {
        let status = response.status();
        let headers = response.headers().clone();
        let body_bytes = response.bytes().await
            .map_err(|e| BackworksError::Proxy(format!("Failed to read response body: {}", e)))?;
        
        // Convert status code between crate versions
        let status_code = http::StatusCode::from_u16(status.as_u16())
            .map_err(|e| BackworksError::Proxy(format!("Invalid status code: {}", e)))?;
        
        let mut builder = Response::builder().status(status_code);
        
        // Copy headers - convert between http crate versions
        for (name, value) in headers.iter() {
            let name_str = name.as_str();
            let value_bytes = value.as_bytes();
            if let Ok(value_str) = std::str::from_utf8(value_bytes) {
                builder = builder.header(name_str, value_str);
            }
        }
        
        builder.body(Body::from(body_bytes.to_vec()))
            .map_err(|e| BackworksError::Proxy(format!("Failed to build response: {}", e)))
    }

    async fn update_metrics(&self, target_name: &str, success: bool, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(target_name) {
            metric.total_requests += 1;
            
            if success {
                metric.successful_requests += 1;
            } else {
                metric.failed_requests += 1;
            }
            
            // Update average response time
            let total_time = metric.avg_response_time * (metric.total_requests - 1) as f64 + duration.as_millis() as f64;
            metric.avg_response_time = total_time / metric.total_requests as f64;
        }
    }

    async fn start_health_checks(&self) {
        let targets = self.targets.clone();
        let metrics = self.metrics.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let target_list = targets.read().await.clone();
                for target in target_list {
                    if let Some(health_check) = &target.health_check {
                        let health_url = format!("{}{}", target.url, health_check.path);
                        
                        let client = Client::new();
                        let is_healthy = match client
                            .get(&health_url)
                            .timeout(health_check.timeout)
                            .send()
                            .await
                        {
                            Ok(resp) => resp.status().is_success(),
                            Err(_) => false,
                        };
                        
                        // Update metrics
                        let mut metrics_map = metrics.write().await;
                        if let Some(metric) = metrics_map.get_mut(&target.name) {
                            metric.is_healthy = is_healthy;
                            metric.last_health_check = Some(chrono::Utc::now());
                        }
                        
                        tracing::debug!("Health check for {}: {}", target.name, if is_healthy { "healthy" } else { "unhealthy" });
                    }
                }
            }
        });
    }

    /// Handle HTTP proxy request - this is the main proxy method that should be used
    pub async fn handle_request(&self, request: Request<Body>) -> BackworksResult<Response<Body>> {
        let start_time = std::time::Instant::now();
        
        // If capture is enabled, capture the request and response
        if let Some(ref capture_handler) = self.capture_handler {
            let mut headers = std::collections::HashMap::new();
            for (key, value) in request.headers() {
                if let Ok(value_str) = value.to_str() {
                    headers.insert(key.to_string(), value_str.to_string());
                }
            }
            
            let path = request.uri().path().to_string();
            let method = request.method().to_string();
            let query_params = request.uri().query()
                .map(|q| serde_urlencoded::from_str::<HashMap<String, String>>(q).unwrap_or_default())
                .unwrap_or_default();
            
            // Read request body for capture
            let (parts, body) = request.into_parts();
            let body_bytes = axum::body::to_bytes(body, usize::MAX).await
                .map_err(|e| BackworksError::Proxy(format!("Failed to read request body: {}", e)))?;
            
            let body_value = if !body_bytes.is_empty() {
                match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                    Ok(json) => Some(json),
                    Err(_) => {
                        // If not JSON, store as string
                        match String::from_utf8(body_bytes.to_vec()) {
                            Ok(text) => Some(serde_json::Value::String(text)),
                            Err(_) => None, // Skip binary data
                        }
                    }
                }
            } else {
                None
            };
            
            // Reconstruct request with body
            let request = Request::from_parts(parts, Body::from(body_bytes.to_vec()));
            
            let request_id = capture_handler.capture_request(
                method,
                path,
                headers,
                query_params,
                body_value,
            ).await.unwrap_or(uuid::Uuid::nil());
            
            // Proxy the request
            let proxy_result = self.proxy_request(request).await;
            
            match proxy_result {
                Ok(response) => {
                    let duration = start_time.elapsed();
                    
                    // Capture the response if we have a valid request ID
                    if request_id != uuid::Uuid::nil() {
                        let (parts, body) = response.into_parts();
                        let body_bytes = axum::body::to_bytes(body, usize::MAX).await
                            .map_err(|e| BackworksError::Proxy(format!("Failed to read response body: {}", e)))?;
                        
                        let mut response_headers = std::collections::HashMap::new();
                        for (key, value) in &parts.headers {
                            if let Ok(value_str) = value.to_str() {
                                response_headers.insert(key.to_string(), value_str.to_string());
                            }
                        }
                        
                        let response_body = if !body_bytes.is_empty() {
                            match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                                Ok(json) => Some(json),
                                Err(_) => {
                                    match String::from_utf8(body_bytes.to_vec()) {
                                        Ok(text) => Some(serde_json::Value::String(text)),
                                        Err(_) => None,
                                    }
                                }
                            }
                        } else {
                            None
                        };
                        
                        let _ = capture_handler.capture_response(
                            request_id,
                            parts.status.as_u16(),
                            response_headers,
                            response_body,
                            duration,
                        ).await;
                        
                        tracing::debug!("Captured proxy request/response pair");
                        
                        // Reconstruct response
                        Ok(Response::from_parts(parts, Body::from(body_bytes.to_vec())))
                    } else {
                        Ok(response)
                    }
                }
                Err(e) => Err(e),
            }
        } else {
            // No capture, just proxy the request
            self.proxy_request(request).await
        }
    }

    /// Start a capture session if capture is enabled
    pub async fn start_capture_session(&self, session_name: String) -> BackworksResult<Option<uuid::Uuid>> {
        if let Some(ref capture_handler) = self.capture_handler {
            let session_id = capture_handler.start_session(session_name).await?;
            tracing::info!("Started proxy capture session: {}", session_id);
            Ok(Some(session_id))
        } else {
            Ok(None)
        }
    }

    /// Stop a capture session
    pub async fn stop_capture_session(&self, session_id: uuid::Uuid) -> BackworksResult<()> {
        if let Some(ref capture_handler) = self.capture_handler {
            capture_handler.stop_session(session_id).await?;
            tracing::info!("Stopped proxy capture session: {}", session_id);
        }
        Ok(())
    }

    /// Export captured data from proxy
    pub async fn export_captured_data(&self, session_id: uuid::Uuid, format: &str) -> BackworksResult<Option<String>> {
        if let Some(ref capture_handler) = self.capture_handler {
            let exported = capture_handler.export_session(session_id, format).await?;
            Ok(Some(exported))
        } else {
            Ok(None)
        }
    }

    /// Generate API configuration from captured proxy data
    pub async fn generate_api_config(&self, session_id: uuid::Uuid) -> BackworksResult<Option<String>> {
        if let Some(ref capture_handler) = self.capture_handler {
            let config = capture_handler.generate_api_from_capture(session_id).await?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    /// Handle request with ProxyConfig and RequestData (for compatibility)
    pub async fn handle_request_data(&self, _config: &ProxyConfig, request_data: &crate::server::RequestData) -> BackworksResult<String> {
        // Convert RequestData to HTTP Request<Body>
        let method = request_data.method.parse::<http::Method>()
            .map_err(|e| BackworksError::Proxy(format!("Invalid HTTP method: {}", e)))?;
        
        // Build URI from path_params and query_params
        let default_path = "/".to_string();
        let path = request_data.path_params.get("path").unwrap_or(&default_path);
        let query_string = if !request_data.query_params.is_empty() {
            format!("?{}", serde_urlencoded::to_string(&request_data.query_params).unwrap_or_default())
        } else {
            String::new()
        };
        
        let uri = format!("{}{}", path, query_string);
        
        let mut request_builder = Request::builder()
            .method(method)
            .uri(uri);
        
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
        
        // Use the existing handle_request method
        match self.handle_request(request).await {
            Ok(response) => {
                let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await
                    .map_err(|e| BackworksError::Proxy(format!("Failed to read proxy response: {}", e)))?;
                
                // Return raw response body for production use
                Ok(String::from_utf8_lossy(&body_bytes).to_string())
            }
            Err(e) => Err(e),
        }
    }

    // Legacy method for testing - returns wrapped response with metadata
    pub async fn handle_request_data_with_metadata(&self, config: &ProxyConfig, request_data: &crate::server::RequestData) -> BackworksResult<String> {
        // Convert RequestData to HTTP Request<Body>
        let method = request_data.method.parse::<http::Method>()
            .map_err(|e| BackworksError::Proxy(format!("Invalid HTTP method: {}", e)))?;
        
        // Build URI from path_params and query_params
        let default_path = "/".to_string();
        let path = request_data.path_params.get("path").unwrap_or(&default_path);
        let query_string = if !request_data.query_params.is_empty() {
            format!("?{}", serde_urlencoded::to_string(&request_data.query_params).unwrap_or_default())
        } else {
            String::new()
        };
        
        let uri = format!("{}{}", path, query_string);
        
        let mut request_builder = Request::builder()
            .method(method)
            .uri(uri);
        
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
        
        // Use the existing handle_request method
        match self.handle_request(request).await {
            Ok(response) => {
                let status = response.status();
                let headers = response.headers().clone();
                let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await
                    .map_err(|e| BackworksError::Proxy(format!("Failed to read proxy response: {}", e)))?;
                
                // Return a JSON response with the proxied data (for compatibility with tests)
                let response_json = serde_json::json!({
                    "proxied": true,
                    "target": config.target,
                    "method": request_data.method,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "status": status.as_u16(),
                    "headers": headers.iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect::<std::collections::HashMap<_, _>>(),
                    "body": String::from_utf8_lossy(&body_bytes)
                });
                
                Ok(response_json.to_string())
            }
            Err(e) => Err(e),
        }
    }

    // Transformation methods
    async fn apply_request_transformation(&self, mut request: Request<Body>) -> BackworksResult<Request<Body>> {
        if let Some(transform_config) = &self.config.transform_request {
            tracing::info!("Applying request transformations");
            request = self.transform_request(request, transform_config).await?;
        } else {
            tracing::debug!("No request transformations configured");
        }
        Ok(request)
    }

    async fn apply_response_transformation(&self, mut response: Response<Body>) -> BackworksResult<Response<Body>> {
        if let Some(transform_config) = &self.config.transform_response {
            tracing::info!("Applying response transformations");
            response = self.transform_response(response, transform_config).await?;
        } else {
            tracing::debug!("No response transformations configured");
        }
        Ok(response)
    }

    async fn transform_request(&self, mut request: Request<Body>, config: &TransformConfig) -> BackworksResult<Request<Body>> {
        tracing::info!("Starting request transformation");
        
        // Transform headers
        if let Some(add_headers) = &config.add_headers {
            tracing::info!("Adding request headers: {:?}", add_headers);
            let headers = request.headers_mut();
            for (key, value) in add_headers {
                if let (Ok(name), Ok(value)) = (key.parse::<HeaderName>(), value.parse::<HeaderValue>()) {
                    headers.insert(name, value);
                }
            }
        }
        
        if let Some(remove_headers) = &config.remove_headers {
            let headers = request.headers_mut();
            for key in remove_headers {
                if let Ok(name) = key.parse::<HeaderName>() {
                    headers.remove(&name);
                }
            }
        }

        // Transform path
        if let Some(path_config) = &config.path_rewrite {
            tracing::info!("Applying path transformation: {:?}", path_config);
            let uri = request.uri();
            let mut new_path = uri.path().to_string();
            tracing::info!("Original path: {}", new_path);
            
            if let Some(prefix) = &path_config.add_prefix {
                new_path = format!("{}{}", prefix, new_path);
                tracing::info!("After add_prefix: {}", new_path);
            }
            
            if let Some(prefix) = &path_config.strip_prefix {
                if new_path.starts_with(prefix) {
                    new_path = new_path[prefix.len()..].to_string();
                    tracing::info!("After strip_prefix: {}", new_path);
                }
            }
            
            if let Some(pattern_replace) = &path_config.pattern_replace {
                for replace_rule in pattern_replace {
                    // Simple string replacement for now (could be enhanced with regex)
                    new_path = new_path.replace(&replace_rule.pattern, &replace_rule.replacement);
                    tracing::info!("After pattern replace: {}", new_path);
                }
            }
            
            // Build new URI
            let mut new_uri = format!("{}", new_path);
            if let Some(query) = uri.query() {
                new_uri.push('?');
                new_uri.push_str(query);
            }
            tracing::info!("Final transformed URI: {}", new_uri);
            
            *request.uri_mut() = new_uri.parse()
                .map_err(|e| BackworksError::Proxy(format!("Invalid transformed URI: {}", e)))?;
        }

        // Transform query parameters
        if let Some(query_config) = &config.query_transform {
            let uri = request.uri().clone();
            let mut query_params: HashMap<String, String> = HashMap::new();
            
            // Parse existing query parameters
            if let Some(query) = uri.query() {
                for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
                    query_params.insert(key.into_owned(), value.into_owned());
                }
            }
            
            // Add parameters
            if let Some(add_params) = &query_config.add_params {
                for (key, value) in add_params {
                    query_params.insert(key.clone(), value.clone());
                }
            }
            
            // Remove parameters
            if let Some(remove_params) = &query_config.remove_params {
                for key in remove_params {
                    query_params.remove(key);
                }
            }
            
            // Rename parameters
            if let Some(rename_params) = &query_config.rename_params {
                for (old_key, new_key) in rename_params {
                    if let Some(value) = query_params.remove(old_key) {
                        query_params.insert(new_key.clone(), value);
                    }
                }
            }
            
            // Rebuild URI with new query parameters
            let mut new_uri = uri.path().to_string();
            if !query_params.is_empty() {
                new_uri.push('?');
                let query_string = serde_urlencoded::to_string(&query_params)
                    .map_err(|e| BackworksError::Proxy(format!("Failed to encode query parameters: {}", e)))?;
                new_uri.push_str(&query_string);
            }
            
            *request.uri_mut() = new_uri.parse()
                .map_err(|e| BackworksError::Proxy(format!("Invalid transformed URI: {}", e)))?;
        }

        // Transform body
        if let Some(body_config) = &config.body_transform {
            let (parts, body) = request.into_parts();
            let body_bytes = axum::body::to_bytes(body, usize::MAX).await
                .map_err(|e| BackworksError::Proxy(format!("Failed to read request body: {}", e)))?;
            
            let transformed_body = self.transform_body_content(&body_bytes, body_config).await?;
            
            // Rebuild request with new body
            let new_request = Request::from_parts(parts, Body::from(transformed_body));
            request = new_request;
        }

        Ok(request)
    }

    async fn transform_response(&self, mut response: Response<Body>, config: &TransformConfig) -> BackworksResult<Response<Body>> {
        tracing::info!("Starting response transformation");
        
        // Transform status code
        if let Some(status_code) = config.force_status_code {
            tracing::info!("Setting status code to: {}", status_code);
            *response.status_mut() = StatusCode::from_u16(status_code)
                .map_err(|e| BackworksError::Proxy(format!("Invalid status code: {}", e)))?;
        }
        
        // Transform headers
        if let Some(add_headers) = &config.add_headers {
            tracing::info!("Adding response headers: {:?}", add_headers);
            let headers = response.headers_mut();
            for (key, value) in add_headers {
                if let (Ok(name), Ok(value)) = (key.parse::<HeaderName>(), value.parse::<HeaderValue>()) {
                    headers.insert(name, value);
                }
            }
        }
        
        if let Some(remove_headers) = &config.remove_headers {
            let headers = response.headers_mut();
            for key in remove_headers {
                if let Ok(name) = key.parse::<HeaderName>() {
                    headers.remove(&name);
                }
            }
        }

        // Transform body
        if let Some(body_config) = &config.body_transform {
            let (parts, body) = response.into_parts();
            let body_bytes = axum::body::to_bytes(body, usize::MAX).await
                .map_err(|e| BackworksError::Proxy(format!("Failed to read response body: {}", e)))?;
            
            let transformed_body = self.transform_body_content(&body_bytes, body_config).await?;
            
            // Rebuild response with new body
            let new_response = Response::from_parts(parts, Body::from(transformed_body));
            response = new_response;
        }

        Ok(response)
    }

    async fn transform_body_content(&self, body_bytes: &[u8], body_config: &BodyTransform) -> BackworksResult<Vec<u8>> {
        // Try to parse as JSON first for JSON transformations
        if let Ok(body_str) = std::str::from_utf8(body_bytes) {
            if let Ok(mut json_value) = serde_json::from_str::<Value>(body_str) {
                // Apply JSON field additions
                if let Some(json_field_addition) = &body_config.json_field_addition {
                    if let Value::Object(ref mut map) = json_value {
                        for (key, value) in json_field_addition {
                            map.insert(key.clone(), value.clone());
                        }
                    }
                }
                
                // Apply JSON field removals
                if let Some(json_field_removal) = &body_config.json_field_removal {
                    if let Value::Object(ref mut map) = json_value {
                        for key in json_field_removal {
                            map.remove(key);
                        }
                    }
                }
                
                // Apply JSON field renaming
                if let Some(json_field_renaming) = &body_config.json_field_renaming {
                    if let Value::Object(ref mut map) = json_value {
                        for (old_key, new_key) in json_field_renaming {
                            if let Some(value) = map.remove(old_key) {
                                map.insert(new_key.clone(), value);
                            }
                        }
                    }
                }
                
                return Ok(serde_json::to_vec(&json_value)
                    .map_err(|e| BackworksError::Proxy(format!("Failed to serialize JSON: {}", e)))?);
            }
        }
        
        // Fall back to string transformations
        let mut content = String::from_utf8_lossy(body_bytes).to_string();
        
        if let Some(string_replace) = &body_config.string_replace {
            for replace_rule in string_replace {
                if replace_rule.is_regex.unwrap_or(false) {
                    // TODO: Implement regex replacement when regex crate is available
                    tracing::warn!("Regex replacement not yet implemented, falling back to string replace");
                    content = content.replace(&replace_rule.pattern, &replace_rule.replacement);
                } else {
                    content = content.replace(&replace_rule.pattern, &replace_rule.replacement);
                }
            }
        }
        
        Ok(content.into_bytes())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_proxy_config() -> ProxyConfig {
        ProxyConfig {
            target: "http://localhost:8001".to_string(),
            targets: Some(vec![
                ProxyTarget {
                    name: "backend1".to_string(),
                    url: "http://localhost:8001".to_string(),
                    weight: Some(1.0),
                    health_check: None,
                    timeout: None,
                    retry_attempts: None,
                    circuit_breaker: None,
                }
            ]),
            strip_prefix: None,
            timeout: Some(30),
            transform_request: None,
            transform_response: None,
            health_checks: Some(false),
            load_balancing: Some(crate::config::LoadBalancingConfig {
                algorithm: LoadBalancingAlgorithm::RoundRobin,
            }),
            headers: Some(HashMap::new()),
            capture: None,
        }
    }

    #[tokio::test]
    async fn test_proxy_handler_creation() {
        let config = create_test_proxy_config();
        let handler = ProxyHandler::new(config);
        
        assert!(handler.start().await.is_ok());
    }

    #[tokio::test]
    async fn test_target_management() {
        let config = create_test_proxy_config();
        let handler = ProxyHandler::new(config);
        
        let target = ProxyTarget {
            name: "test_target".to_string(),
            url: "http://example.com".to_string(),
            weight: Some(1.0),
            health_check: None,
            timeout: None,
            retry_attempts: None,
            circuit_breaker: None,
        };
        
        handler.add_target(target.clone()).await.unwrap();
        
        let targets = handler.get_targets().await;
        assert!(targets.iter().any(|t| t.name == "test_target"));
        
        handler.remove_target("test_target").await.unwrap();
        
        let targets = handler.get_targets().await;
        assert!(!targets.iter().any(|t| t.name == "test_target"));
    }

    #[test]
    fn test_url_building() {
        let config = create_test_proxy_config();
        let handler = ProxyHandler::new(config);
        
        let target = ProxyTarget {
            name: "test".to_string(),
            url: "http://backend.example.com:8080".to_string(),
            weight: None,
            health_check: None,
            timeout: None,
            retry_attempts: None,
            circuit_breaker: None,
        };
        
        let uri: http::Uri = "/api/v1/users?page=1".parse().unwrap();
        let target_url = handler.build_target_url(&target, &uri).unwrap();
        
        assert_eq!(target_url.as_str(), "http://backend.example.com:8080/api/v1/users?page=1");
    }
}
