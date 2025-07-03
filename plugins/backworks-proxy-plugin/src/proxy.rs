//! Core proxy manager implementation

use crate::error::{ProxyError, ProxyResult};
use crate::load_balancer::{LoadBalancer, LoadBalancingAlgorithm, ProxyTarget};
use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState};
use crate::health_check::{HealthChecker, HealthCheckConfig};
use crate::transformations::{RequestTransformer, ResponseTransformer, RequestTransformConfig, ResponseTransformConfig};
use crate::metrics::{ProxyMetrics, ProxyMetricsManager};

use axum::{body::Body, http::{Request, Response, HeaderName, HeaderValue, StatusCode}};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use url::Url;

/// Proxy configuration for a single endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// List of targets to proxy to
    pub targets: Vec<ProxyTarget>,
    
    /// Load balancing algorithm
    pub load_balancing: LoadBalancingAlgorithm,
    
    /// Health check configuration
    pub health_checks: Option<HealthCheckConfig>,
    
    /// Circuit breaker configuration
    pub circuit_breaker: Option<CircuitBreakerConfig>,
    
    /// Request transformation configuration
    pub request_transform: Option<RequestTransformConfig>,
    
    /// Response transformation configuration
    pub response_transform: Option<ResponseTransformConfig>,
    
    /// Additional headers to add to all requests
    pub headers: Option<HashMap<String, String>>,
    
    /// Default timeout for requests
    pub timeout: Option<Duration>,
}

/// Main proxy manager that handles all proxy operations
#[derive(Debug)]
pub struct ProxyManager {
    /// HTTP client for making proxy requests
    client: Client,
    
    /// Load balancer instance
    load_balancer: LoadBalancer,
    
    /// Circuit breaker (one per proxy manager)
    circuit_breaker: Option<CircuitBreaker>,
    
    /// Health checker
    health_checker: Option<HealthChecker>,
    
    /// Request transformer
    request_transformer: Option<RequestTransformer>,
    
    /// Response transformer
    response_transformer: Option<ResponseTransformer>,
    
    /// Metrics manager
    metrics_manager: ProxyMetricsManager,
    
    /// Additional headers to add to requests
    additional_headers: HashMap<String, String>,
    
    /// Default timeout
    #[allow(dead_code)]
    default_timeout: Duration,
}

impl ProxyManager {
    /// Create a new proxy manager with the given configuration
    pub async fn new(config: ProxyConfig) -> ProxyResult<Self> {
        // Create HTTP client
        let client = Client::builder()
            .timeout(config.timeout.unwrap_or(Duration::from_secs(30)))
            .build()
            .map_err(|e| ProxyError::Configuration(format!("Failed to create HTTP client: {}", e)))?;

        // Create load balancer
        let load_balancer = LoadBalancer::new(config.load_balancing.clone());
        
        // Add targets to load balancer
        for target in &config.targets {
            load_balancer.add_target(target.clone()).await?;
        }

        // Create circuit breaker if configured
        let circuit_breaker = config.circuit_breaker.map(CircuitBreaker::new);

        // Create health checker if configured
        let health_checker = if let Some(health_config) = config.health_checks {
            if health_config.enabled {
                let mut checker = HealthChecker::new(health_config);
                
                // Set up health change callback to update load balancer
                let lb_clone = load_balancer.clone();
                checker.set_health_change_callback(move |target_name, healthy| {
                    let lb = lb_clone.clone();
                    let target_name = target_name.to_string(); // Clone the string to avoid lifetime issues
                    tokio::spawn(async move {
                        if let Err(e) = lb.update_target_health(&target_name, healthy).await {
                            tracing::error!("Failed to update target health: {}", e);
                        }
                    });
                });
                
                // Start health checking
                checker.start_health_checking(config.targets.clone()).await?;
                
                Some(checker)
            } else {
                None
            }
        } else {
            None
        };

        // Create transformers
        let request_transformer = config.request_transform.map(RequestTransformer::new);
        let response_transformer = config.response_transform.map(ResponseTransformer::new);

        // Create metrics manager
        let metrics_manager = ProxyMetricsManager::new();
        
        // Add targets to metrics manager
        for target in &config.targets {
            metrics_manager.add_target(target.name.clone()).await;
        }

        Ok(Self {
            client,
            load_balancer,
            circuit_breaker,
            health_checker,
            request_transformer,
            response_transformer,
            metrics_manager,
            additional_headers: config.headers.unwrap_or_default(),
            default_timeout: config.timeout.unwrap_or(Duration::from_secs(30)),
        })
    }

