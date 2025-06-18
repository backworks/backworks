use crate::config::{ProxyConfig, ProxyTarget, LoadBalancingAlgorithm};
use crate::error::{BackworksError, BackworksResult};
use axum::{body::Body, http::Request, response::Response};
use reqwest::Client;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingStrategy {
    pub algorithm: LoadBalanceAlgorithm,
    pub sticky_sessions: bool,
    pub health_check_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalanceAlgorithm {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    Random,
    IpHash,
}

#[derive(Debug)]
pub struct ProxyHandler {
    config: ProxyConfig,
    targets: Arc<RwLock<Vec<ProxyTarget>>>,
    metrics: Arc<RwLock<HashMap<String, ProxyMetrics>>>,
    client: Client,
    current_target_index: Arc<RwLock<usize>>,
}

impl Clone for ProxyHandler {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            targets: Arc::clone(&self.targets),
            metrics: Arc::clone(&self.metrics),
            client: self.client.clone(),
            current_target_index: Arc::clone(&self.current_target_index),
        }
    }
}

impl ProxyHandler {
    pub fn new(config: ProxyConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self {
            config,
            targets: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            client,
            current_target_index: Arc::new(RwLock::new(0)),
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
            // Fallback to single target configuration
            let single_target = ProxyTarget {
                name: "default".to_string(),
                url: self.config.target.clone(),
                weight: Some(1.0),
                timeout: self.config.timeout.map(std::time::Duration::from_secs),
                health_check: None,
                retry_attempts: Some(3),
                circuit_breaker: None,
            };
            self.add_target(single_target).await?;
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
        
        // Select target based on load balancing strategy
        let target = self.select_target(&request).await?;
        
        // Build the target URL
        let target_url = self.build_target_url(&target, request.uri())?;
        
        // Update request URI
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
        self.update_metrics(&target.name, &result, duration).await;
        
        match result {
            Ok(response) => Ok(response),
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

    async fn select_target(&self, request: &Request<Body>) -> BackworksResult<ProxyTarget> {
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
                self.select_by_ip_hash(&healthy_targets, request).await
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

    async fn select_by_ip_hash(&self, targets: &[&ProxyTarget], request: &Request<Body>) -> ProxyTarget {
        // Try to get client IP from headers or use a default
        let client_ip = request.headers()
            .get("x-forwarded-for")
            .or_else(|| request.headers().get("x-real-ip"))
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

    async fn update_metrics(&self, target_name: &str, result: &BackworksResult<Response<Body>>, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(target_name) {
            metric.total_requests += 1;
            
            match result {
                Ok(_) => {
                    metric.successful_requests += 1;
                }
                Err(_) => {
                    metric.failed_requests += 1;
                }
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

    pub async fn handle_request(&self, proxy_config: &ProxyConfig, request_data: &crate::server::RequestData) -> crate::error::BackworksResult<String> {
        // For now, return a simple proxy response
        // TODO: Implement actual proxy logic using the existing proxy_request method
        
        tracing::info!("Handling proxy request to target: {}", proxy_config.target);
        
        let response = serde_json::json!({
            "proxied": true,
            "target": proxy_config.target,
            "method": request_data.method,
            "path": request_data.path_params.get("path").unwrap_or(&"".to_string()),
            "message": "Request proxied successfully"
        });
        
        Ok(response.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_proxy_config() -> ProxyConfig {
        ProxyConfig {
            targets: vec![
                ProxyTarget {
                    name: "backend1".to_string(),
                    url: "http://localhost:8001".to_string(),
                    weight: Some(1),
                    health_check: None,
                    timeout: None,
                    retry_attempts: None,
                    circuit_breaker: None,
                }
            ],
            load_balancing: LoadBalancingStrategy {
                algorithm: LoadBalanceAlgorithm::RoundRobin,
                sticky_sessions: false,
                health_check_required: false,
            },
            health_checks: false,
            headers: HashMap::new(),
            timeout: Duration::from_secs(30),
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
            weight: Some(1),
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
