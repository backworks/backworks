# External Plugins for Backworks

This directory contains external plugin implementations for Backworks.

## Plugin Structure

Each plugin is a separate Rust crate that implements the `BackworksPlugin` trait from the core `backworks` library.

## Available Plugins

- **backworks-sqlite-plugin**: SQLite database integration plugin
- **backworks-postgres-plugin**: PostgreSQL database integration plugin (future)
- **backworks-redis-plugin**: Redis cache integration plugin (future)
- **backworks-auth-plugin**: Authentication/authorization plugin (future)

## Creating New Plugins

To create a new plugin:

1. Create a new directory: `mkdir my-plugin`
2. Initialize: `cd my-plugin && cargo init --lib`
3. Add `backworks` as dependency in `Cargo.toml`
4. Implement the `BackworksPlugin` trait
5. Export your plugin in `lib.rs`

## Plugin Interface

All plugins must implement the `BackworksPlugin` trait:

```rust
use backworks::{BackworksPlugin, BackworksResult, PluginHealth};
use async_trait::async_trait;

#[async_trait]
impl BackworksPlugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "My custom plugin" }
    
    async fn initialize(&self, config: &serde_json::Value) -> BackworksResult<()> {
        // Plugin initialization logic
        Ok(())
    }
    
    async fn shutdown(&self) -> BackworksResult<()> {
        // Cleanup logic
        Ok(())
    }
    
    // Additional hooks as needed...
}
```

## Core Philosophy

- **Core**: Provides only the plugin architecture framework
- **Plugins**: Implement all specific functionality (database, auth, caching, etc.)
- **Separation**: Core has no knowledge of specific plugin implementations
- **Modularity**: Each plugin is a separate, independent project
