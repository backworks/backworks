use backworks::proxy::ProxyHandler;
use backworks::config::{ProxyConfig, LoadBalancingConfig, LoadBalancingAlgorithm};
use axum::{http::{Request, Method}, body::Body};
use std::collections::HashMap;

#[tokio::test]
async fn test_basic_proxy_handler_creation() {
    let config = ProxyConfig {
        target: "http://httpbin.org".to_string(),
        targets: None,
        strip_prefix: None,
        timeout: Some(30),
        transform_request: None,
        transform_response: None,
        health_checks: Some(false),
        load_balancing: Some(LoadBalancingConfig {
            algorithm: LoadBalancingAlgorithm::RoundRobin,
        }),
        headers: Some(HashMap::new()),
        capture: None,
    };
    
    let handler = ProxyHandler::new(config);
    assert!(handler.start().await.is_ok());
}

#[tokio::test]
async fn test_proxy_request_creation() {
    let config = ProxyConfig {
        target: "http://httpbin.org".to_string(),
        targets: None,
        strip_prefix: None,
        timeout: Some(30),
        transform_request: None,
        transform_response: None,
        health_checks: Some(false),
        load_balancing: Some(LoadBalancingConfig {
            algorithm: LoadBalancingAlgorithm::RoundRobin,
        }),
        headers: Some(HashMap::new()),
        capture: None,
    };
    
    let handler = ProxyHandler::new(config);
    assert!(handler.start().await.is_ok());
    
    // Create a simple GET request
    let request = Request::builder()
        .method(Method::GET)
        .uri("/get")
        .header("user-agent", "backworks-test")
        .body(Body::empty())
        .unwrap();
    
    // Test that proxy request handling doesn't panic
    // Note: This might fail due to network issues, but it should at least not panic
    let result = handler.handle_request(request).await;
    // We don't assert success here as httpbin.org might not be available
    // but we can assert that the method doesn't panic
    println!("Proxy request result: {:?}", result.is_ok());
}

#[tokio::test]
async fn test_proxy_metrics() {
    let config = ProxyConfig {
        target: "http://httpbin.org".to_string(),
        targets: None,
        strip_prefix: None,
        timeout: Some(30),
        transform_request: None,
        transform_response: None,
        health_checks: Some(false),
        load_balancing: Some(LoadBalancingConfig {
            algorithm: LoadBalancingAlgorithm::RoundRobin,
        }),
        headers: Some(HashMap::new()),
        capture: None,
    };
    
    let handler = ProxyHandler::new(config);
    assert!(handler.start().await.is_ok());
    
    // Test that metrics can be retrieved
    let metrics = handler.get_metrics().await;
    assert!(metrics.len() >= 1); // Should have at least the default target
    
    let targets = handler.get_targets().await;
    assert!(targets.len() >= 1); // Should have at least the default target
}
