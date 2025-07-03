//! Request and response transformation utilities

use crate::error::{ProxyError, ProxyResult};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Uri};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Request transformation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTransformConfig {
    /// Headers to add to the request
    pub add_headers: Option<HashMap<String, String>>,
    
    /// Headers to remove from the request
    pub remove_headers: Option<Vec<String>>,
    
    /// Header name mappings (old_name -> new_name)
    pub header_mapping: Option<HashMap<String, String>>,
    
    /// Path prefix to strip from the request
    pub strip_prefix: Option<String>,
    
    /// Path prefix to add to the request
    pub add_prefix: Option<String>,
    
    /// Path rewrite rules (pattern -> replacement)
    pub path_rewrites: Option<HashMap<String, String>>,
    
    /// Query parameters to add
    pub add_query_params: Option<HashMap<String, String>>,
    
    /// Query parameters to remove
    pub remove_query_params: Option<Vec<String>>,
    
    /// Body transformation rules
    pub body_transform: Option<BodyTransformConfig>,
}

/// Response transformation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTransformConfig {
    /// Headers to add to the response
    pub add_headers: Option<HashMap<String, String>>,
    
    /// Headers to remove from the response
    pub remove_headers: Option<Vec<String>>,
    
    /// Header name mappings (old_name -> new_name)
    pub header_mapping: Option<HashMap<String, String>>,
    
    /// Status code mappings (old_code -> new_code)
    pub status_code_mapping: Option<HashMap<u16, u16>>,
    
    /// Body transformation rules
    pub body_transform: Option<BodyTransformConfig>,
}

/// Body transformation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyTransformConfig {
    /// JSON field mappings (old_field -> new_field)
    pub json_field_mapping: Option<HashMap<String, String>>,
    
    /// JSON fields to remove
    pub json_remove_fields: Option<Vec<String>>,
    
    /// JSON fields to add (field -> value)
    pub json_add_fields: Option<HashMap<String, Value>>,
    
    /// Text replacements (pattern -> replacement)
    pub text_replacements: Option<HashMap<String, String>>,
    
    /// Template for body transformation (using handlebars syntax)
    pub template: Option<String>,
}

/// Request transformer
#[derive(Debug)]
pub struct RequestTransformer {
    config: RequestTransformConfig,
}

impl RequestTransformer {
    pub fn new(config: RequestTransformConfig) -> Self {
        Self { config }
    }

    /// Transform request headers
    pub fn transform_headers(&self, headers: &mut HeaderMap) -> ProxyResult<()> {
        // Remove headers
        if let Some(ref remove_headers) = self.config.remove_headers {
            for header_name in remove_headers {
                if let Ok(name) = HeaderName::try_from(header_name) {
                    headers.remove(&name);
                }
            }
        }

        // Add headers
        if let Some(ref add_headers) = self.config.add_headers {
            for (name, value) in add_headers {
                if let (Ok(header_name), Ok(header_value)) = (
                    HeaderName::try_from(name),
                    HeaderValue::try_from(value)
                ) {
                    headers.insert(header_name, header_value);
                }
            }
        }

        // Map headers (rename)
        if let Some(ref header_mapping) = self.config.header_mapping {
            let mut headers_to_rename = Vec::new();
            
            for (old_name, new_name) in header_mapping {
                if let Ok(old_header_name) = HeaderName::try_from(old_name) {
                    if let Some(value) = headers.remove(&old_header_name) {
                        headers_to_rename.push((new_name.clone(), value));
                    }
                }
            }
            
            for (new_name, value) in headers_to_rename {
                if let Ok(header_name) = HeaderName::try_from(&new_name) {
                    headers.insert(header_name, value);
                }
            }
        }

        Ok(())
    }

    /// Transform request URI
    pub fn transform_uri(&self, uri: &Uri) -> ProxyResult<Uri> {
        let mut path = uri.path().to_string();
        let mut query = uri.query().map(|q| q.to_string());

        // Strip prefix
        if let Some(ref prefix) = self.config.strip_prefix {
            if path.starts_with(prefix) {
                path = path[prefix.len()..].to_string();
                if !path.starts_with('/') {
                    path = format!("/{}", path);
                }
            }
        }

        // Add prefix
        if let Some(ref prefix) = self.config.add_prefix {
            path = format!("{}{}", prefix, path);
        }

        // Apply path rewrites
        if let Some(ref rewrites) = self.config.path_rewrites {
            for (pattern, replacement) in rewrites {
                if path.contains(pattern) {
                    path = path.replace(pattern, replacement);
                }
            }
        }

        // Handle query parameters
        let mut query_params: HashMap<String, String> = HashMap::new();
        
        if let Some(q) = &query {
            query_params = serde_urlencoded::from_str(q).unwrap_or_default();
        }

        // Remove query parameters
        if let Some(ref remove_params) = self.config.remove_query_params {
            for param in remove_params {
                query_params.remove(param);
            }
        }

        // Add query parameters
        if let Some(ref add_params) = self.config.add_query_params {
            for (key, value) in add_params {
                query_params.insert(key.clone(), value.clone());
            }
        }

        // Rebuild query string
        if !query_params.is_empty() {
            query = Some(serde_urlencoded::to_string(&query_params)
                .map_err(|e| ProxyError::Transformation(e.to_string()))?);
        } else {
            query = None;
        }

        // Build new URI
        let uri_str = match query {
            Some(q) => format!("{}?{}", path, q),
            None => path,
        };

        uri_str.parse().map_err(|e| ProxyError::Transformation(format!("URI parse error: {}", e)))
    }