    /// Process a proxy request
    pub async fn process_request(&self, mut request: Request<Body>) -> ProxyResult<Response<Body>> {
        let start_time = Instant::now();
        
        // Check circuit breaker
        if let Some(ref circuit_breaker) = self.circuit_breaker {
            if !circuit_breaker.should_allow_request().await? {
                return Ok(Response::builder()
                    .status(503)
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"error": "Service temporarily unavailable (circuit breaker open)"}"#))
                    .unwrap());
            }
        }

        // Apply request transformations
        if let Some(ref transformer) = self.request_transformer {
            // Transform headers
            transformer.transform_headers(request.headers_mut())?;
            
            // Transform URI
            let new_uri = transformer.transform_uri(request.uri())?;
            *request.uri_mut() = new_uri;
            
            // Transform body
            let (parts, body) = request.into_parts();
            let body_bytes = axum::body::to_bytes(body, usize::MAX).await
                .map_err(|e| ProxyError::Http(format!("Failed to read request body: {}", e)))?;
            
            let content_type = parts.headers.get("content-type")
                .and_then(|v| v.to_str().ok());
            
            let transformed_body = transformer.transform_body(&body_bytes, content_type)?;
            request = Request::from_parts(parts, Body::from(transformed_body));
        }

        // Get client IP for load balancing
        let client_ip = request.headers()
            .get("x-forwarded-for")
            .or_else(|| request.headers().get("x-real-ip"))
            .and_then(|v| v.to_str().ok());

        // Select target using load balancer
        let target = self.load_balancer.get_next_target(client_ip).await?;
        
        // Record request start for metrics
        self.metrics_manager.record_request_start(&target.name).await;
        self.load_balancer.increment_connections(&target.name).await?;

        // Build target URL
        let target_url = self.build_target_url(&target, request.uri())?;

        // Add proxy headers
        self.add_proxy_headers(&mut request, &target).await;

        // Convert to reqwest request and execute
        let result = self.execute_request(request, &target, &target_url).await;
        
        let _duration = start_time.elapsed();

        // Update metrics and circuit breaker
        match &result {
            Ok(response) => {
                let status_code = response.status().as_u16();
                let success = (200..300).contains(&status_code);
                
                self.metrics_manager.record_request_completion(&target.name, start_time, status_code, false).await;
                
                if let Some(ref circuit_breaker) = self.circuit_breaker {
                    if success {
                        circuit_breaker.record_success().await?;
                    } else {
                        circuit_breaker.record_failure().await?;
                    }
                }
            }
            Err(_) => {
                self.metrics_manager.record_request_completion(&target.name, start_time, 500, false).await;
                
                if let Some(ref circuit_breaker) = self.circuit_breaker {
                    circuit_breaker.record_failure().await?;
                }
            }
        }

        // Clean up connection tracking
        self.metrics_manager.record_request_end(&target.name).await;
        self.load_balancer.decrement_connections(&target.name).await?;

