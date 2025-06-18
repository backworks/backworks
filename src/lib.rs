//! # Backworks
//! 
//! Configuration-driven API platform that works backwards from your needs.
//! 
//! Backworks enables rapid API creation, evolution, and monitoring via a single YAML configuration.
//! It supports multiple execution modes (mock, capture, runtime, database, proxy, hybrid) and 
//! AI-powered enhancements for modern API development.

// Re-export main modules for library usage
pub mod config;
pub mod engine;
pub mod server;
pub mod error;
pub mod mock;
pub mod ai;
pub mod dashboard;
pub mod runtime;
pub mod database;
pub mod capture;
pub mod proxy;

// Re-export commonly used types
pub use config::BackworksConfig;
pub use engine::BackworksEngine;
pub use error::{BackworksError, Result};
