use crate::config::CaptureConfig;
use crate::error::{BackworksError, BackworksResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedRequest {
    pub id: Uuid,
    pub session_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
    pub response: Option<CapturedResponse>,
    pub response_status: Option<u16>,
    pub response_headers: Option<HashMap<String, String>>,
    pub response_body: Option<String>,
    pub duration: Option<std::time::Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSession {
    pub id: Uuid,
    pub name: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub request_count: u64,
    pub status: CaptureStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptureStatus {
    Active,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureFilter {
    pub methods: Option<Vec<String>>,
    pub path_patterns: Option<Vec<String>>,
    pub status_codes: Option<Vec<u16>>,
    pub min_duration: Option<std::time::Duration>,
    pub max_duration: Option<std::time::Duration>,
}

#[derive(Debug)]
pub struct CaptureHandler {
    config: CaptureConfig,
    sessions: Arc<RwLock<HashMap<Uuid, CaptureSession>>>,
    captured_requests: Arc<RwLock<HashMap<Uuid, Vec<CapturedRequest>>>>,
    active_session: Arc<RwLock<Option<Uuid>>>,
}

impl Clone for CaptureHandler {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            sessions: Arc::clone(&self.sessions),
            captured_requests: Arc::clone(&self.captured_requests),
            active_session: Arc::clone(&self.active_session),
        }
    }
}

impl CaptureHandler {
    pub fn new(config: CaptureConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            captured_requests: Arc::new(RwLock::new(HashMap::new())),
            active_session: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start(&self) -> BackworksResult<()> {
        tracing::info!("Starting capture handler");
        
        if self.config.auto_start.unwrap_or(false) {
            self.start_session("auto_session".to_string()).await?;
        }
        
        Ok(())
    }

    pub async fn start_session(&self, name: String) -> BackworksResult<Uuid> {
        let session_id = Uuid::new_v4();
        let session = CaptureSession {
            id: session_id,
            name,
            started_at: chrono::Utc::now(),
            ended_at: None,
            request_count: 0,
            status: CaptureStatus::Active,
        };
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, session);
        
        let mut captured_requests = self.captured_requests.write().await;
        captured_requests.insert(session_id, Vec::new());
        
        let mut active_session = self.active_session.write().await;
        *active_session = Some(session_id);
        
        tracing::info!("Started capture session: {}", session_id);
        Ok(session_id)
    }

    pub async fn stop_session(&self, session_id: Uuid) -> BackworksResult<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.status = CaptureStatus::Stopped;
            session.ended_at = Some(chrono::Utc::now());
            
            let mut active_session = self.active_session.write().await;
            if *active_session == Some(session_id) {
                *active_session = None;
            }
            
            tracing::info!("Stopped capture session: {}", session_id);
        }
        
        Ok(())
    }

    pub async fn pause_session(&self, session_id: Uuid) -> BackworksResult<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.status = CaptureStatus::Paused;
            tracing::info!("Paused capture session: {}", session_id);
        }
        
        Ok(())
    }

    pub async fn resume_session(&self, session_id: Uuid) -> BackworksResult<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.status = CaptureStatus::Active;
            tracing::info!("Resumed capture session: {}", session_id);
        }
        
        Ok(())
    }

    pub async fn capture_request(
        &self,
        method: String,
        path: String,
        headers: HashMap<String, String>,
        query_params: HashMap<String, String>,
        body: Option<serde_json::Value>,
    ) -> BackworksResult<Uuid> {
        let active_session = self.active_session.read().await;
        let session_id = match *active_session {
            Some(id) => id,
            None => return Ok(Uuid::nil()), // No active session
        };
        
        // Check if session is active
        let sessions = self.sessions.read().await;
        let session = sessions.get(&session_id);
        if let Some(session) = session {
            if !matches!(session.status, CaptureStatus::Active) {
                return Ok(Uuid::nil()); // Session not active
            }
        } else {
            return Ok(Uuid::nil()); // Session not found
        }
        drop(sessions);
        
        // Apply filters
        if !self.should_capture(&method, &path, &headers, &query_params).await {
            return Ok(Uuid::nil());
        }
        
        let request_id = Uuid::new_v4();
        let captured_request = CapturedRequest {
            id: request_id,
            session_id: None,
            timestamp: chrono::Utc::now(),
            method,
            path,
            headers,
            query_params,
            body,
            response: None,
            response_status: None,
            response_headers: None,
            response_body: None,
            duration: None,
        };
        
        let mut captured_requests = self.captured_requests.write().await;
        if let Some(requests) = captured_requests.get_mut(&session_id) {
            requests.push(captured_request);
            
            // Update session request count
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&session_id) {
                session.request_count += 1;
            }
        }
        
        tracing::debug!("Captured request: {} in session: {}", request_id, session_id);
        Ok(request_id)
    }

    pub async fn capture_response(
        &self,
        request_id: Uuid,
        status_code: u16,
        headers: HashMap<String, String>,
        body: Option<serde_json::Value>,
        duration: std::time::Duration,
    ) -> BackworksResult<()> {
        let captured_response = CapturedResponse {
            status_code,
            headers,
            body,
        };
        
        let mut captured_requests = self.captured_requests.write().await;
        for (_, requests) in captured_requests.iter_mut() {
            if let Some(request) = requests.iter_mut().find(|r| r.id == request_id) {
                request.response = Some(captured_response);
                request.duration = Some(duration);
                tracing::debug!("Captured response for request: {}", request_id);
                break;
            }
        }
        
        Ok(())
    }

    pub async fn get_sessions(&self) -> Vec<CaptureSession> {
        self.sessions.read().await.values().cloned().collect()
    }

    pub async fn get_session(&self, session_id: Uuid) -> Option<CaptureSession> {
        self.sessions.read().await.get(&session_id).cloned()
    }

    pub async fn get_captured_requests(&self, session_id: Uuid, filter: Option<CaptureFilter>) -> Vec<CapturedRequest> {
        let captured_requests = self.captured_requests.read().await;
        let requests = captured_requests.get(&session_id).cloned().unwrap_or_default();
        
        if let Some(filter) = filter {
            self.apply_filter(requests, filter)
        } else {
            requests
        }
    }

    pub async fn export_session(&self, session_id: Uuid, format: &str) -> BackworksResult<String> {
        let session = self.get_session(session_id).await
            .ok_or_else(|| crate::error::BackworksError::Config(format!("Session not found: {}", session_id)))?;
        
        let requests = self.get_captured_requests(session_id, None).await;
        
        match format.to_lowercase().as_str() {
            "json" => {
                let export_data = serde_json::json!({
                    "session": session,
                    "requests": requests
                });
                Ok(serde_json::to_string_pretty(&export_data)?)
            }
            "yaml" => {
                // Generate YAML configuration from captured requests
                self.generate_yaml_config(requests).await
            }
            "har" => {
                // Generate HAR (HTTP Archive) format
                self.generate_har_format(session, requests).await
            }
            _ => Err(crate::error::BackworksError::Config(format!("Unsupported export format: {}", format)))
        }
    }

    pub async fn generate_api_from_capture(&self, session_id: Uuid) -> BackworksResult<String> {
        let requests = self.get_captured_requests(session_id, None).await;
        self.generate_yaml_config(requests).await
    }

    async fn should_capture(
        &self,
        method: &str,
        path: &str,
        _headers: &HashMap<String, String>,
        _query_params: &HashMap<String, String>,
    ) -> bool {
        // Apply include/exclude filters
        if let Some(include_patterns) = &self.config.include_patterns {
            let matches = include_patterns.iter().any(|pattern| {
                path.contains(pattern) || glob::Pattern::new(pattern).map(|p| p.matches(path)).unwrap_or(false)
            });
            if !matches {
                return false;
            }
        }
        
        if let Some(exclude_patterns) = &self.config.exclude_patterns {
            let matches = exclude_patterns.iter().any(|pattern| {
                path.contains(pattern) || glob::Pattern::new(pattern).map(|p| p.matches(path)).unwrap_or(false)
            });
            if matches {
                return false;
            }
        }
        
        // Apply method filter
        if let Some(methods) = &self.config.methods {
            if !methods.contains(&method.to_string()) {
                return false;
            }
        }
        
        true
    }

    fn apply_filter(&self, requests: Vec<CapturedRequest>, filter: CaptureFilter) -> Vec<CapturedRequest> {
        requests.into_iter()
            .filter(|request| {
                // Filter by methods
                if let Some(methods) = &filter.methods {
                    if !methods.contains(&request.method) {
                        return false;
                    }
                }
                
                // Filter by path patterns
                if let Some(patterns) = &filter.path_patterns {
                    let matches = patterns.iter().any(|pattern| {
                        request.path.contains(pattern) || 
                        glob::Pattern::new(pattern).map(|p| p.matches(&request.path)).unwrap_or(false)
                    });
                    if !matches {
                        return false;
                    }
                }
                
                // Filter by status codes
                if let Some(status_codes) = &filter.status_codes {
                    if let Some(response) = &request.response {
                        if !status_codes.contains(&response.status_code) {
                            return false;
                        }
                    } else {
                        return false; // No response captured
                    }
                }
                
                // Filter by duration
                if let Some(duration) = request.duration {
                    if let Some(min_duration) = filter.min_duration {
                        if duration < min_duration {
                            return false;
                        }
                    }
                    if let Some(max_duration) = filter.max_duration {
                        if duration > max_duration {
                            return false;
                        }
                    }
                }
                
                true
            })
            .collect()
    }

    async fn generate_yaml_config(&self, requests: Vec<CapturedRequest>) -> BackworksResult<String> {
        let mut yaml = String::new();
        yaml.push_str("# Generated API configuration from captured requests\n");
        yaml.push_str("name: captured_api\n");
        yaml.push_str("version: 1.0.0\n");
        yaml.push_str("endpoints:\n");
        
        // Group requests by method and path pattern
        let mut endpoint_groups: HashMap<(String, String), Vec<&CapturedRequest>> = HashMap::new();
        
        for request in &requests {
            let path_pattern = self.extract_path_pattern(&request.path);
            let key = (request.method.clone(), path_pattern);
            endpoint_groups.entry(key).or_insert_with(Vec::new).push(request);
        }
        
        for ((method, path), group_requests) in endpoint_groups {
            yaml.push_str(&format!("  - path: {}\n", path));
            yaml.push_str(&format!("    method: {}\n", method));
            yaml.push_str("    mode: mock\n");
            yaml.push_str("    mock:\n");
            
            // Generate response based on captured responses
            if let Some(first_request) = group_requests.first() {
                if let Some(response) = &first_request.response {
                    yaml.push_str(&format!("      status: {}\n", response.status_code));
                    
                    if !response.headers.is_empty() {
                        yaml.push_str("      headers:\n");
                        for (key, value) in &response.headers {
                            if key.to_lowercase() != "content-length" {
                                yaml.push_str(&format!("        {}: \"{}\"\n", key, value));
                            }
                        }
                    }
                    
                    if let Some(body) = &response.body {
                        yaml.push_str("      body: |\n");
                        let body_str = serde_json::to_string_pretty(body)?;
                        for line in body_str.lines() {
                            yaml.push_str(&format!("        {}\n", line));
                        }
                    }
                }
            }
            
            yaml.push_str("\n");
        }
        
        Ok(yaml)
    }

    async fn generate_har_format(&self, _session: CaptureSession, requests: Vec<CapturedRequest>) -> BackworksResult<String> {
        let har_data = serde_json::json!({
            "log": {
                "version": "1.2",
                "creator": {
                    "name": "Backworks",
                    "version": "1.0.0"
                },
                "entries": requests.iter().map(|request| {
                    serde_json::json!({
                        "startedDateTime": request.timestamp.to_rfc3339(),
                        "time": request.duration.map(|d| d.as_millis()).unwrap_or(0),
                        "request": {
                            "method": request.method,
                            "url": format!("http://localhost{}", request.path),
                            "headers": request.headers.iter().map(|(k, v)| {
                                serde_json::json!({"name": k, "value": v})
                            }).collect::<Vec<_>>(),
                            "queryString": request.query_params.iter().map(|(k, v)| {
                                serde_json::json!({"name": k, "value": v})
                            }).collect::<Vec<_>>(),
                            "postData": request.body.as_ref().map(|body| {
                                serde_json::json!({
                                    "mimeType": "application/json",
                                    "text": body.to_string()
                                })
                            })
                        },
                        "response": request.response.as_ref().map(|response| {
                            serde_json::json!({
                                "status": response.status_code,
                                "statusText": "",
                                "headers": response.headers.iter().map(|(k, v)| {
                                    serde_json::json!({"name": k, "value": v})
                                }).collect::<Vec<_>>(),
                                "content": {
                                    "mimeType": response.headers.get("content-type").unwrap_or(&"application/json".to_string()),
                                    "text": response.body.as_ref().map(|b| b.to_string()).unwrap_or_default()
                                }
                            })
                        })
                    })
                }).collect::<Vec<_>>()
            }
        });
        
        Ok(serde_json::to_string_pretty(&har_data)?)
    }

    fn extract_path_pattern(&self, path: &str) -> String {
        // Simple pattern extraction: replace numeric segments and UUIDs with placeholders
        let segments: Vec<&str> = path.split('/').collect();
        let pattern_segments: Vec<String> = segments
            .iter()
            .map(|segment| {
                if segment.parse::<i64>().is_ok() {
                    "{id}".to_string()
                } else if segment.parse::<uuid::Uuid>().is_ok() {
                    "{uuid}".to_string()
                } else if segment.len() > 10 && segment.chars().all(|c| c.is_alphanumeric()) {
                    "{token}".to_string() // Likely a token or hash
                } else {
                    segment.to_string()
                }
            })
            .collect();
        
        pattern_segments.join("/")
    }

    pub async fn handle_request(&self, endpoint_name: &str, request_data: &crate::server::RequestData) -> crate::error::BackworksResult<String> {
        // Capture the request if we have an active session
        if let Some(session_id) = *self.active_session.read().await {
            let captured_request = CapturedRequest {
                id: uuid::Uuid::new_v4(),
                session_id: Some(session_id.to_string()),
                timestamp: chrono::Utc::now(),
                method: request_data.method.clone(),
                path: request_data.path_params.get("path").unwrap_or(&"".to_string()).clone(),
                headers: request_data.headers.iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect(),
                query_params: request_data.query_params.clone(),
                body: request_data.body.clone(),
                response: None,
                response_status: None,
                response_headers: None,
                response_body: None,
                duration: None,
            };

            let mut captured_requests = self.captured_requests.write().await;
            if let Some(requests) = captured_requests.get_mut(&session_id) {
                requests.push(captured_request);
            }

            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&session_id) {
                session.request_count += 1;
            }

            tracing::info!("Captured request for endpoint: {}", endpoint_name);
        }

        // Return a capture acknowledgment response
        let response = serde_json::json!({
            "captured": true,
            "endpoint": endpoint_name,
            "method": request_data.method,
            "message": "Request captured successfully"
        });

        Ok(response.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Capturer {
    port: u16,
    output: String,
}

impl Capturer {
    pub fn new(port: u16, output: String) -> Self {
        Self { port, output }
    }
    
    pub async fn start(&self) -> BackworksResult<()> {
        tracing::info!("Starting capture on port {} with output {}", self.port, self.output);
        // Simulate capture logic by writing a placeholder to the output file
        let content = format!("# Simulated capture data\nCaptured on port: {}\n", self.port);
        tokio::fs::write(&self.output, content)
            .await
            .map_err(|e| BackworksError::Io(e))?;
        Ok(())
    }

    pub async fn capture_for_duration(&self, duration: std::time::Duration) -> BackworksResult<()> {
        tracing::info!("Starting capture for {} seconds on port {}", duration.as_secs(), self.port);
        // Simulate capture logic
        tokio::time::sleep(duration).await;
        tokio::fs::write(&self.output, format!("# Simulated capture data\nCaptured for {} seconds on port: {}\n", duration.as_secs(), self.port))
            .await
            .map_err(|e| BackworksError::Io(e))?;
        tracing::info!("Capture completed");
        Ok(())
    }

    pub async fn capture_indefinitely(&self) -> BackworksResult<()> {
        tracing::info!("Starting indefinite capture on port {}", self.port);
        // Simulate indefinite capture by appending to the output file every minute
        loop {
            let content = format!("# Simulated capture data\nCaptured indefinitely on port: {}\n", self.port);
            tokio::fs::write(&self.output, content)
                .await
                .map_err(|e| BackworksError::Io(e))?;
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }

    pub async fn generate_from_file(&self, input: std::path::PathBuf, output: std::path::PathBuf) -> BackworksResult<()> {
        tracing::info!("Generating config from file: {:?} to {:?}", input, output);
        // Simulate file-based config generation
        let config_content = format!("# Generated Backworks config\nname: generated-api\nendpoints: {{}}\nSource: {:?}", input);
        tokio::fs::write(output, config_content)
            .await
            .map_err(|e| BackworksError::Io(e))?;
        Ok(())
    }

    pub async fn from_captured_data(&self, _data: &[CapturedRequest]) -> BackworksResult<String> {
        // Simulate config generation from captured data
        Ok("# Generated Backworks config\nname: generated-api\nendpoints: {}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    fn create_test_capture_config() -> CaptureConfig {
        CaptureConfig {
            enabled: Some(true),
            auto_start: Some(false),
            include_patterns: None,
            exclude_patterns: None,
            methods: None,
            analyze: Some(true),
            learn_schema: Some(true),
        }
    }

    fn create_filtered_capture_config() -> CaptureConfig {
        CaptureConfig {
            enabled: Some(true),
            auto_start: Some(false),
            include_patterns: Some(vec!["/api/*".to_string(), "/v1/*".to_string()]),
            exclude_patterns: Some(vec!["/health".to_string(), "*.css".to_string()]),
            methods: Some(vec!["GET".to_string(), "POST".to_string()]),
            analyze: Some(true),
            learn_schema: Some(true),
        }
    }

    #[tokio::test]
    async fn test_capture_handler_creation() {
        let config = create_test_capture_config();
        let handler = CaptureHandler::new(config);
        
        assert!(handler.start().await.is_ok());
        
        // Test that no sessions exist initially
        let sessions = handler.get_sessions().await;
        assert_eq!(sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_capture_handler_auto_start() {
        let mut config = create_test_capture_config();
        config.auto_start = Some(true);
        let handler = CaptureHandler::new(config);
        
        assert!(handler.start().await.is_ok());
        
        // Auto-start should create a session
        let sessions = handler.get_sessions().await;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "auto_session");
    }

    #[tokio::test]
    async fn test_capture_session_lifecycle() {
        let config = create_test_capture_config();
        let handler = CaptureHandler::new(config);
        
        let session_id = handler.start_session("test_session".to_string()).await.unwrap();
        
        let sessions = handler.get_sessions().await;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "test_session");
        assert!(matches!(sessions[0].status, CaptureStatus::Active));
        
        handler.pause_session(session_id).await.unwrap();
        let session = handler.get_session(session_id).await.unwrap();
        assert!(matches!(session.status, CaptureStatus::Paused));
        
        handler.resume_session(session_id).await.unwrap();
        let session = handler.get_session(session_id).await.unwrap();
        assert!(matches!(session.status, CaptureStatus::Active));
        
        handler.stop_session(session_id).await.unwrap();
        let session = handler.get_session(session_id).await.unwrap();
        assert!(matches!(session.status, CaptureStatus::Stopped));
        assert!(session.ended_at.is_some());
    }

    #[tokio::test]
    async fn test_multiple_sessions() {
        let config = create_test_capture_config();
        let handler = CaptureHandler::new(config);
        
        let _session1 = handler.start_session("session_1".to_string()).await.unwrap();
        let _session2 = handler.start_session("session_2".to_string()).await.unwrap();
        let session3 = handler.start_session("session_3".to_string()).await.unwrap();
        
        let sessions = handler.get_sessions().await;
        assert_eq!(sessions.len(), 3);
        
        // Only the last session should be active
        let active_session = *handler.active_session.read().await;
        assert_eq!(active_session, Some(session3));
        
        // Stop the active session
        handler.stop_session(session3).await.unwrap();
        let active_session = *handler.active_session.read().await;
        assert_eq!(active_session, None);
    }

    #[tokio::test]
    async fn test_request_capture() {
        let config = create_test_capture_config();
        let handler = CaptureHandler::new(config);
        
        let session_id = handler.start_session("test_session".to_string()).await.unwrap();
        
        let request_id = handler.capture_request(
            "GET".to_string(),
            "/users/123".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
        ).await.unwrap();
        
        assert_ne!(request_id, Uuid::nil());
        
        handler.capture_response(
            request_id,
            200,
            HashMap::new(),
            Some(serde_json::json!({"id": 123, "name": "Test User"})),
            std::time::Duration::from_millis(100),
        ).await.unwrap();
        
        let requests = handler.get_captured_requests(session_id, None).await;
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[0].path, "/users/123");
        assert!(requests[0].response.is_some());
        assert_eq!(requests[0].response.as_ref().unwrap().status_code, 200);
        assert_eq!(requests[0].duration, Some(std::time::Duration::from_millis(100)));
        
        // Check session request count
        let session = handler.get_session(session_id).await.unwrap();
        assert_eq!(session.request_count, 1);
    }

    #[tokio::test]
    async fn test_request_capture_no_active_session() {
        let config = create_test_capture_config();
        let handler = CaptureHandler::new(config);
        
        // No active session
        let request_id = handler.capture_request(
            "GET".to_string(),
            "/users/123".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
        ).await.unwrap();
        
        // Should return nil UUID when no active session
        assert_eq!(request_id, Uuid::nil());
    }

    #[tokio::test]
    async fn test_request_capture_paused_session() {
        let config = create_test_capture_config();
        let handler = CaptureHandler::new(config);
        
        let session_id = handler.start_session("test_session".to_string()).await.unwrap();
        handler.pause_session(session_id).await.unwrap();
        
        let request_id = handler.capture_request(
            "GET".to_string(),
            "/users/123".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
        ).await.unwrap();
        
        // Should return nil UUID when session is paused
        assert_eq!(request_id, Uuid::nil());
    }

    #[tokio::test]
    async fn test_capture_filtering() {
        let config = create_filtered_capture_config();
        let handler = CaptureHandler::new(config);
        
        let session_id = handler.start_session("filtered_session".to_string()).await.unwrap();
        
        // Should capture: matches include pattern and method
        let request_id1 = handler.capture_request(
            "GET".to_string(),
            "/api/users".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
        ).await.unwrap();
        assert_ne!(request_id1, Uuid::nil());
        
        // Should not capture: doesn't match include pattern
        let request_id2 = handler.capture_request(
            "GET".to_string(),
            "/admin/users".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
        ).await.unwrap();
        assert_eq!(request_id2, Uuid::nil());
        
        // Should not capture: matches exclude pattern
        let request_id3 = handler.capture_request(
            "GET".to_string(),
            "/health".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
        ).await.unwrap();
        assert_eq!(request_id3, Uuid::nil());
        
        // Should not capture: method not allowed
        let request_id4 = handler.capture_request(
            "DELETE".to_string(),
            "/api/users/123".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
        ).await.unwrap();
        assert_eq!(request_id4, Uuid::nil());
        
        let requests = handler.get_captured_requests(session_id, None).await;
        assert_eq!(requests.len(), 1);
    }

    #[tokio::test]
    async fn test_request_filtering_by_capture_filter() {
        let config = create_test_capture_config();
        let handler = CaptureHandler::new(config);
        
        let session_id = handler.start_session("filter_test".to_string()).await.unwrap();
        
        // Add multiple requests with different characteristics
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        
        // GET request
        let req1_id = handler.capture_request(
            "GET".to_string(),
            "/api/users".to_string(),
            headers.clone(),
            HashMap::new(),
            None,
        ).await.unwrap();
        handler.capture_response(
            req1_id,
            200,
            HashMap::new(),
            Some(serde_json::json!({"users": []})),
            Duration::from_millis(50),
        ).await.unwrap();
        
        // POST request
        let req2_id = handler.capture_request(
            "POST".to_string(),
            "/api/orders".to_string(),
            headers.clone(),
            HashMap::new(),
            Some(serde_json::json!({"product_id": 123})),
        ).await.unwrap();
        handler.capture_response(
            req2_id,
            201,
            HashMap::new(),
            Some(serde_json::json!({"order_id": 456})),
            Duration::from_millis(150),
        ).await.unwrap();
        
        // PUT request
        let req3_id = handler.capture_request(
            "PUT".to_string(),
            "/api/products/789".to_string(),
            headers,
            HashMap::new(),
            Some(serde_json::json!({"name": "Updated Product"})),
        ).await.unwrap();
        handler.capture_response(
            req3_id,
            500,
            HashMap::new(),
            Some(serde_json::json!({"error": "Internal error"})),
            Duration::from_millis(300),
        ).await.unwrap();
        
        // Test filtering by method
        let filter = CaptureFilter {
            methods: Some(vec!["GET".to_string(), "POST".to_string()]),
            path_patterns: None,
            status_codes: None,
            min_duration: None,
            max_duration: None,
        };
        let filtered_requests = handler.get_captured_requests(session_id, Some(filter)).await;
        assert_eq!(filtered_requests.len(), 2);
        
        // Test filtering by status code
        let filter = CaptureFilter {
            methods: None,
            path_patterns: None,
            status_codes: Some(vec![200, 201]),
            min_duration: None,
            max_duration: None,
        };
        let filtered_requests = handler.get_captured_requests(session_id, Some(filter)).await;
        assert_eq!(filtered_requests.len(), 2);
        
        // Test filtering by duration
        let filter = CaptureFilter {
            methods: None,
            path_patterns: None,
            status_codes: None,
            min_duration: Some(Duration::from_millis(100)),
            max_duration: Some(Duration::from_millis(200)),
        };
        let filtered_requests = handler.get_captured_requests(session_id, Some(filter)).await;
        assert_eq!(filtered_requests.len(), 1);
        assert_eq!(filtered_requests[0].method, "POST");
    }

    #[tokio::test]
    async fn test_path_pattern_extraction() {
        let config = create_test_capture_config();
        let handler = CaptureHandler::new(config);
        
        assert_eq!(handler.extract_path_pattern("/users/123"), "/users/{id}");
        assert_eq!(handler.extract_path_pattern("/api/v1/posts/456/comments"), "/api/v1/posts/{id}/comments");
        assert_eq!(handler.extract_path_pattern("/auth/token/abc123def456"), "/auth/token/{token}");
        assert_eq!(handler.extract_path_pattern("/orders/550e8400-e29b-41d4-a716-446655440000"), "/orders/{uuid}");
        assert_eq!(handler.extract_path_pattern("/api/v2/users/profile"), "/api/v2/users/profile");
    }

    #[tokio::test]
    async fn test_yaml_config_generation() {
        let config = create_test_capture_config();
        let handler = CaptureHandler::new(config);
        
        let session_id = handler.start_session("yaml_test".to_string()).await.unwrap();
        
        // Create sample requests
        let req1_id = handler.capture_request(
            "GET".to_string(),
            "/api/users/123".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
        ).await.unwrap();
        
        let mut response_headers = HashMap::new();
        response_headers.insert("content-type".to_string(), "application/json".to_string());
        
        handler.capture_response(
            req1_id,
            200,
            response_headers,
            Some(serde_json::json!({"id": 123, "name": "John Doe"})),
            Duration::from_millis(100),
        ).await.unwrap();
        
        let yaml_config = handler.generate_api_from_capture(session_id).await.unwrap();
        
        assert!(yaml_config.contains("name: captured_api"));
        assert!(yaml_config.contains("endpoints:"));
        assert!(yaml_config.contains("path: /api/users/{id}"));
        assert!(yaml_config.contains("method: GET"));
        assert!(yaml_config.contains("status: 200"));
    }

    #[tokio::test]
    async fn test_export_formats() {
        let config = create_test_capture_config();
        let handler = CaptureHandler::new(config);
        
        let session_id = handler.start_session("export_test".to_string()).await.unwrap();
        
        // Add a sample request
        let req_id = handler.capture_request(
            "GET".to_string(),
            "/api/test".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
        ).await.unwrap();
        
        handler.capture_response(
            req_id,
            200,
            HashMap::new(),
            Some(serde_json::json!({"message": "test"})),
            Duration::from_millis(50),
        ).await.unwrap();
        
        // Test JSON export
        let json_export = handler.export_session(session_id, "json").await.unwrap();
        assert!(json_export.contains("\"session\""));
        assert!(json_export.contains("\"requests\""));
        
        // Test YAML export
        let yaml_export = handler.export_session(session_id, "yaml").await.unwrap();
        assert!(yaml_export.contains("name: captured_api"));
        
        // Test HAR export
        let har_export = handler.export_session(session_id, "har").await.unwrap();
        assert!(har_export.contains("\"log\""));
        assert!(har_export.contains("\"version\": \"1.2\""));
        assert!(har_export.contains("\"entries\""));
        
        // Test unsupported format
        let result = handler.export_session(session_id, "xml").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_capturer_utility() {
        let capturer = Capturer::new(8080, "/tmp/test_capture.txt".to_string());
        
        // Test basic capture
        assert!(capturer.start().await.is_ok());
        
        // Test timed capture
        let start_time = std::time::Instant::now();
        assert!(capturer.capture_for_duration(Duration::from_millis(100)).await.is_ok());
        let elapsed = start_time.elapsed();
        assert!(elapsed >= Duration::from_millis(90)); // Allow some tolerance
        
        // Test config generation
        let input_path = std::path::PathBuf::from("/tmp/input.json");
        let output_path = std::path::PathBuf::from("/tmp/output.yaml");
        assert!(capturer.generate_from_file(input_path, output_path).await.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_capture_operations() {
        let config = create_test_capture_config();
        let handler = Arc::new(CaptureHandler::new(config));
        
        let session_id = handler.start_session("concurrent_test".to_string()).await.unwrap();
        
        // Spawn multiple concurrent capture operations
        let mut handles = vec![];
        
        for i in 0..10 {
            let handler_clone = Arc::clone(&handler);
            let handle = tokio::spawn(async move {
                let request_id = handler_clone.capture_request(
                    "GET".to_string(),
                    format!("/api/test/{}", i),
                    HashMap::new(),
                    HashMap::new(),
                    None,
                ).await.unwrap();
                
                handler_clone.capture_response(
                    request_id,
                    200,
                    HashMap::new(),
                    Some(serde_json::json!({"id": i})),
                    Duration::from_millis(10),
                ).await.unwrap();
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        let requests = handler.get_captured_requests(session_id, None).await;
        assert_eq!(requests.len(), 10);
        
        let session = handler.get_session(session_id).await.unwrap();
        assert_eq!(session.request_count, 10);
    }

    #[tokio::test]
    async fn test_performance_with_large_dataset() {
        let config = create_test_capture_config();
        let handler = CaptureHandler::new(config);
        
        let session_id = handler.start_session("performance_test".to_string()).await.unwrap();
        
        let start_time = std::time::Instant::now();
        
        // Capture 1000 requests
        for i in 0..1000 {
            let request_id = handler.capture_request(
                "GET".to_string(),
                format!("/api/items/{}", i),
                HashMap::new(),
                HashMap::new(),
                Some(serde_json::json!({"item_id": i})),
            ).await.unwrap();
            
            handler.capture_response(
                request_id,
                200,
                HashMap::new(),
                Some(serde_json::json!({"result": format!("item_{}", i)})),
                Duration::from_millis(1),
            ).await.unwrap();
        }
        
        let elapsed = start_time.elapsed();
        println!("Captured 1000 requests in {:?}", elapsed);
        
        // Verify all requests were captured
        let requests = handler.get_captured_requests(session_id, None).await;
        assert_eq!(requests.len(), 1000);
        
        // Test retrieval performance
        let start_time = std::time::Instant::now();
        let _requests = handler.get_captured_requests(session_id, None).await;
        let retrieval_time = start_time.elapsed();
        println!("Retrieved 1000 requests in {:?}", retrieval_time);
        
        // Should be reasonably fast (less than 100ms for 1000 requests)
        assert!(retrieval_time < Duration::from_millis(100));
    }
}
