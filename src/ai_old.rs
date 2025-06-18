use crate::config::AIConfig;
use crate::error::BackworksResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPattern {
    pub id: Uuid,
    pub method: String,
    pub path_pattern: String,
    pub frequency: u32,
    pub avg_response_time: f64,
    pub error_rate: f64,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub id: Uuid,
    pub anomaly_type: String,
    pub description: String,
    pub severity: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaPrediction {
    pub field_name: String,
    pub field_type: String,
    pub confidence: f64,
    pub examples: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIInsight {
    pub id: Uuid,
    pub insight_type: String,
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct AIEnhancer {
    config: AIConfig,
    patterns: RwLock<Vec<RequestPattern>>,
    anomalies: RwLock<Vec<Anomaly>>,
    insights: RwLock<Vec<AIInsight>>,
}

impl AIEnhancer {
    pub fn new(config: AIConfig) -> Self {
        Self {
            config,
            patterns: RwLock::new(Vec::new()),
            anomalies: RwLock::new(Vec::new()),
            insights: RwLock::new(Vec::new()),
        }
    }

    pub async fn start(&self) -> BackworksResult<()> {
        tracing::info!("Starting AI enhancer with features: {:?}", self.config.features);
        
        // Initialize AI models and services based on configuration
        if self.config.features.pattern_recognition {
            self.initialize_pattern_recognition().await?;
        }
        
        if self.config.features.schema_prediction {
            self.initialize_schema_prediction().await?;
        }
        
        if self.config.features.anomaly_detection {
            self.initialize_anomaly_detection().await?;
        }
        
        Ok(())
    }

    pub async fn analyze_request(
        &self,
        method: &str,
        path: &str,
        body: Option<&serde_json::Value>,
        response_time: f64,
        status_code: u16,
    ) -> BackworksResult<()> {
        if self.config.features.pattern_recognition {
            self.update_patterns(method, path, response_time, status_code).await?;
        }
        
        if self.config.features.anomaly_detection {
            self.detect_anomalies(method, path, response_time, status_code).await?;
        }
        
        if let Some(body) = body {
            if self.config.features.schema_prediction {
                self.analyze_schema(body).await?;
            }
        }
        
        Ok(())
    }

    pub async fn predict_schema(&self, sample_data: &serde_json::Value) -> BackworksResult<Vec<SchemaPrediction>> {
        if !self.config.features.schema_prediction {
            return Ok(Vec::new());
        }
        
        let mut predictions = Vec::new();
        
        if let Some(obj) = sample_data.as_object() {
            for (key, value) in obj {
                let prediction = self.predict_field_type(key, value).await?;
                predictions.push(prediction);
            }
        }
        
        Ok(predictions)
    }

    pub async fn get_patterns(&self) -> Vec<RequestPattern> {
        self.patterns.read().await.clone()
    }

    pub async fn get_anomalies(&self) -> Vec<Anomaly> {
        self.anomalies.read().await.clone()
    }

    pub async fn get_insights(&self) -> Vec<AIInsight> {
        self.insights.read().await.clone()
    }

    pub async fn generate_api_suggestion(&self, existing_endpoints: &[String]) -> BackworksResult<AIInsight> {
        let patterns = self.get_patterns().await;
        
        // Analyze patterns to suggest new endpoints
        let mut suggested_endpoint = String::new();
        let mut confidence = 0.0;
        
        if !patterns.is_empty() {
            // Simple heuristic: suggest CRUD operations for frequently accessed resources
            let most_frequent = patterns.iter()
                .max_by_key(|p| p.frequency)
                .unwrap();
            
            if most_frequent.method == "GET" && !existing_endpoints.iter().any(|e| e.contains("POST")) {
                suggested_endpoint = format!("POST {}", most_frequent.path_pattern);
                confidence = 0.8;
            }
        }
        
        Ok(AIInsight {
            id: Uuid::new_v4(),
            insight_type: "api_suggestion".to_string(),
            title: "Suggested API Endpoint".to_string(),
            description: format!("Consider adding: {}", suggested_endpoint),
            confidence,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        })
    }

    async fn initialize_pattern_recognition(&self) -> BackworksResult<()> {
        tracing::info!("Initializing pattern recognition");
        // In a real implementation, this would load pre-trained models
        // For now, we'll use simple statistical analysis
        Ok(())
    }

    async fn initialize_schema_prediction(&self) -> BackworksResult<()> {
        tracing::info!("Initializing schema prediction");
        // In a real implementation, this would load ML models for schema inference
        Ok(())
    }

    async fn initialize_anomaly_detection(&self) -> BackworksResult<()> {
        tracing::info!("Initializing anomaly detection");
        // In a real implementation, this would set up anomaly detection models
        Ok(())
    }

    async fn update_patterns(
        &self,
        method: &str,
        path: &str,
        response_time: f64,
        status_code: u16,
    ) -> BackworksResult<()> {
        let mut patterns = self.patterns.write().await;
        
        // Find existing pattern or create new one
        let path_pattern = self.extract_path_pattern(path);
        
        if let Some(pattern) = patterns.iter_mut().find(|p| p.method == method && p.path_pattern == path_pattern) {
            pattern.frequency += 1;
            pattern.avg_response_time = (pattern.avg_response_time + response_time) / 2.0;
            pattern.error_rate = if status_code >= 400 {
                (pattern.error_rate + 1.0) / 2.0
            } else {
                pattern.error_rate * 0.95
            };
            pattern.last_seen = chrono::Utc::now();
        } else {
            patterns.push(RequestPattern {
                id: Uuid::new_v4(),
                method: method.to_string(),
                path_pattern,
                frequency: 1,
                avg_response_time: response_time,
                error_rate: if status_code >= 400 { 1.0 } else { 0.0 },
                last_seen: chrono::Utc::now(),
            });
        }
        
        Ok(())
    }

    async fn detect_anomalies(
        &self,
        method: &str,
        path: &str,
        response_time: f64,
        status_code: u16,
    ) -> BackworksResult<()> {
        let patterns = self.patterns.read().await;
        let path_pattern = self.extract_path_pattern(path);
        
        if let Some(pattern) = patterns.iter().find(|p| p.method == method && p.path_pattern == path_pattern) {
            // Check for response time anomalies
            if response_time > pattern.avg_response_time * 3.0 {
                let anomaly = Anomaly {
                    id: Uuid::new_v4(),
                    anomaly_type: "slow_response".to_string(),
                    description: format!(
                        "Response time {}ms is significantly higher than average {}ms",
                        response_time as u64,
                        pattern.avg_response_time as u64
                    ),
                    severity: "medium".to_string(),
                    timestamp: chrono::Utc::now(),
                    context: {
                        let mut ctx = HashMap::new();
                        ctx.insert("method".to_string(), serde_json::Value::String(method.to_string()));
                        ctx.insert("path".to_string(), serde_json::Value::String(path.to_string()));
                        ctx.insert("response_time".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(response_time).unwrap()));
                        ctx.insert("avg_response_time".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(pattern.avg_response_time).unwrap()));
                        ctx
                    },
                };
                
                self.anomalies.write().await.push(anomaly);
            }
        }
        
        Ok(())
    }

    async fn analyze_schema(&self, body: &serde_json::Value) -> BackworksResult<()> {
        // This would analyze the request/response body structure
        // and learn about the API schema over time
        tracing::debug!("Analyzing schema for body: {}", body);
        Ok(())
    }

    async fn predict_field_type(&self, field_name: &str, value: &serde_json::Value) -> BackworksResult<SchemaPrediction> {
        let (field_type, confidence) = match value {
            serde_json::Value::String(s) => {
                if s.contains('@') {
                    ("email".to_string(), 0.9)
                } else if s.len() == 36 && s.contains('-') {
                    ("uuid".to_string(), 0.8)
                } else if chrono::DateTime::parse_from_rfc3339(s).is_ok() {
                    ("datetime".to_string(), 0.9)
                } else {
                    ("string".to_string(), 1.0)
                }
            },
            serde_json::Value::Number(_) => {
                if field_name.contains("id") {
                    ("id".to_string(), 0.8)
                } else {
                    ("number".to_string(), 1.0)
                }
            },
            serde_json::Value::Bool(_) => ("boolean".to_string(), 1.0),
            serde_json::Value::Array(_) => ("array".to_string(), 1.0),
            serde_json::Value::Object(_) => ("object".to_string(), 1.0),
            serde_json::Value::Null => ("nullable".to_string(), 0.5),
        };
        
        Ok(SchemaPrediction {
            field_name: field_name.to_string(),
            field_type,
            confidence,
            examples: vec![value.clone()],
        })
    }

    fn extract_path_pattern(&self, path: &str) -> String {
        // Simple pattern extraction: replace numeric segments with {id}
        let segments: Vec<&str> = path.split('/').collect();
        let pattern_segments: Vec<String> = segments
            .iter()
            .map(|segment| {
                if segment.parse::<i64>().is_ok() || segment.parse::<uuid::Uuid>().is_ok() {
                    "{id}".to_string()
                } else {
                    segment.to_string()
                }
            })
            .collect();
        
        pattern_segments.join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AIFeatures;

    fn create_test_ai_config() -> AIConfig {
        AIConfig {
            enabled: true,
            features: AIFeatures {
                pattern_recognition: true,
                schema_prediction: true,
                anomaly_detection: true,
                intelligent_mocking: false,
            },
            models: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_ai_enhancer_creation() {
        let config = create_test_ai_config();
        let enhancer = AIEnhancer::new(config);
        
        assert!(enhancer.start().await.is_ok());
    }

    #[tokio::test]
    async fn test_pattern_recognition() {
        let config = create_test_ai_config();
        let enhancer = AIEnhancer::new(config);
        
        enhancer.analyze_request("GET", "/users/123", None, 100.0, 200).await.unwrap();
        enhancer.analyze_request("GET", "/users/456", None, 110.0, 200).await.unwrap();
        
        let patterns = enhancer.get_patterns().await;
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].path_pattern, "/users/{id}");
        assert_eq!(patterns[0].frequency, 2);
    }

    #[tokio::test]
    async fn test_schema_prediction() {
        let config = create_test_ai_config();
        let enhancer = AIEnhancer::new(config);
        
        let sample_data = serde_json::json!({
            "id": 123,
            "email": "test@example.com",
            "created_at": "2023-01-01T00:00:00Z",
            "active": true
        });
        
        let predictions = enhancer.predict_schema(&sample_data).await.unwrap();
        assert_eq!(predictions.len(), 4);
        
        let email_prediction = predictions.iter().find(|p| p.field_name == "email").unwrap();
        assert_eq!(email_prediction.field_type, "email");
        assert!(email_prediction.confidence > 0.8);
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let config = create_test_ai_config();
        let enhancer = AIEnhancer::new(config);
        
        // Establish a pattern
        enhancer.analyze_request("GET", "/users/123", None, 100.0, 200).await.unwrap();
        enhancer.analyze_request("GET", "/users/456", None, 100.0, 200).await.unwrap();
        
        // Introduce an anomaly
        enhancer.analyze_request("GET", "/users/789", None, 1000.0, 200).await.unwrap();
        
        let anomalies = enhancer.get_anomalies().await;
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].anomaly_type, "slow_response");
    }
}