    /// Transform request body
    pub fn transform_body(&self, body: &[u8], content_type: Option<&str>) -> ProxyResult<Vec<u8>> {
        if let Some(ref body_config) = self.config.body_transform {
            self.transform_body_with_config(body, content_type, body_config)
        } else {
            Ok(body.to_vec())
        }
    }

    fn transform_body_with_config(
        &self,
        body: &[u8],
        content_type: Option<&str>,
        config: &BodyTransformConfig,
    ) -> ProxyResult<Vec<u8>> {
        // Try JSON transformation first if content type suggests JSON
        if let Some(ct) = content_type {
            if ct.contains("application/json") {
                if let Ok(json_str) = String::from_utf8(body.to_vec()) {
                    if let Ok(mut json_value) = serde_json::from_str::<Value>(&json_str) {
                        self.transform_json_value(&mut json_value, config)?;
                        let transformed = serde_json::to_vec(&json_value)
                            .map_err(|e| ProxyError::Transformation(e.to_string()))?;
                        return Ok(transformed);
                    }
                }
            }
        }

        // Fallback to text transformation
        if let Ok(text) = String::from_utf8(body.to_vec()) {
            let transformed = self.transform_text(&text, config)?;
            Ok(transformed.into_bytes())
        } else {
            // Binary data - return as-is
            Ok(body.to_vec())
        }
    }

    fn transform_json_value(&self, value: &mut Value, config: &BodyTransformConfig) -> ProxyResult<()> {
        if let Value::Object(ref mut map) = value {
            // Remove fields
            if let Some(ref remove_fields) = config.json_remove_fields {
                for field in remove_fields {
                    map.remove(field);
                }
            }

            // Add fields
            if let Some(ref add_fields) = config.json_add_fields {
                for (field, field_value) in add_fields {
                    map.insert(field.clone(), field_value.clone());
                }
            }

            // Map fields (rename)
            if let Some(ref field_mapping) = config.json_field_mapping {
                let mut fields_to_rename = Vec::new();
                
                for (old_field, new_field) in field_mapping {
                    if let Some(field_value) = map.remove(old_field) {
                        fields_to_rename.push((new_field.clone(), field_value));
                    }
                }
                
                for (new_field, field_value) in fields_to_rename {
                    map.insert(new_field, field_value);
                }
            }
        }

        Ok(())
    }

    fn transform_text(&self, text: &str, config: &BodyTransformConfig) -> ProxyResult<String> {
        let mut result = text.to_string();

        // Apply text replacements
        if let Some(ref replacements) = config.text_replacements {
            for (pattern, replacement) in replacements {
                result = result.replace(pattern, replacement);
            }
        }

        // TODO: Implement template transformation using handlebars
        // if let Some(ref template) = config.template {
        //     // Use handlebars to render template with context
        // }

        Ok(result)
    }
}

/// Response transformer
#[derive(Debug)]
pub struct ResponseTransformer {
    config: ResponseTransformConfig,
}

impl ResponseTransformer {
    pub fn new(config: ResponseTransformConfig) -> Self {
        Self { config }
    }

    /// Transform response headers
    pub fn transform_headers(&self, headers: &mut HeaderMap) -> ProxyResult<()> {
        // Remove headers
        if let Some(ref remove_headers) = self.config.remove_headers {
            for header_name in remove_headers {
                if let Ok(name) = HeaderName::try_from(header_name) {
                    headers.remove(&name);
                }
            }
        }

        // Add headers
        if let Some(ref add_headers) = self.config.add_headers {
            for (name, value) in add_headers {
                if let (Ok(header_name), Ok(header_value)) = (
                    HeaderName::try_from(name),
                    HeaderValue::try_from(value)
                ) {
                    headers.insert(header_name, header_value);
                }
            }
        }

        // Map headers (rename)
        if let Some(ref header_mapping) = self.config.header_mapping {
            let mut headers_to_rename = Vec::new();
            
            for (old_name, new_name) in header_mapping {
                if let Ok(old_header_name) = HeaderName::try_from(old_name) {
                    if let Some(value) = headers.remove(&old_header_name) {
                        headers_to_rename.push((new_name.clone(), value));
                    }
                }
            }
            
            for (new_name, value) in headers_to_rename {
                if let Ok(header_name) = HeaderName::try_from(&new_name) {
                    headers.insert(header_name, value);
                }
            }
        }

        Ok(())
    }

