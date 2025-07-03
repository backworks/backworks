//! Error handling for the proxy plugin

use std::fmt;

/// Result type for proxy operations
pub type ProxyResult<T> = Result<T, ProxyError>;

/// Proxy plugin error types
#[derive(Debug, Clone)]
pub enum ProxyError {
    /// Configuration error
    Configuration(String),
    
    /// HTTP request/response error
    Http(String),
    
    /// Network connection error
    Network(String),
    
    /// Load balancing error
    LoadBalancing(String),
    
    /// Circuit breaker error
    CircuitBreaker(String),
    
    /// Health check error
    HealthCheck(String),
    
    /// Transformation error
    Transformation(String),
    
    /// Timeout error
    Timeout(String),
    
    /// Target unavailable error
    TargetUnavailable(String),
    
    /// Internal plugin error
    Internal(String),
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            ProxyError::Http(msg) => write!(f, "HTTP error: {}", msg),
            ProxyError::Network(msg) => write!(f, "Network error: {}", msg),
            ProxyError::LoadBalancing(msg) => write!(f, "Load balancing error: {}", msg),
            ProxyError::CircuitBreaker(msg) => write!(f, "Circuit breaker error: {}", msg),
            ProxyError::HealthCheck(msg) => write!(f, "Health check error: {}", msg),
            ProxyError::Transformation(msg) => write!(f, "Transformation error: {}", msg),
            ProxyError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
            ProxyError::TargetUnavailable(msg) => write!(f, "Target unavailable: {}", msg),
            ProxyError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ProxyError {}

impl From<reqwest::Error> for ProxyError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            ProxyError::Timeout(error.to_string())
        } else if error.is_connect() {
            ProxyError::Network(error.to_string())
        } else {
            ProxyError::Http(error.to_string())
        }
    }
}

impl From<hyper::Error> for ProxyError {
    fn from(error: hyper::Error) -> Self {
        ProxyError::Http(error.to_string())
    }
}

impl From<url::ParseError> for ProxyError {
    fn from(error: url::ParseError) -> Self {
        ProxyError::Configuration(format!("URL parse error: {}", error))
    }
}

impl From<serde_json::Error> for ProxyError {
    fn from(error: serde_json::Error) -> Self {
        ProxyError::Transformation(format!("JSON error: {}", error))
    }
}

impl From<ProxyError> for backworks::error::BackworksError {
    fn from(error: ProxyError) -> Self {
        match error {
            ProxyError::Configuration(msg) => backworks::error::BackworksError::config(msg),
            ProxyError::Http(msg) => backworks::error::BackworksError::server(msg),
            ProxyError::Network(msg) => backworks::error::BackworksError::server(msg),
            ProxyError::Timeout(msg) => backworks::error::BackworksError::server(msg),
            ProxyError::Internal(msg) => backworks::error::BackworksError::server(msg),
            _ => backworks::error::BackworksError::plugin(format!("Proxy plugin error: {}", error)),
        }
    }
}
