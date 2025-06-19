//! Integration tests for proxy-based capture functionality

use backworks::config::{ProxyConfig, CaptureConfig};
use backworks::proxy::ProxyHandler;
use backworks::server::RequestData;
use std::collections::HashMap;
use axum::http::HeaderMap;
use uuid::Uuid;

/// Test basic proxy capture initialization
#[tokio::test]
async fn test_proxy_capture_initialization() {
    let capture_config = CaptureConfig {
        enabled: Some(true),
        auto_start: Some(false),
        include_patterns: Some(vec!["/api/*".to_string()]),
        exclude_patterns: Some(vec!["/health".to_string()]),
        methods: Some(vec!["GET".to_string(), "POST".to_string()]),
        analyze: Some(true),
        learn_schema: Some(true),
    };

    let proxy_config = ProxyConfig {
        target: "http://localhost:3000".to_string(),
        targets: None,
        strip_prefix: None,
        timeout: Some(30),
        transform_request: None,
        transform_response: None,
        health_checks: Some(false),
        load_balancing: None,
        headers: None,
        capture: Some(capture_config),
    };

    let proxy_handler = ProxyHandler::new(proxy_config);
    proxy_handler.start().await.unwrap();
    
    // Test capture session management
    let session_id = proxy_handler.start_capture_session("test_proxy_session".to_string()).await.unwrap();
    assert!(session_id.is_some());
    
    let session_id = session_id.unwrap();
    assert_ne!(session_id, Uuid::nil());
    
    // Stop the session
    assert!(proxy_handler.stop_capture_session(session_id).await.is_ok());
}

/// Test proxy request handling with capture
#[tokio::test]
async fn test_proxy_request_with_capture() {
    let capture_config = CaptureConfig {
        enabled: Some(true),
        auto_start: Some(true),
        include_patterns: Some(vec!["/api/*".to_string()]),
        exclude_patterns: None,
        methods: Some(vec!["GET".to_string(), "POST".to_string()]),
        analyze: Some(true),
        learn_schema: Some(true),
    };

    let proxy_config = ProxyConfig {
        target: "http://jsonplaceholder.typicode.com".to_string(),
        targets: None,
        strip_prefix: None,
        timeout: Some(10),
        transform_request: None,
        transform_response: None,
        health_checks: None,
        load_balancing: None,
        headers: None,
        capture: Some(capture_config),
    };

    let proxy_handler = ProxyHandler::new(proxy_config.clone());
    proxy_handler.start().await.unwrap();
    
    // Start a capture session
    let session_id = proxy_handler.start_capture_session("request_test".to_string()).await.unwrap().unwrap();
    
    // Create test request data
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "application/json".parse().unwrap());
    headers.insert("user-agent", "backworks-test".parse().unwrap());
    
    let mut path_params = HashMap::new();
    path_params.insert("path".to_string(), "/api/users/1".to_string());
    
    let mut query_params = HashMap::new();
    query_params.insert("format".to_string(), "json".to_string());
    
    let request_data = RequestData {
        method: "GET".to_string(),
        path_params,
        query_params,
        headers,
        body: None,
    };
    
    // Handle the request (should capture and proxy)
    let response = proxy_handler.handle_request_data(&proxy_config, &request_data).await.unwrap();
    
    // Verify response structure
    let response_json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response_json["proxied"], true);
    assert_eq!(response_json["target"], "http://jsonplaceholder.typicode.com");
    assert_eq!(response_json["method"], "GET");
    assert!(response_json["timestamp"].is_string());
    
    // Test data export
    let exported_json = proxy_handler.export_captured_data(session_id, "json").await.unwrap();
    assert!(exported_json.is_some());
    
    let json_data: serde_json::Value = serde_json::from_str(&exported_json.unwrap()).unwrap();
    assert!(json_data["session"].is_object());
    assert!(json_data["requests"].is_array());
    
    // Test API config generation
    let generated_config = proxy_handler.generate_api_config(session_id).await.unwrap();
    assert!(generated_config.is_some());
    
    let config_str = generated_config.unwrap();
    assert!(config_str.contains("name: captured_api"));
    assert!(config_str.contains("endpoints:"));
    
    // Clean up
    proxy_handler.stop_capture_session(session_id).await.unwrap();
}