    /// Transform response status code
    pub fn transform_status_code(&self, status_code: u16) -> u16 {
        if let Some(ref status_mapping) = self.config.status_code_mapping {
            status_mapping.get(&status_code).copied().unwrap_or(status_code)
        } else {
            status_code
        }
    }

    /// Transform response body
    pub fn transform_body(&self, body: &[u8], content_type: Option<&str>) -> ProxyResult<Vec<u8>> {
        if let Some(ref body_config) = self.config.body_transform {
            let request_transformer = RequestTransformer::new(RequestTransformConfig {
                body_transform: Some(body_config.clone()),
                ..Default::default()
            });
            request_transformer.transform_body_with_config(body, content_type, body_config)
        } else {
            Ok(body.to_vec())
        }
    }
}

impl Default for RequestTransformConfig {
    fn default() -> Self {
        Self {
            add_headers: None,
            remove_headers: None,
            header_mapping: None,
            strip_prefix: None,
            add_prefix: None,
            path_rewrites: None,
            add_query_params: None,
            remove_query_params: None,
            body_transform: None,
        }
    }
}

impl Default for ResponseTransformConfig {
    fn default() -> Self {
        Self {
            add_headers: None,
            remove_headers: None,
            header_mapping: None,
            status_code_mapping: None,
            body_transform: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, Uri};
    use std::collections::HashMap;

    #[test]
    fn test_request_header_transformation() {
        let mut config = RequestTransformConfig::default();
        let mut add_headers = HashMap::new();
        add_headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
        config.add_headers = Some(add_headers);
        config.remove_headers = Some(vec!["X-Remove-Me".to_string()]);

        let transformer = RequestTransformer::new(config);
        let mut headers = HeaderMap::new();
        headers.insert("X-Remove-Me", "should-be-removed".parse().unwrap());

        transformer.transform_headers(&mut headers).unwrap();

        assert!(!headers.contains_key("X-Remove-Me"));
        assert_eq!(headers.get("X-Custom-Header").unwrap(), "custom-value");
    }

    #[test]
    fn test_uri_path_transformation() {
        let mut config = RequestTransformConfig::default();
        config.strip_prefix = Some("/api/v1".to_string());
        config.add_prefix = Some("/v2".to_string());

        let transformer = RequestTransformer::new(config);
        let uri: Uri = "/api/v1/users".parse().unwrap();

        let transformed = transformer.transform_uri(&uri).unwrap();
        assert_eq!(transformed.path(), "/v2/users");
    }

    #[test]
    fn test_query_param_transformation() {
        let mut config = RequestTransformConfig::default();
        let mut add_params = HashMap::new();
        add_params.insert("source".to_string(), "proxy".to_string());
        config.add_query_params = Some(add_params);
        config.remove_query_params = Some(vec!["internal".to_string()]);

        let transformer = RequestTransformer::new(config);
        let uri: Uri = "/users?page=1&internal=true".parse().unwrap();

        let transformed = transformer.transform_uri(&uri).unwrap();
        let query = transformed.query().unwrap();
        
        assert!(query.contains("source=proxy"));
        assert!(!query.contains("internal=true"));
        assert!(query.contains("page=1"));
    }

    #[test]
    fn test_json_body_transformation() {
        let body_config = BodyTransformConfig {
            json_field_mapping: None,
            json_remove_fields: Some(vec!["password".to_string()]),
            json_add_fields: {
                let mut fields = HashMap::new();
                fields.insert("source".to_string(), Value::String("proxy".to_string()));
                Some(fields)
            },
            text_replacements: None,
            template: None,
        };

        let config = RequestTransformConfig {
            body_transform: Some(body_config),
            ..Default::default()
        };

        let transformer = RequestTransformer::new(config);
        let body = r#"{"username": "test", "password": "secret"}"#.as_bytes();

        let transformed = transformer.transform_body(body, Some("application/json")).unwrap();
        let transformed_str = String::from_utf8(transformed).unwrap();
        let transformed_json: Value = serde_json::from_str(&transformed_str).unwrap();

        assert!(!transformed_json.as_object().unwrap().contains_key("password"));
        assert_eq!(transformed_json["source"], "proxy");
        assert_eq!(transformed_json["username"], "test");
    }

    #[test]
    fn test_response_status_code_mapping() {
        let mut config = ResponseTransformConfig::default();
        let mut status_mapping = HashMap::new();
        status_mapping.insert(404, 200);
        config.status_code_mapping = Some(status_mapping);

        let transformer = ResponseTransformer::new(config);
        
        assert_eq!(transformer.transform_status_code(404), 200);
        assert_eq!(transformer.transform_status_code(500), 500); // Unchanged
    }
}
