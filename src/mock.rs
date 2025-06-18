use std::sync::Arc;
use serde_json::Value;
use handlebars::Handlebars;
use chrono::Utc;
use uuid::Uuid;
use rand::{thread_rng, Rng};

use crate::config::{BackworksConfig, EndpointConfig, MockConfig, MockResponse};
use crate::server::RequestData;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct MockHandler {
    config: Arc<BackworksConfig>,
    handlebars: Handlebars<'static>,
}

impl MockHandler {
    pub fn new(config: Arc<BackworksConfig>) -> Self {
        let mut handlebars = Handlebars::new();
        
        // Register helper functions for dynamic data generation
        Self::register_helpers(&mut handlebars);
        
        Self {
            config,
            handlebars,
        }
    }
    
    pub async fn handle_request(
        &self,
        endpoint_name: &str,
        endpoint_config: &EndpointConfig,
        request_data: &RequestData,
    ) -> Result<Value> {
        // Check for method-specific mock responses first
        if let Some(ref mock_responses) = endpoint_config.mock_responses {
            let method_key = format!("{} {}", request_data.method, endpoint_config.path);
            
            if let Some(response) = mock_responses.get(&request_data.method)
                .or_else(|| mock_responses.get(&method_key)) {
                return self.process_mock_response(response, request_data).await;
            }
        }
        
        // Fall back to general mock configuration
        if let Some(ref mock_config) = endpoint_config.mock {
            return self.process_mock_config(mock_config, request_data).await;
        }
        
        // Default response if no mock configuration
        Ok(serde_json::json!({
            "message": format!("Mock response for {}", endpoint_name),
            "method": request_data.method,
            "timestamp": Utc::now(),
            "path": endpoint_config.path
        }))
    }
    
    async fn process_mock_response(
        &self,
        mock_response: &MockResponse,
        request_data: &RequestData,
    ) -> Result<Value> {
        let context = self.create_template_context(request_data);
        let body_str = serde_json::to_string(&mock_response.body)?;
        let rendered = self.handlebars.render_template(&body_str, &context)?;
        let result: Value = serde_json::from_str(&rendered)?;
        Ok(result)
    }
    
    async fn process_mock_config(
        &self,
        mock_config: &MockConfig,
        request_data: &RequestData,
    ) -> Result<Value> {
        if let Some(ref data) = mock_config.data {
            let context = self.create_template_context(request_data);
            let data_str = serde_json::to_string(data)?;
            let rendered = self.handlebars.render_template(&data_str, &context)?;
            let result: Value = serde_json::from_str(&rendered)?;
            Ok(result)
        } else if mock_config.ai_generated.unwrap_or(false) {
            // TODO: Integrate with AI for generating mock data
            Ok(self.generate_ai_mock_data(mock_config, request_data).await?)
        } else {
            // Generate basic mock data
            Ok(self.generate_basic_mock_data(request_data))
        }
    }
    
    fn create_template_context(&self, request_data: &RequestData) -> Value {
        let now = Utc::now();
        
        serde_json::json!({
            "request": {
                "method": request_data.method,
                "body": request_data.body,
                "headers": self.headers_to_json(&request_data.headers),
            },
            "path": request_data.path_params,
            "query": request_data.query_params,
            "now": now.to_rfc3339(),
            "timestamp": now.timestamp(),
            "random_id": Uuid::new_v4().to_string(),
            "random_int": thread_rng().gen_range(1..1000),
            "random_float": thread_rng().gen_range(0.0..100.0),
        })
    }
    
    fn headers_to_json(&self, headers: &axum::http::HeaderMap) -> Value {
        let mut map = serde_json::Map::new();
        for (key, value) in headers {
            if let Ok(value_str) = value.to_str() {
                map.insert(key.to_string(), Value::String(value_str.to_string()));
            }
        }
        Value::Object(map)
    }
    
    async fn generate_ai_mock_data(
        &self,
        _mock_config: &MockConfig,
        _request_data: &RequestData,
    ) -> Result<Value> {
        // Simulate AI-powered mock data generation
        Ok(serde_json::json!({
            "ai_generated": true,
            "data": "This is simulated AI-generated mock data."
        }))
    }
    
    fn generate_basic_mock_data(&self, request_data: &RequestData) -> Value {
        match request_data.method.as_str() {
            "GET" => serde_json::json!({
                "data": "Mock GET response",
                "timestamp": Utc::now()
            }),
            "POST" => serde_json::json!({
                "id": Uuid::new_v4(),
                "created": true,
                "timestamp": Utc::now(),
                "data": request_data.body
            }),
            "PUT" => serde_json::json!({
                "updated": true,
                "timestamp": Utc::now(),
                "data": request_data.body
            }),
            "DELETE" => serde_json::json!({
                "deleted": true,
                "timestamp": Utc::now()
            }),
            _ => serde_json::json!({
                "message": format!("Mock {} response", request_data.method),
                "timestamp": Utc::now()
            })
        }
    }
    
