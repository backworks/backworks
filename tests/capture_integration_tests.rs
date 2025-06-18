//! Integration tests for the capture module
//! 
//! These tests demonstrate real-world usage scenarios and validate
//! the capture functionality end-to-end.

use backworks::capture::{CaptureHandler, CaptureFilter, CaptureStatus, Capturer};
use backworks::config::CaptureConfig;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// Test realistic e-commerce API capture scenario
#[tokio::test]
async fn test_ecommerce_api_capture_scenario() {
    let config = CaptureConfig {
        enabled: true,
        auto_start: Some(false),
        include_patterns: Some(vec![
            "/api/v1/products/*".to_string(),
            "/api/v1/orders/*".to_string(),
            "/api/v1/users/*".to_string(),
        ]),
        exclude_patterns: Some(vec![
            "/api/v1/health".to_string(),
            "/api/v1/metrics".to_string(),
        ]),
        methods: Some(vec!["GET".to_string(), "POST".to_string(), "PUT".to_string()]),
        max_requests: Some(1000),
        storage_path: Some("./test_captures/ecommerce".to_string()),
    };

    let handler = CaptureHandler::new(config);
    let session_id = handler.start_session("ecommerce_test".to_string()).await.unwrap();

    // Simulate product browsing
    let product_requests = vec![
        ("/api/v1/products", "GET"),
        ("/api/v1/products/1", "GET"),
        ("/api/v1/products/2", "GET"),
        ("/api/v1/products/categories", "GET"),
    ];

    for (path, method) in product_requests {
        let req_id = handler.capture_request(
            method.to_string(),
            path.to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
        ).await.unwrap();

        handler.capture_response(
            req_id,
            200,
            HashMap::new(),
            Some(serde_json::json!({
                "success": true,
                "data": format!("Response from {}", path)
            })),
            Duration::from_millis(50),
        ).await.unwrap();
    }

    // Simulate user registration and login
    let mut user_headers = HashMap::new();
    user_headers.insert("content-type".to_string(), "application/json".to_string());

    let register_req = handler.capture_request(
        "POST".to_string(),
        "/api/v1/users/register".to_string(),
        user_headers.clone(),
        HashMap::new(),
        Some(serde_json::json!({
            "email": "test@example.com",
            "password": "secure123",
            "name": "Test User"
        })),
    ).await.unwrap();

    handler.capture_response(
        register_req,
        201,
        user_headers.clone(),
        Some(serde_json::json!({
            "user_id": 12345,
            "email": "test@example.com",
            "created_at": "2024-01-15T10:30:00Z"
        })),
        Duration::from_millis(150),
    ).await.unwrap();

    // Simulate order creation
    let order_req = handler.capture_request(
        "POST".to_string(),
        "/api/v1/orders".to_string(),
        user_headers.clone(),
        HashMap::new(),
        Some(serde_json::json!({
            "user_id": 12345,
            "items": [
                {"product_id": 1, "quantity": 2},
                {"product_id": 2, "quantity": 1}
            ],
            "payment_method": "credit_card"
        })),
    ).await.unwrap();

    handler.capture_response(
        order_req,
        201,
        user_headers,
        Some(serde_json::json!({
            "order_id": "ord_abc123",
            "status": "confirmed",
            "total": 129.99,
            "estimated_delivery": "2024-01-18"
        })),
        Duration::from_millis(200),
    ).await.unwrap();

    // Verify capture results
    let requests = handler.get_captured_requests(session_id, None).await;
    assert_eq!(requests.len(), 6); // 4 product + 1 user + 1 order

    // Test filtering by method
    let get_filter = CaptureFilter {
        methods: Some(vec!["GET".to_string()]),
        path_patterns: None,
        status_codes: None,
        min_duration: None,
        max_duration: None,
    };
    let get_requests = handler.get_captured_requests(session_id, Some(get_filter)).await;
    assert_eq!(get_requests.len(), 4);

    // Test filtering by path pattern
    let product_filter = CaptureFilter {
        methods: None,
        path_patterns: Some(vec!["/api/v1/products*".to_string()]),
        status_codes: None,
        min_duration: None,
        max_duration: None,
    };
    let product_requests = handler.get_captured_requests(session_id, Some(product_filter)).await;
    assert_eq!(product_requests.len(), 4);

    // Generate API configuration
    let yaml_config = handler.generate_api_from_capture(session_id).await.unwrap();
    assert!(yaml_config.contains("name: captured_api"));
    assert!(yaml_config.contains("/api/v1/products"));
    assert!(yaml_config.contains("/api/v1/users/register"));
    assert!(yaml_config.contains("/api/v1/orders"));

    // Export in different formats
    let json_export = handler.export_session(session_id, "json").await.unwrap();
    assert!(serde_json::from_str::<serde_json::Value>(&json_export).is_ok());

    let har_export = handler.export_session(session_id, "har").await.unwrap();
    assert!(har_export.contains("\"log\""));
    assert!(har_export.contains("\"version\": \"1.2\""));

    handler.stop_session(session_id).await.unwrap();
}

