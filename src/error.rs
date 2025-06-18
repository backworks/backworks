use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Error, Debug)]
pub enum BackworksError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Server error: {0}")]
    Server(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Runtime error: {0}")]
    Runtime(String),
    
    #[error("AI processing error: {0}")]
    AI(String),
    
    #[error("Proxy error: {0}")]
    Proxy(String),
    
    #[error("Capture error: {0}")]
    Capture(String),
    
    #[error("HTTP client error: {0}")]
    Http(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_yaml::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("SQL error: {0}")]
    Sql(#[from] sqlx::Error),
    
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    
    #[error("Template error: {0}")]
    Template(#[from] handlebars::TemplateError),
    
    #[error("Render error: {0}")]
    Render(#[from] handlebars::RenderError),
    
    // Plugin system errors
    #[error("Plugin initialization failed: {0}")]
    PluginInitializationFailed(String),
    
    #[error("Plugin timeout: {0}")]
    PluginTimeout(String),
    
    #[error("Critical plugin failure: {0:?}")]
    CriticalPluginFailure(Vec<String>),
    
    #[error("Plugin configuration invalid: {0}")]
    PluginConfigInvalid(String),
    
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),
}

impl BackworksError {
    pub fn config<T: ToString>(msg: T) -> Self {
        Self::Config(msg.to_string())
    }
    
    pub fn server<T: ToString>(msg: T) -> Self {
        Self::Server(msg.to_string())
    }
    
    pub fn database<T: ToString>(msg: T) -> Self {
        Self::Database(msg.to_string())
    }
    
    pub fn runtime<T: ToString>(msg: T) -> Self {
        Self::Runtime(msg.to_string())
    }
    
    pub fn ai<T: ToString>(msg: T) -> Self {
        Self::AI(msg.to_string())
    }
    
    pub fn http<T: ToString>(msg: T) -> Self {
        Self::Http(msg.to_string())
    }
    
    pub fn proxy<T: ToString>(msg: T) -> Self {
        Self::Proxy(msg.to_string())
    }
    
    pub fn capture<T: ToString>(msg: T) -> Self {
        Self::Capture(msg.to_string())
    }
}

impl IntoResponse for BackworksError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            BackworksError::Config(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            BackworksError::Runtime(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            BackworksError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            BackworksError::AI(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            BackworksError::Proxy(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            BackworksError::Capture(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            BackworksError::Http(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            BackworksError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            BackworksError::Serialization(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            BackworksError::Json(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            BackworksError::Sql(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            BackworksError::Server(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            BackworksError::Request(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            BackworksError::Template(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            BackworksError::Render(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            BackworksError::PluginInitializationFailed(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            BackworksError::PluginTimeout(_) => (StatusCode::GATEWAY_TIMEOUT, self.to_string()),
            BackworksError::CriticalPluginFailure(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            BackworksError::PluginConfigInvalid(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            BackworksError::PluginNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
        };

        let body = Json(serde_json::json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, BackworksError>;
pub type BackworksResult<T> = Result<T>;