    fn register_helpers(handlebars: &mut Handlebars) {
        // Register helper for current timestamp
        handlebars.register_helper("now", Box::new(|_: &handlebars::Helper, _: &Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| {
            let now = Utc::now().to_rfc3339();
            out.write(&now)?;
            Ok(())
        }));
        
        // Register helper for random integers
        handlebars.register_helper("random_int", Box::new(|h: &handlebars::Helper, _: &Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| {
            let min = h.param(0).and_then(|v| v.value().as_i64()).unwrap_or(1);
            let max = h.param(1).and_then(|v| v.value().as_i64()).unwrap_or(1000);
            let random_num = thread_rng().gen_range(min..=max);
            out.write(&random_num.to_string())?;
            Ok(())
        }));
        
        // Register helper for random UUIDs
        handlebars.register_helper("random_uuid", Box::new(|_: &handlebars::Helper, _: &Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| {
            let uuid = Uuid::new_v4().to_string();
            out.write(&uuid)?;
            Ok(())
        }));
        
        // Register helper for random floats
        handlebars.register_helper("random_float", Box::new(|h: &handlebars::Helper, _: &Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| {
            let min = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
            let max = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(100.0);
            let random_num = thread_rng().gen_range(min..max);
            out.write(&format!("{:.2}", random_num))?;
            Ok(())
        }));
        
        // Register helper for random names
        handlebars.register_helper("random_name", Box::new(|_: &handlebars::Helper, _: &Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| {
            let first_names = ["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry"];
            let last_names = ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis"];
            
            let first = first_names[thread_rng().gen_range(0..first_names.len())];
            let last = last_names[thread_rng().gen_range(0..last_names.len())];
            
            out.write(&format!("{} {}", first, last))?;
            Ok(())
        }));
        
        // Register helper for random emails
        handlebars.register_helper("random_email", Box::new(|_: &handlebars::Helper, _: &Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| {
            let domains = ["example.com", "test.org", "demo.net", "sample.io"];
            let domain = domains[thread_rng().gen_range(0..domains.len())];
            let username = format!("user{}", thread_rng().gen_range(1..1000));
            
            out.write(&format!("{}@{}", username, domain))?;
            Ok(())
        }));
        
        // Register helper for date arithmetic
        handlebars.register_helper("date_add", Box::new(|h: &handlebars::Helper, _: &Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| {
            let base = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
            let days = h.param(1).and_then(|v| v.value().as_i64()).unwrap_or(0);
            
            if base == "now()" {
                let future = Utc::now() + chrono::Duration::days(days);
                out.write(&future.to_rfc3339())?;
            } else {
                out.write(base)?;
            }
            Ok(())
        }));
        
        handlebars.register_helper("date_subtract", Box::new(|h: &handlebars::Helper, _: &Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| {
            let base = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
            let days = h.param(1).and_then(|v| v.value().as_i64()).unwrap_or(0);
            
            if base == "now()" {
                let past = Utc::now() - chrono::Duration::days(days);
                out.write(&past.to_rfc3339())?;
            } else {
                out.write(base)?;
            }
            Ok(())
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ServerConfig, ExecutionMode};
    use std::collections::HashMap;
    
    fn create_test_config() -> BackworksConfig {
        BackworksConfig {
            name: "test_api".to_string(),
            description: None,
            version: None,
            mode: ExecutionMode::Mock,
            endpoints: HashMap::new(),
            server: ServerConfig::default(),
            ai: Default::default(),
            dashboard: None,
            database: None,
            apis: None,
            cache: None,
            security: None,
            monitoring: None,
            global_headers: HashMap::new(),
            logging: Default::default(),
        }
    }
    
    #[tokio::test]
    async fn test_mock_handler_creation() {
        let config = Arc::new(create_test_config());
        let handler = MockHandler::new(config);
        
        // Test that the handler was created successfully
        assert!(!handler.handlebars.get_templates().is_empty());
    }
    
    #[tokio::test]
    async fn test_basic_mock_data_generation() {
        let config = Arc::new(create_test_config());
        let handler = MockHandler::new(config);
        
        let request_data = RequestData {
            method: "GET".to_string(),
            path_params: HashMap::new(),
            query_params: HashMap::new(),
            headers: axum::http::HeaderMap::new(),
            body: None,
        };
        
        let result = handler.generate_basic_mock_data(&request_data);
        assert!(result.is_object());
        assert!(result.get("data").is_some());
        assert!(result.get("timestamp").is_some());
    }
}