/// Test capture filtering and performance monitoring
#[tokio::test]
async fn test_capture_filtering_and_monitoring() {
    let config = CaptureConfig {
        enabled: true,
        auto_start: Some(true),
        include_patterns: Some(vec!["/api/*".to_string()]),
        exclude_patterns: Some(vec![
            "/api/health".to_string(),
            "/api/metrics".to_string(),
            "*.css".to_string(),
            "*.js".to_string(),
        ]),
        methods: Some(vec!["GET".to_string(), "POST".to_string()]),
        max_requests: Some(100),
        storage_path: Some("./test_captures/monitoring".to_string()),
    };

    let handler = CaptureHandler::new(config);
    handler.start().await.unwrap();

    // Wait a bit for auto-start session to be created
    sleep(Duration::from_millis(10)).await;
    let sessions = handler.get_sessions().await;
    assert_eq!(sessions.len(), 1);
    let session_id = sessions[0].id;

    // Test various requests that should be filtered differently
    let test_cases = vec![
        ("/api/users", "GET", true),          // Should capture
        ("/api/posts", "POST", true),         // Should capture
        ("/api/health", "GET", false),        // Should exclude (health check)
        ("/static/style.css", "GET", false),  // Should exclude (CSS file)
        ("/api/data", "DELETE", false),       // Should exclude (method not allowed)
        ("/admin/panel", "GET", false),       // Should exclude (not in include pattern)
    ];

    for (path, method, should_capture) in test_cases {
        let req_id = handler.capture_request(
            method.to_string(),
            path.to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
        ).await.unwrap();

        if should_capture {
            assert_ne!(req_id, Uuid::nil(), "Request to {} {} should have been captured", method, path);
            
            handler.capture_response(
                req_id,
                200,
                HashMap::new(),
                Some(serde_json::json!({"path": path, "method": method})),
                Duration::from_millis(50),
            ).await.unwrap();
        } else {
            assert_eq!(req_id, Uuid::nil(), "Request to {} {} should NOT have been captured", method, path);
        }
    }

    // Verify only the expected requests were captured
    let captured_requests = handler.get_captured_requests(session_id, None).await;
    assert_eq!(captured_requests.len(), 2); // Only /api/users GET and /api/posts POST
    
    // Verify the captured requests are correct
    let paths: Vec<&str> = captured_requests.iter().map(|r| r.path.as_str()).collect();
    assert!(paths.contains(&"/api/users"));
    assert!(paths.contains(&"/api/posts"));
}

