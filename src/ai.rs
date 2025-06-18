//! Legacy AI module - DEPRECATED
//! 
//! This module is kept for backward compatibility.
//! AI features are now provided through the plugin system.
//! See `plugins::ai` for the new AI plugin implementation.

use crate::error::BackworksError;
use serde::{Deserialize, Serialize};

/// Legacy AI enhancer - deprecated in favor of AI plugin
#[derive(Debug, Clone)]
pub struct AIEnhancer {
    // Empty struct for compatibility
}

impl AIEnhancer {
    /// Creates a new AI enhancer (deprecated)
    /// 
    /// This method is kept for backward compatibility but does nothing.
    /// Use the AI plugin system instead.
    pub fn new(_config: serde_json::Value) -> Self {
        tracing::warn!("AIEnhancer is deprecated. Use the AI plugin system instead.");
        Self {}
    }

    /// Analyze request (deprecated)
    pub async fn analyze_request(&self, _data: &str) -> Result<AIAnalysis, BackworksError> {
        // Return empty analysis for compatibility
        Ok(AIAnalysis::default())
    }

    /// Enhance response (deprecated)  
    pub async fn enhance_response(&self, _original: &str) -> Result<String, BackworksError> {
        // Return original response unchanged
        Ok(_original.to_string())
    }

    /// Detect patterns (deprecated)
    pub async fn detect_patterns(&self, _requests: &[String]) -> Result<Vec<String>, BackworksError> {
        // Return empty patterns for compatibility
        Ok(vec![])
    }
}

/// AI analysis result structure (for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysis {
    pub patterns: Vec<String>,
    pub anomalies: Vec<String>, 
    pub suggestions: Vec<String>,
    pub confidence: f64,
}

impl Default for AIAnalysis {
    fn default() -> Self {
        Self {
            patterns: vec![],
            anomalies: vec![],
            suggestions: vec![],
            confidence: 0.0,
        }
    }
}