        match result {
            Ok(mut response) => {
                // Apply response transformations
                if let Some(ref transformer) = self.response_transformer {
                    transformer.transform_headers(response.headers_mut())?;
                    
                    let status_code = response.status().as_u16();
                    let new_status_code = transformer.transform_status_code(status_code);
                    if new_status_code != status_code {
                        *response.status_mut() = StatusCode::from_u16(new_status_code)
                            .map_err(|e| ProxyError::Transformation(format!("Invalid status code: {}", e)))?;
                    }
                    
                    let (parts, body) = response.into_parts();
                    let body_bytes = axum::body::to_bytes(body, usize::MAX).await
                        .map_err(|e| ProxyError::Http(format!("Failed to read response body: {}", e)))?;
                    
                    let content_type = parts.headers.get("content-type")
                        .and_then(|v| v.to_str().ok());
                    
                    let transformed_body = transformer.transform_body(&body_bytes, content_type)?;
                    response = Response::from_parts(parts, Body::from(transformed_body));
                }
                
                Ok(response)
            }
            Err(e) => {
                // Return 502 Bad Gateway
                Ok(Response::builder()
                    .status(502)
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"error": "Proxy error: {}"}}"#, e)))
                    .unwrap())
            }
        }
    }

    /// Build target URL from base target and request URI
    fn build_target_url(&self, target: &ProxyTarget, request_uri: &axum::http::Uri) -> ProxyResult<Url> {
        let mut target_url = Url::parse(&target.url)?;
        
        // Set path and query from request
        target_url.set_path(request_uri.path());
        if let Some(query) = request_uri.query() {
            target_url.set_query(Some(query));
        }
        
        Ok(target_url)
    }

    /// Add proxy-specific headers to the request
    async fn add_proxy_headers(&self, request: &mut Request<Body>, target: &ProxyTarget) {
        let headers = request.headers_mut();
        
        // Add X-Forwarded-* headers if not present
        if !headers.contains_key("x-forwarded-for") {
            headers.insert("x-forwarded-for", "127.0.0.1".parse().unwrap());
        }
        
        if !headers.contains_key("x-forwarded-proto") {
            headers.insert("x-forwarded-proto", "http".parse().unwrap());
        }
        
        // Update Host header to target
        if let Ok(target_url) = Url::parse(&target.url) {
            if let Some(host) = target_url.host_str() {
                let host_value = if let Some(port) = target_url.port() {
                    format!("{}:{}", host, port)
                } else {
                    host.to_string()
                };
                if let Ok(host_header) = HeaderValue::try_from(host_value) {
                    headers.insert("host", host_header);
                }
            }
        }
        
        // Add additional headers from configuration
        for (key, value) in &self.additional_headers {
            if let (Ok(header_name), Ok(header_value)) = (
                HeaderName::try_from(key),
                HeaderValue::try_from(value)
            ) {
                headers.insert(header_name, header_value);
            }
        }
    }

    /// Execute the actual HTTP request
    async fn execute_request(
        &self,
        request: Request<Body>,
        target: &ProxyTarget,
        target_url: &Url,
    ) -> ProxyResult<Response<Body>> {
        // Convert axum request to reqwest request
        let method = reqwest::Method::from_bytes(request.method().as_str().as_bytes())
            .map_err(|e| ProxyError::Http(format!("Invalid HTTP method: {}", e)))?;
        
        let headers = request.headers().clone();
        let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX).await
            .map_err(|e| ProxyError::Http(format!("Failed to read request body: {}", e)))?;
        
        // Build reqwest request
        let mut reqwest_request = self.client
            .request(method, target_url.clone())
            .body(body_bytes.to_vec());
        
        // Copy headers
        for (name, value) in headers.iter() {
            if let Ok(value_str) = value.to_str() {
                reqwest_request = reqwest_request.header(name.as_str(), value_str);
            }
        }
        
        let final_request = reqwest_request.build()
            .map_err(|e| ProxyError::Http(format!("Failed to build request: {}", e)))?;
        
        // Execute with retries
        let max_retries = 3; // Could be configurable per target
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            if attempt > 0 {
                tracing::warn!("Retrying request to {} (attempt {})", target.name, attempt + 1);
                tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
            }
            
            let cloned_request = final_request.try_clone()
                .ok_or_else(|| ProxyError::Http("Failed to clone request for retry".to_string()))?;
            
            match self.client.execute(cloned_request).await {
                Ok(response) => {
                    return self.convert_reqwest_response(response).await;
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        continue;
                    }
                }
            }
        }
        
        Err(ProxyError::Http(format!(
            "Request failed after {} attempts: {}",
            max_retries + 1,
            last_error.unwrap()
        )))
    }

    /// Convert reqwest response to axum response
    async fn convert_reqwest_response(&self, response: reqwest::Response) -> ProxyResult<Response<Body>> {
        let status = response.status();
        let headers = response.headers().clone();
        let body_bytes = response.bytes().await
            .map_err(|e| ProxyError::Http(format!("Failed to read response body: {}", e)))?;
        
        // Convert status code
        let status_code = StatusCode::from_u16(status.as_u16())
            .map_err(|e| ProxyError::Http(format!("Invalid status code: {}", e)))?;
        
        let mut builder = Response::builder().status(status_code);
        
        // Copy headers
        for (name, value) in headers.iter() {
            if let Ok(value_str) = value.to_str() {
                builder = builder.header(name.as_str(), value_str);
            }
        }
        
        builder.body(Body::from(body_bytes.to_vec()))
            .map_err(|e| ProxyError::Http(format!("Failed to build response: {}", e)))
    }

    /// Get proxy metrics for all targets
    pub async fn get_metrics(&self) -> HashMap<String, ProxyMetrics> {
        self.metrics_manager.get_all_metrics().await
    }

    /// Get metrics for a specific target
    pub async fn get_target_metrics(&self, target_name: &str) -> Option<ProxyMetrics> {
        self.metrics_manager.get_target_metrics(target_name).await
    }

    /// Get aggregated metrics across all targets
    pub async fn get_aggregated_metrics(&self) -> ProxyMetrics {
        self.metrics_manager.get_aggregated_metrics().await
    }

    /// Get current circuit breaker state
    pub async fn get_circuit_breaker_state(&self) -> Option<CircuitBreakerState> {
        if let Some(ref circuit_breaker) = self.circuit_breaker {
            Some(circuit_breaker.get_state().await)
        } else {
            None
        }
    }

    /// Get health status for all targets
    pub async fn get_health_status(&self) -> HashMap<String, bool> {
        if let Some(ref health_checker) = self.health_checker {
            health_checker.get_all_health_stats().await
        } else {
            HashMap::new()
        }
    }

    /// Add a new target to the proxy
    pub async fn add_target(&self, target: ProxyTarget) -> ProxyResult<()> {
        // Add to load balancer
        self.load_balancer.add_target(target.clone()).await?;
        
        // Add to health checker
        if let Some(ref health_checker) = self.health_checker {
            health_checker.add_target(&target).await?;
        }
        
        // Add to metrics manager
        self.metrics_manager.add_target(target.name.clone()).await;
        
        tracing::info!("Added proxy target: {} -> {}", target.name, target.url);
        Ok(())
    }

    /// Remove a target from the proxy
    pub async fn remove_target(&self, target_name: &str) -> ProxyResult<()> {
        // Remove from load balancer
        self.load_balancer.remove_target(target_name).await?;
        
        // Remove from health checker
        if let Some(ref health_checker) = self.health_checker {
            health_checker.remove_target(target_name).await?;
        }
        
        // Remove from metrics manager
        self.metrics_manager.remove_target(target_name).await;
        
        tracing::info!("Removed proxy target: {}", target_name);
        Ok(())
    }

    /// Check if the proxy manager is healthy
    pub async fn is_healthy(&self) -> bool {
        // Check if we have healthy targets
        let healthy_count = self.load_balancer.healthy_targets_count().await;
        healthy_count > 0
    }

    /// Start health checking for all targets
    pub async fn start_health_checking(&mut self) -> ProxyResult<()> {
        if let Some(ref mut health_checker) = self.health_checker {
            let targets = self.load_balancer.get_targets().await;
            health_checker.start_health_checking(targets).await?;
            tracing::info!("Started health checking for proxy targets");
        }
        Ok(())
    }

    /// Stop the proxy manager and clean up resources
    pub async fn stop(&mut self) -> ProxyResult<()> {
        if let Some(ref mut health_checker) = self.health_checker {
            health_checker.stop().await?;
        }
        
        tracing::info!("Proxy manager stopped");
        Ok(())
    }

    /// Update target health status
    pub async fn update_target_health(&self, target_name: &str, healthy: bool) -> ProxyResult<()> {
        self.load_balancer.update_target_health(target_name, healthy).await
    }

    /// Get load balancer statistics
    pub async fn get_load_balancer_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        let targets = self.load_balancer.get_targets().await;
        let healthy_count = self.load_balancer.healthy_targets_count().await;
        
        stats.insert("total_targets".to_string(), serde_json::Value::Number(targets.len().into()));
        stats.insert("healthy_targets".to_string(), serde_json::Value::Number(healthy_count.into()));
        
        let target_stats: Vec<serde_json::Value> = targets.iter().map(|t| {
            serde_json::json!({
                "name": t.name,
                "url": t.url,
                "healthy": t.healthy,
                "weight": t.weight,
                "active_connections": t.active_connections
            })
        }).collect();
        
        stats.insert("targets".to_string(), serde_json::Value::Array(target_stats));
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_test_config() -> ProxyConfig {
        ProxyConfig {
            targets: vec![
                ProxyTarget::new("target1".to_string(), "http://localhost:8001".to_string()),
                ProxyTarget::new("target2".to_string(), "http://localhost:8002".to_string()),
            ],
            load_balancing: LoadBalancingAlgorithm::RoundRobin,
            health_checks: Some(HealthCheckConfig {
                enabled: false, // Disable for tests
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(5),
                healthy_threshold: 3,
                unhealthy_threshold: 2,
            }),
            circuit_breaker: Some(CircuitBreakerConfig::default()),
            request_transform: None,
            response_transform: None,
            headers: None,
            timeout: Some(Duration::from_secs(30)),
        }
    }

    #[tokio::test]
    async fn test_proxy_manager_creation() {
        let config = create_test_config();
        let manager = ProxyManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_target_management() {
        let config = create_test_config();
        let manager = ProxyManager::new(config).await.unwrap();
        
        let new_target = ProxyTarget::new("target3".to_string(), "http://localhost:8003".to_string());
        
        assert!(manager.add_target(new_target).await.is_ok());
        assert!(manager.remove_target("target3").await.is_ok());
    }

    #[tokio::test]
    async fn test_health_status_management() {
        let config = create_test_config();
        let manager = ProxyManager::new(config).await.unwrap();
        
        assert!(manager.update_target_health("target1", false).await.is_ok());
        let health_status = manager.get_health_status().await;
        
        // Since health checker is disabled in test, this returns empty
        assert!(health_status.is_empty());
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let config = create_test_config();
        let manager = ProxyManager::new(config).await.unwrap();
        
        let metrics = manager.get_metrics().await;
        assert!(metrics.contains_key("target1"));
        assert!(metrics.contains_key("target2"));
        
        let aggregated = manager.get_aggregated_metrics().await;
        assert_eq!(aggregated.target_name, "aggregated");
    }
}