/// Test concurrent capture operations and thread safety
#[tokio::test]
async fn test_concurrent_capture_operations() {
    let config = CaptureConfig {
        enabled: true,
        auto_start: Some(false),
        include_patterns: None,
        exclude_patterns: None,
        methods: None,
        max_requests: Some(1000),
        storage_path: Some("./test_captures/concurrent".to_string()),
    };

    let handler = std::sync::Arc::new(CaptureHandler::new(config));
    let session_id = handler.start_session("concurrent_test".to_string()).await.unwrap();

    // Spawn multiple concurrent tasks
    let mut handles = vec![];
    let num_tasks = 10;
    let requests_per_task = 20;

    for task_id in 0..num_tasks {
        let handler_clone = std::sync::Arc::clone(&handler);
        let handle = tokio::spawn(async move {
            for req_id in 0..requests_per_task {
                let path = format!("/api/task/{}/request/{}", task_id, req_id);
                let method = if req_id % 2 == 0 { "GET" } else { "POST" };
                
                let request_id = handler_clone.capture_request(
                    method.to_string(),
                    path,
                    HashMap::new(),
                    HashMap::new(),
                    Some(serde_json::json!({
                        "task_id": task_id,
                        "request_id": req_id,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })),
                ).await.unwrap();

                handler_clone.capture_response(
                    request_id,
                    200,
                    HashMap::new(),
                    Some(serde_json::json!({
                        "success": true,
                        "task_id": task_id,
                        "request_id": req_id
                    })),
                    Duration::from_millis(10 + (req_id as u64 * 5)),
                ).await.unwrap();
                
                // Small delay to simulate realistic timing
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all requests were captured correctly
    let captured_requests = handler.get_captured_requests(session_id, None).await;
    let expected_total = num_tasks * requests_per_task;
    assert_eq!(captured_requests.len(), expected_total);

    // Verify session statistics
    let session = handler.get_session(session_id).await.unwrap();
    assert_eq!(session.request_count, expected_total as u64);
    assert!(matches!(session.status, CaptureStatus::Active));

    // Test filtering by different criteria
    let get_filter = CaptureFilter {
        methods: Some(vec!["GET".to_string()]),
        path_patterns: None,
        status_codes: None,
        min_duration: None,
        max_duration: None,
    };
    let get_requests = handler.get_captured_requests(session_id, Some(get_filter)).await;
    assert_eq!(get_requests.len(), expected_total / 2); // Half should be GET requests

    // Test duration filtering
    let slow_requests_filter = CaptureFilter {
        methods: None,
        path_patterns: None,
        status_codes: None,
        min_duration: Some(Duration::from_millis(50)),
        max_duration: None,
    };
    let slow_requests = handler.get_captured_requests(session_id, Some(slow_requests_filter)).await;
    assert!(slow_requests.len() > 0);
    
    println!("Concurrent test completed: {} total requests captured", captured_requests.len());
}

/// Test realistic data export and configuration generation
#[tokio::test]
async fn test_realistic_data_export() {
    let config = CaptureConfig {
        enabled: true,
        auto_start: Some(false),
        include_patterns: Some(vec!["/api/*".to_string()]),
        exclude_patterns: None,
        methods: None,
        max_requests: Some(500),
        storage_path: Some("./test_captures/export".to_string()),
    };

    let handler = CaptureHandler::new(config);
    let session_id = handler.start_session("export_test".to_string()).await.unwrap();

    // Create realistic API endpoints with various response patterns
    let endpoints = vec![
        ("/api/users", "GET", 200, serde_json::json!([
            {"id": 1, "name": "Alice", "email": "alice@example.com"},
            {"id": 2, "name": "Bob", "email": "bob@example.com"}
        ])),
        ("/api/users/1", "GET", 200, serde_json::json!({
            "id": 1, "name": "Alice", "email": "alice@example.com", 
            "profile": {"age": 30, "city": "New York"}
        })),
        ("/api/users", "POST", 201, serde_json::json!({
            "id": 3, "name": "Charlie", "email": "charlie@example.com", "created_at": "2024-01-15T10:30:00Z"
        })),
        ("/api/posts", "GET", 200, serde_json::json!([
            {"id": 1, "title": "Hello World", "author_id": 1},
            {"id": 2, "title": "Rust Programming", "author_id": 2}
        ])),
        ("/api/posts/1", "PUT", 200, serde_json::json!({
            "id": 1, "title": "Hello Updated World", "author_id": 1, "updated_at": "2024-01-15T11:00:00Z"
        })),
        ("/api/orders", "POST", 201, serde_json::json!({
            "order_id": "ord_123", "user_id": 1, "total": 99.99, "status": "pending"
        })),
        ("/api/health", "GET", 200, serde_json::json!({"status": "healthy"})),
    ];

    for (path, method, status, response_body) in endpoints {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        
        let request_body = if method == "POST" || method == "PUT" {
            Some(serde_json::json!({"test": "data"}))
        } else {
            None
        };

        let req_id = handler.capture_request(
            method.to_string(),
            path.to_string(),
            headers.clone(),
            HashMap::new(),
            request_body,
        ).await.unwrap();

        if req_id != Uuid::nil() {
            handler.capture_response(
                req_id,
                status,
                headers,
                Some(response_body),
                Duration::from_millis(50 + (status as u64 - 200) * 10),
            ).await.unwrap();
        }
    }

    // Test JSON export
    let json_export = handler.export_session(session_id, "json").await.unwrap();
    let json_data: serde_json::Value = serde_json::from_str(&json_export).unwrap();
    
    assert!(json_data["session"].is_object());
    assert!(json_data["requests"].is_array());
    
    let requests_array = json_data["requests"].as_array().unwrap();
    assert!(requests_array.len() > 0);

    // Test YAML configuration generation
    let yaml_config = handler.generate_api_from_capture(session_id).await.unwrap();
    
    // Verify YAML structure
    assert!(yaml_config.contains("name: captured_api"));
    assert!(yaml_config.contains("endpoints:"));
    assert!(yaml_config.contains("path: /api/users"));
    assert!(yaml_config.contains("method: GET"));
    assert!(yaml_config.contains("path: /api/users/{id}"));
    assert!(yaml_config.contains("mode: mock"));
    
    // Test HAR export
    let har_export = handler.export_session(session_id, "har").await.unwrap();
    let har_data: serde_json::Value = serde_json::from_str(&har_export).unwrap();
    
    assert_eq!(har_data["log"]["version"], "1.2");
    assert_eq!(har_data["log"]["creator"]["name"], "Backworks");
    assert!(har_data["log"]["entries"].is_array());
    
    let entries = har_data["log"]["entries"].as_array().unwrap();
    assert!(entries.len() > 0);
    
    // Verify HAR entry structure
    let first_entry = &entries[0];
    assert!(first_entry["startedDateTime"].is_string());
    assert!(first_entry["request"].is_object());
    assert!(first_entry["response"].is_object());
    
    println!("Export test completed successfully");
    println!("JSON export size: {} characters", json_export.len());
    println!("YAML config size: {} characters", yaml_config.len());
    println!("HAR export size: {} characters", har_export.len());
}

/// Test capturer utility functions
#[tokio::test]
async fn test_capturer_utility_functions() {
    let temp_dir = std::env::temp_dir();
    let output_file = temp_dir.join("test_capture_output.txt");
    let capturer = Capturer::new(9090, output_file.to_string_lossy().to_string());

    // Test basic start functionality
    let result = capturer.start().await;
    assert!(result.is_ok());
    
    // Verify file was created with expected content
    let content = tokio::fs::read_to_string(&output_file).await.unwrap();
    assert!(content.contains("Simulated capture data"));
    assert!(content.contains("9090"));

    // Test timed capture
    let start_time = std::time::Instant::now();
    let result = capturer.capture_for_duration(Duration::from_millis(100)).await;
    let elapsed = start_time.elapsed();
    
    assert!(result.is_ok());
    assert!(elapsed >= Duration::from_millis(90)); // Allow some tolerance for timing
    assert!(elapsed < Duration::from_millis(200)); // But not too much

    // Test config file generation
    let input_path = temp_dir.join("input_capture.json");
    let output_config_path = temp_dir.join("generated_config.yaml");
    
    // Create a dummy input file
    tokio::fs::write(&input_path, r#"{"test": "data"}"#).await.unwrap();
    
    let result = capturer.generate_from_file(input_path.clone(), output_config_path.clone()).await;
    assert!(result.is_ok());
    
    // Verify generated config file
    let config_content = tokio::fs::read_to_string(&output_config_path).await.unwrap();
    assert!(config_content.contains("Generated Backworks config"));
    assert!(config_content.contains("name: generated-api"));
    assert!(config_content.contains(&format!("{:?}", input_path)));

    // Test from captured data
    let captured_data = vec![];
    let result = capturer.from_captured_data(&captured_data).await;
    assert!(result.is_ok());
    
    let generated_config = result.unwrap();
    assert!(generated_config.contains("Generated Backworks config"));
    assert!(generated_config.contains("name: generated-api"));

    // Clean up test files
    let _ = tokio::fs::remove_file(&output_file).await;
    let _ = tokio::fs::remove_file(&input_path).await;
    let _ = tokio::fs::remove_file(&output_config_path).await;
}