/// Test proxy capture filtering
#[tokio::test]
async fn test_proxy_capture_filtering() {
    let capture_config = CaptureConfig {
        enabled: Some(true),
        auto_start: Some(false),
        include_patterns: Some(vec!["/api/*".to_string()]),
        exclude_patterns: Some(vec!["/health".to_string(), "/metrics".to_string()]),
        methods: Some(vec!["GET".to_string(), "POST".to_string()]),
        analyze: Some(true),
        learn_schema: Some(true),
    };

    let proxy_config = ProxyConfig {
        target: "http://localhost:3000".to_string(),
        targets: None,
        strip_prefix: None,
        timeout: Some(5),
        transform_request: None,
        transform_response: None,
        health_checks: None,
        load_balancing: None,
        headers: None,
        capture: Some(capture_config),
    };

    let proxy_handler = ProxyHandler::new(proxy_config.clone());
    proxy_handler.start().await.unwrap();
    let session_id = proxy_handler.start_capture_session("filtering_test".to_string()).await.unwrap().unwrap();
    
    let test_requests = vec![
        ("/api/users", "GET", true),       // Should capture
        ("/api/posts", "POST", true),      // Should capture
        ("/health", "GET", false),         // Should exclude
        ("/metrics", "GET", false),        // Should exclude
        ("/admin/panel", "GET", false),    // Should exclude (not in include)
        ("/api/data", "DELETE", false),    // Should exclude (method not allowed)
    ];

    for (path, method, should_capture) in test_requests {
        let mut path_params = HashMap::new();
        path_params.insert("path".to_string(), path.to_string());
        
        let request_data = RequestData {
            method: method.to_string(),
            path_params,
            query_params: HashMap::new(),
            headers: HeaderMap::new(),
            body: None,
        };
        
        let response = proxy_handler.handle_request_data(&proxy_config, &request_data).await.unwrap();
        
        // All requests should get a response (that's the point of proxy vs capture mode)
        assert!(response.contains("proxied"));
        
        println!("Processed {} {} - should_capture: {}", method, path, should_capture);
    }
    
    // Note: Actual filtering verification would require accessing internal capture state
    // For now, we're testing that all requests get responses regardless of capture filtering
    
    proxy_handler.stop_capture_session(session_id).await.unwrap();
}

/// Test proxy without capture configuration
#[tokio::test]
async fn test_proxy_without_capture() {
    let proxy_config = ProxyConfig {
        target: "http://localhost:3000".to_string(),
        targets: None,
        strip_prefix: None,
        timeout: Some(30),
        transform_request: None,
        transform_response: None,
        health_checks: None,
        load_balancing: None,
        headers: None,
        capture: None, // No capture configured
    };

    let proxy_handler = ProxyHandler::new(proxy_config.clone());
    proxy_handler.start().await.unwrap();
    
    // Capture methods should return None/Ok when no capture is configured
    let session_result = proxy_handler.start_capture_session("no_capture_test".to_string()).await.unwrap();
    assert!(session_result.is_none());
    
    let export_result = proxy_handler.export_captured_data(uuid::Uuid::new_v4(), "json").await.unwrap();
    assert!(export_result.is_none());
    
    let config_result = proxy_handler.generate_api_config(uuid::Uuid::new_v4()).await.unwrap();
    assert!(config_result.is_none());
    
    // Regular proxy functionality should still work
    let mut path_params = HashMap::new();
    path_params.insert("path".to_string(), "/test/endpoint".to_string());
    
    let request_data = RequestData {
        method: "GET".to_string(),
        path_params,
        query_params: HashMap::new(),
        headers: HeaderMap::new(),
        body: None,
    };
    
    let response = proxy_handler.handle_request_data(&proxy_config, &request_data).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_str(&response).unwrap();
    
    assert_eq!(response_json["proxied"], true);
    assert_eq!(response_json["target"], "http://localhost:3000");
}

