//! # Backworks
//! 
//! Configuration-driven API platform that works backwards from your needs.
//! 
//! Backworks enables rapid API creation, evolution, and monitoring via a single YAML configuration.
//! It supports multiple execution modes (mock, capture, runtime, hybrid) and 
//! plugin-powered enhancements for modern API development.
//!
//! ## Pure Plugin Architecture
//! 
//! Backworks core provides only the plugin architecture framework.
//! All specific functionality (database, auth, caching, proxy, etc.) is implemented
//! as external plugins in the `plugins/` directory.

// Re-export main modules for library usage
pub mod config;
pub mod engine;
pub mod server;
pub mod error;
pub mod plugin;
pub mod resilience;
pub mod dashboard;
pub mod runtime;
pub mod capture;
pub mod analyzer;

// Re-export commonly used types
pub use config::BackworksConfig;
pub use engine::BackworksEngine;
pub use error::{BackworksError, Result};
pub use plugin::{BackworksPlugin, PluginManager, PluginHealth, HealthStatus};
pub use resilience::{ResilientPluginConfig, PluginMetrics};
