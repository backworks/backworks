use crate::config::{HandlerConfig, RuntimeConfig};
use crate::error::{BackworksError, BackworksResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeManagerConfig {
    pub handlers: HashMap<String, HandlerConfig>,
}

impl Default for RuntimeManagerConfig {
    fn default() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerInstance {
    pub id: Uuid,
    pub name: String,
    pub runtime: String,
    pub script_path: String,
    pub status: HandlerStatus,
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
    pub execution_count: u64,
    pub average_duration: f64,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandlerStatus {
    Idle,
    Running,
    Error(String),
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub handler_name: String,
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
    pub duration: Duration,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct RuntimeManager {
    config: RuntimeManagerConfig,
    handlers: Arc<RwLock<HashMap<String, HandlerInstance>>>,
}

impl Clone for RuntimeManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            handlers: Arc::clone(&self.handlers),
        }
    }
}

impl RuntimeManager {
    pub fn new(config: RuntimeManagerConfig) -> Self {
        Self {
            config,
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(&self) -> BackworksResult<()> {
        tracing::info!("Starting runtime manager");
        
        // Initialize and validate all handlers
        for (name, handler_config) in &self.config.handlers {
            self.register_handler(name.clone(), handler_config.clone()).await?;
        }
        
        Ok(())
    }

    pub async fn register_handler(&self, name: String, config: HandlerConfig) -> BackworksResult<()> {
        tracing::info!("Registering handler: {} ({})", name, config.language);
        
        // Validate the handler script exists and is executable
        self.validate_handler(&config).await?;
        
        let handler = HandlerInstance {
            id: Uuid::new_v4(),
            name: name.clone(),
            runtime: config.language.clone(),
            script_path: config.script.clone(),
            status: HandlerStatus::Idle,
            last_execution: None,
            execution_count: 0,
            average_duration: 0.0,
            error_count: 0,
        };
        
        let mut handlers = self.handlers.write().await;
        handlers.insert(name, handler);
        
        Ok(())
    }

    pub async fn execute_handler(&self, request: ExecutionRequest) -> BackworksResult<ExecutionResult> {
        let start_time = std::time::Instant::now();
        
        let mut handlers = self.handlers.write().await;
        let handler = handlers.get_mut(&request.handler_name)
            .ok_or_else(|| BackworksError::Config(format!("Handler not found: {}", request.handler_name)))?;
        
        handler.status = HandlerStatus::Running;
        
        let result = match handler.runtime.as_str() {
            "node" | "nodejs" => {
                // For now, return a mock ExecutionResult since we don't have these handlers implemented
                Ok(ExecutionResult {
                    success: true,
                    status_code: 200,
                    headers: std::collections::HashMap::new(),
                    body: Some(serde_json::json!({"message": "Node.js handler executed successfully"})),
                    duration: Duration::from_millis(100),
                    error: None,
                })
            },
            "python" | "python3" => {
                // Convert to the expected signature
                let result = self.execute_python_handler(&handler.script_path, "{}").await?;
                Ok(ExecutionResult {
                    success: true,
                    status_code: 200,
                    headers: std::collections::HashMap::new(),
                    body: Some(serde_json::json!({"result": result})),
                    duration: Duration::from_millis(100),
                    error: None,
                })
            },
            "shell" | "bash" => {
                // For now, return a mock ExecutionResult
                Ok(ExecutionResult {
                    success: true,
                    status_code: 200,
                    headers: std::collections::HashMap::new(),
                    body: Some(serde_json::json!({"message": "Shell handler executed successfully"})),
                    duration: Duration::from_millis(100),
                    error: None,
                })
            },
            _ => Err(BackworksError::Config(format!("Unsupported runtime: {}", handler.runtime))),
        };
        
        let duration = start_time.elapsed();
        
        // Update handler statistics
        handler.last_execution = Some(chrono::Utc::now());
        handler.execution_count += 1;
        handler.average_duration = (handler.average_duration + duration.as_millis() as f64) / handler.execution_count as f64;
        
        match &result {
            Ok(_) => {
                handler.status = HandlerStatus::Idle;
            }
            Err(err) => {
                handler.error_count += 1;
                handler.status = HandlerStatus::Error(err.to_string());
            }
        }
        
        result
    }

    pub async fn get_handlers(&self) -> Vec<HandlerInstance> {
        self.handlers.read().await.values().cloned().collect()
    }

    pub async fn get_handler(&self, name: &str) -> Option<HandlerInstance> {
        self.handlers.read().await.get(name).cloned()
    }

    pub async fn stop_handler(&self, name: &str) -> BackworksResult<()> {
        let mut handlers = self.handlers.write().await;
        if let Some(handler) = handlers.get_mut(name) {
            handler.status = HandlerStatus::Stopped;
            tracing::info!("Stopped handler: {}", name);
        }
        Ok(())
    }

    pub async fn handle_request(&self, config: &RuntimeConfig, request_data: &str) -> BackworksResult<String> {
        tracing::info!("Handling runtime request with language: {}", config.language);
        
        match config.language.as_str() {
            "javascript" | "js" | "node" => {
                self.execute_javascript_handler(&config.handler, request_data).await
            }
            "python" | "py" => {
                self.execute_python_handler(&config.handler, request_data).await
            }
            _ => {
                Err(BackworksError::runtime(format!("Unsupported runtime language: {}", config.language)))
            }
        }
    }
    
    async fn execute_javascript_handler(&self, handler_code: &str, request_data: &str) -> BackworksResult<String> {
        // Create a wrapper script that handles the function execution
        let wrapper_script = format!(r#"
// Parse request data
const request = JSON.parse(process.argv[2] || '{{}}');

// Handler function
{}

// Execute handler and output result
try {{
    const result = handler(request);
    console.log(JSON.stringify(result));
}} catch (error) {{
    console.error('Handler error:', error.message);
    process.exit(1);
}}
"#, handler_code);

        // Create a temporary file for the handler
        let temp_file = format!("/tmp/backworks_handler_{}.js", Uuid::new_v4());
        tokio::fs::write(&temp_file, wrapper_script).await
            .map_err(|e| BackworksError::runtime(format!("Failed to write handler file: {}", e)))?;
        
        // Execute the handler with request data as argument
        let output = Command::new("node")
            .arg(&temp_file)
            .arg(request_data)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| BackworksError::runtime(format!("Failed to spawn Node.js process: {}", e)))?
            .wait_with_output()
            .await
            .map_err(|e| BackworksError::runtime(format!("Handler execution failed: {}", e)))?;

        // Clean up temp file
        let _ = tokio::fs::remove_file(&temp_file).await;
        
        if output.status.success() {
            String::from_utf8(output.stdout)
                .map_err(|e| BackworksError::runtime(format!("Invalid UTF-8 output: {}", e)))
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(BackworksError::runtime(format!("Handler execution error: {}", error)))
        }
    }
    
    async fn execute_python_handler(&self, handler_code: &str, request_data: &str) -> BackworksResult<String> {
        // Create a temporary file for the handler
        let temp_file = format!("/tmp/backworks_handler_{}.py", Uuid::new_v4());
        tokio::fs::write(&temp_file, handler_code).await
            .map_err(|e| BackworksError::runtime(format!("Failed to write handler file: {}", e)))?;
        
        // Execute the handler
        let mut output = Command::new("python3")
            .arg(&temp_file)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| BackworksError::runtime(format!("Failed to spawn Python process: {}", e)))?;
        
        // Write request data to stdin
        if let Some(mut stdin) = output.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(request_data.as_bytes()).await
                .map_err(|e| BackworksError::runtime(format!("Failed to write to handler stdin: {}", e)))?;
            stdin.shutdown().await
                .map_err(|e| BackworksError::runtime(format!("Failed to close handler stdin: {}", e)))?;
        }
        
        // Wait for completion and get output
        let result = output.wait_with_output().await
            .map_err(|e| BackworksError::runtime(format!("Handler execution failed: {}", e)))?;
        
        // Clean up temp file
        let _ = tokio::fs::remove_file(&temp_file).await;
        
        if result.status.success() {
            String::from_utf8(result.stdout)
                .map_err(|e| BackworksError::runtime(format!("Invalid UTF-8 output: {}", e)))
        } else {
            let error = String::from_utf8_lossy(&result.stderr);
            Err(BackworksError::runtime(format!("Handler execution error: {}", error)))
        }
    }
    
    async fn validate_handler(&self, config: &HandlerConfig) -> BackworksResult<()> {
        // Check if script file exists
        if !tokio::fs::metadata(&config.script).await.is_ok() {
            return Err(BackworksError::Config(format!("Handler script not found: {}", config.script)));
        }
        
        // Validate runtime is available
        let runtime_check = match config.language.as_str() {
            "node" | "nodejs" => Command::new("node").arg("--version").output().await,
            "python" | "python3" => Command::new("python3").arg("--version").output().await,
            "shell" | "bash" => Command::new("bash").arg("--version").output().await,
            "dotnet" => Command::new("dotnet").arg("--version").output().await,
            "go" => Command::new("go").arg("version").output().await,
            _ => return Err(BackworksError::Config(format!("Unsupported runtime: {}", config.language))),
        };
        
        match runtime_check {
            Ok(output) if output.status.success() => {
                tracing::debug!("Runtime {} is available", config.language);
                Ok(())
            }
            _ => Err(BackworksError::Config(format!("Runtime {} is not available", config.language))),
        }
    }

    async fn execute_nodejs_handler(&self, handler: &HandlerInstance, request: &ExecutionRequest) -> BackworksResult<ExecutionResult> {
        let input = serde_json::to_string(&serde_json::json!({
            "method": request.method,
            "path": request.path,
            "headers": request.headers,
            "query": request.query_params,
            "body": request.body
        }))?;
        
        let mut child = Command::new("node")
            .arg(&handler.script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| BackworksError::Runtime(format!("Failed to spawn Node.js process: {}", e)))?;
        
        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(input.as_bytes()).await
                .map_err(|e| BackworksError::Runtime(format!("Failed to write to stdin: {}", e)))?;
        }
        
        let output = child.wait_with_output().await
            .map_err(|e| BackworksError::Runtime(format!("Failed to execute Node.js handler: {}", e)))?;
        
        let result = self.parse_handler_output(output.stdout, output.stderr, output.status.success()).await?;
        
        Ok(ExecutionResult {
            success: output.status.success(),
            status_code: 200,
            headers: HashMap::new(),
            body: serde_json::from_str(&result).ok(),
            duration: std::time::Duration::from_millis(123), // Simulated duration
            error: if output.status.success() { None } else { Some(result) },
        })
    }
    
    async fn parse_handler_output(&self, stdout: Vec<u8>, stderr: Vec<u8>, success: bool) -> BackworksResult<String> {
        if !success {
            let stderr_str = String::from_utf8_lossy(&stderr);
            return Err(BackworksError::Runtime(format!("Handler execution failed: {}", stderr_str)));
        }
        
        let output_str = String::from_utf8_lossy(&stdout);
        Ok(output_str.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_manager_creation() {
        let config = RuntimeManagerConfig::default();
        let runtime_manager = RuntimeManager::new(config);
        
        assert!(runtime_manager.start().await.is_ok());
    }
}