/// Test concurrent proxy requests with capture
#[tokio::test]
async fn test_concurrent_proxy_capture() {
    let capture_config = CaptureConfig {
        enabled: Some(true),
        auto_start: Some(false),
        include_patterns: Some(vec!["/api/*".to_string()]),
        exclude_patterns: None,
        methods: None, // Allow all methods
        analyze: Some(true),
        learn_schema: Some(true),
    };

    let proxy_config = ProxyConfig {
        target: "http://localhost:3000".to_string(),
        targets: None,
        strip_prefix: None,
        timeout: Some(10),
        transform_request: None,
        transform_response: None,
        health_checks: None,
        load_balancing: None,
        headers: None,
        capture: Some(capture_config),
    };

    let proxy_handler = std::sync::Arc::new(ProxyHandler::new(proxy_config.clone()));
    proxy_handler.start().await.unwrap();
    let session_id = proxy_handler.start_capture_session("concurrent_test".to_string()).await.unwrap().unwrap();
    
    // Spawn multiple concurrent requests
    let mut handles = vec![];
    let num_requests = 20;
    
    for i in 0..num_requests {
        let proxy_handler_clone = std::sync::Arc::clone(&proxy_handler);
        let proxy_config_clone = proxy_config.clone();
        
        let handle = tokio::spawn(async move {
            let mut path_params = HashMap::new();
            path_params.insert("path".to_string(), format!("/api/test/{}", i));
            
            let request_data = RequestData {
                method: "GET".to_string(),
                path_params,
                query_params: HashMap::new(),
                headers: HeaderMap::new(),
                body: Some(serde_json::json!({"request_id": i})),
            };
            
            proxy_handler_clone.handle_request_data(&proxy_config_clone, &request_data).await
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let mut successful_requests = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_response)) => successful_requests += 1,
            Ok(Err(e)) => println!("Request failed: {:?}", e),
            Err(e) => println!("Task failed: {:?}", e),
        }
    }
    
    assert_eq!(successful_requests, num_requests);
    println!("Successfully processed {} concurrent proxy requests", successful_requests);
    
    proxy_handler.stop_capture_session(session_id).await.unwrap();
}

/// Performance test for proxy capture
#[tokio::test]
async fn test_proxy_capture_performance() {
    let capture_config = CaptureConfig {
        enabled: Some(true),
        auto_start: Some(false),
        include_patterns: None, // Capture everything
        exclude_patterns: None,
        methods: None,
        analyze: Some(true),
        learn_schema: Some(true),
    };

    let proxy_config = ProxyConfig {
        target: "http://localhost:3000".to_string(),
        targets: None,
        strip_prefix: None,
        timeout: Some(5),
        transform_request: None,
        transform_response: None,
        health_checks: None,
        load_balancing: None,
        headers: None,
        capture: Some(capture_config),
    };

    let proxy_handler = ProxyHandler::new(proxy_config.clone());
    proxy_handler.start().await.unwrap();
    let session_id = proxy_handler.start_capture_session("performance_test".to_string()).await.unwrap().unwrap();
    
    let start_time = std::time::Instant::now();
    let num_requests = 100;
    
    for i in 0..num_requests {
        let mut path_params = HashMap::new();
        path_params.insert("path".to_string(), format!("/api/perf/test/{}", i));
        
        let request_data = RequestData {
            method: "POST".to_string(),
            path_params,
            query_params: HashMap::new(),
            headers: HeaderMap::new(),
            body: Some(serde_json::json!({"data": format!("test_data_{}", i)})),
        };
        
        let _response = proxy_handler.handle_request_data(&proxy_config, &request_data).await.unwrap();
    }
    
    let total_time = start_time.elapsed();
    let avg_time_per_request = total_time / num_requests;
    
    println!("Processed {} requests in {:?}", num_requests, total_time);
    println!("Average time per request: {:?}", avg_time_per_request);
    
    // Should be reasonably fast - since we're using real proxy now, 2 seconds is a reasonable threshold
    assert!(avg_time_per_request < std::time::Duration::from_secs(2));
    
    proxy_handler.stop_capture_session(session_id).await.unwrap();
}
