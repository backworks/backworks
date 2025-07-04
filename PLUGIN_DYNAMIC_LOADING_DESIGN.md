# Dynamic Plugin Loading Architecture

## Current Issue
The current plugin system requires plugins to be compiled into the main Backworks binary. This prevents external developers from creating independent plugins.

## Solution: Dynamic Plugin Loading

### 1. Plugin SDK (Rust Crate)
Create a separate `backworks-plugin-sdk` crate that external developers can use:

```toml
# External developer's Cargo.toml
[package]
name = "my-custom-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # Creates a shared library

[dependencies]
backworks-plugin-sdk = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

### 2. Plugin ABI (Application Binary Interface)
Define a stable C-compatible interface for cross-language plugin support:

```rust
// backworks-plugin-sdk/src/abi.rs
use std::os::raw::c_char;
use std::ffi::CString;

#[repr(C)]
pub struct PluginInfo {
    pub name: *const c_char,
    pub version: *const c_char,
    pub description: *const c_char,
}

#[repr(C)]
pub struct PluginVTable {
    pub initialize: extern "C" fn(*const c_char) -> i32,
    pub shutdown: extern "C" fn() -> i32,
    pub health_check: extern "C" fn() -> i32,
    pub before_request: extern "C" fn(*const c_char) -> *const c_char,
    pub after_response: extern "C" fn(*const c_char) -> *const c_char,
    pub process_endpoint: extern "C" fn(*const c_char, *const c_char, *const c_char) -> *const c_char,
}

// Plugin must export these symbols
#[no_mangle]
pub extern "C" fn plugin_info() -> PluginInfo { ... }

#[no_mangle]
pub extern "C" fn plugin_vtable() -> PluginVTable { ... }
```

### 3. Plugin Registry and Discovery
```rust
// src/plugin/registry.rs
pub struct PluginRegistry {
    plugin_directories: Vec<PathBuf>,
    loaded_plugins: HashMap<String, LoadedPlugin>,
}

impl PluginRegistry {
    pub async fn scan_for_plugins(&mut self) -> Result<Vec<PluginMetadata>> {
        // Scan directories for .so/.dll/.dylib files
        // Load plugin info without initializing
    }
    
    pub async fn load_plugin(&mut self, path: &Path) -> Result<Box<dyn BackworksPlugin>> {
        // Dynamic library loading
        unsafe {
            let lib = libloading::Library::new(path)?;
            let get_info: libloading::Symbol<extern "C" fn() -> PluginInfo> = 
                lib.get(b"plugin_info")?;
            let get_vtable: libloading::Symbol<extern "C" fn() -> PluginVTable> = 
                lib.get(b"plugin_vtable")?;
            
            // Create plugin wrapper
            Ok(Box::new(DynamicPlugin::new(lib, get_info(), get_vtable())))
        }
    }
}
```

### 4. Plugin Configuration Format
```yaml
# backworks.yaml
plugins:
  # Built-in plugins (compiled in)
  auth:
    type: "builtin"
    config:
      jwt_secret: "secret"
  
  # External plugins (dynamically loaded)
  my-custom-plugin:
    type: "external"
    path: "./plugins/my-custom-plugin.so"
    config:
      api_key: "12345"
      
  # Marketplace plugins (downloaded)
  stripe-plugin:
    type: "marketplace"
    version: "1.2.0"
    config:
      api_key: "${STRIPE_API_KEY}"
```

### 5. Plugin Marketplace Integration
```rust
pub struct PluginMarketplace {
    registry_url: String,
    cache_dir: PathBuf,
}

impl PluginMarketplace {
    pub async fn search(&self, query: &str) -> Result<Vec<PluginListing>> {
        // Search marketplace API
    }
    
    pub async fn install(&self, name: &str, version: &str) -> Result<PathBuf> {
        // Download, verify signature, install
    }
    
    pub async fn update(&self, name: &str) -> Result<()> {
        // Update to latest version
    }
}
```

## Migration Strategy

### Phase 1: Extract Plugin SDK
1. Create `backworks-plugin-sdk` crate
2. Move plugin traits to SDK
3. Update existing plugins to use SDK

### Phase 2: Dynamic Loading
1. Implement ABI layer
2. Add plugin registry
3. Support external .so/.dll files

### Phase 3: Marketplace
1. Plugin marketplace API
2. Plugin signing/verification
3. Automatic updates

## External Plugin Example

```rust
// External developer creates: my-weather-plugin/src/lib.rs
use backworks_plugin_sdk::*;

#[derive(Default)]
pub struct WeatherPlugin {
    api_key: String,
}

#[async_trait]
impl BackworksPlugin for WeatherPlugin {
    fn name(&self) -> &str { "weather" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "Weather API integration" }
    
    async fn initialize(&self, config: &Value) -> BackworksResult<()> {
        // Initialize with API key from config
        Ok(())
    }
    
    async fn process_endpoint_data(&self, endpoint: &str, method: &str, data: &str) -> BackworksResult<Option<String>> {
        if endpoint == "/weather" && method == "GET" {
            // Call weather API and return data
            Ok(Some(json!({"temperature": 25, "humidity": 60}).to_string()))
        } else {
            Ok(None)
        }
    }
}

// Export plugin
export_plugin!(WeatherPlugin);
```

This architecture ensures complete separation between core and plugins while enabling external development.
