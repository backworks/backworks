use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use libloading::{Library, Symbol};
use serde_json::Value;
use crate::error::{BackworksError, Result as BackworksResult};
use crate::plugin::{BackworksPlugin, PluginHealth, HealthStatus};

/// Dynamic plugin loader that can load external compiled plugins
pub struct DynamicPluginLoader {
    plugin_directories: Vec<PathBuf>,
    loaded_libraries: Arc<RwLock<HashMap<String, Library>>>,
}

impl DynamicPluginLoader {
    pub fn new() -> Self {
        Self {
            plugin_directories: vec![
                PathBuf::from("./plugins"),
                PathBuf::from("./external_plugins"),
                PathBuf::from("/usr/local/lib/backworks/plugins"),
            ],
            loaded_libraries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a directory to scan for plugins
    pub fn add_plugin_directory<P: AsRef<Path>>(&mut self, path: P) {
        self.plugin_directories.push(path.as_ref().to_path_buf());
    }

    /// Scan for available plugins in all directories
    pub async fn scan_plugins(&self) -> BackworksResult<Vec<PluginMetadata>> {
        let mut plugins = Vec::new();
        
        for dir in &self.plugin_directories {
            if !dir.exists() {
                continue;
            }
            
            let mut entries = tokio::fs::read_dir(dir).await
                .map_err(|e| BackworksError::Io(e))?;
                
            while let Some(entry) = entries.next_entry().await
                .map_err(|e| BackworksError::Io(e))? {
                
                let path = entry.path();
                if self.is_plugin_file(&path) {
                    if let Ok(metadata) = self.get_plugin_metadata(&path).await {
                        plugins.push(metadata);
                    }
                }
            }
        }
        
        Ok(plugins)
    }

    /// Load a plugin from a specific path
    pub async fn load_plugin<P: AsRef<Path>>(&self, path: P) -> BackworksResult<Box<dyn BackworksPlugin>> {
        let path = path.as_ref();
        let plugin_name = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| BackworksError::Config("Invalid plugin filename".to_string()))?;

        // Load the dynamic library
        let lib = unsafe { Library::new(path) }
            .map_err(|e| BackworksError::Config(format!("Failed to load plugin library: {}", e)))?;

        // Store the library to keep it alive
        self.loaded_libraries.write().await.insert(plugin_name.to_string(), lib);

        // Get the library reference again (since we moved it above)
        let libraries = self.loaded_libraries.read().await;
        let lib = libraries.get(plugin_name).unwrap();

        // Create dynamic plugin wrapper with access to loaded libraries
        let dynamic_plugin = DynamicPlugin::new(lib, plugin_name, self.loaded_libraries.clone())?;
        Ok(Box::new(dynamic_plugin))
    }

    /// Get plugin metadata without fully loading the plugin
    async fn get_plugin_metadata<P: AsRef<Path>>(&self, path: P) -> BackworksResult<PluginMetadata> {
        let lib = unsafe { Library::new(path.as_ref()) }
            .map_err(|e| BackworksError::Config(format!("Failed to load plugin for metadata: {}", e)))?;

        let get_info: Symbol<extern "C" fn() -> PluginInfo> = unsafe {
            lib.get(b"plugin_info")
                .map_err(|e| BackworksError::Config(format!("Plugin missing plugin_info function: {}", e)))?
        };

        let info = get_info();
        
        let name = unsafe { CStr::from_ptr(info.name).to_string_lossy().to_string() };
        let version = unsafe { CStr::from_ptr(info.version).to_string_lossy().to_string() };
        let description = unsafe { CStr::from_ptr(info.description).to_string_lossy().to_string() };

        Ok(PluginMetadata {
            name,
            version,
            description,
            path: path.as_ref().to_path_buf(),
        })
    }

    /// Check if a file is a plugin library
    fn is_plugin_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            matches!(ext.as_str(), "so" | "dll" | "dylib")
        } else {
            false
        }
    }
}

/// Metadata about a discovered plugin
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub path: PathBuf,
}

/// C-compatible plugin info structure (must match SDK)
#[repr(C)]
struct PluginInfo {
    name: *const c_char,
    version: *const c_char,
    description: *const c_char,
}

/// Wrapper around a dynamically loaded plugin
pub struct DynamicPlugin {
    name: String,
    version: String,
    description: String,
    library_name: String,
    libraries: Arc<RwLock<HashMap<String, Library>>>,
}

impl DynamicPlugin {
    fn new(lib: &Library, plugin_name: &str, libraries: Arc<RwLock<HashMap<String, Library>>>) -> BackworksResult<Self> {
        // Get plugin info
        let get_info: Symbol<extern "C" fn() -> PluginInfo> = unsafe {
            lib.get(b"plugin_info")
                .map_err(|e| BackworksError::Config(format!("Plugin missing plugin_info function: {}", e)))?
        };

        let info = get_info();
        
        let name = unsafe { CStr::from_ptr(info.name).to_string_lossy().to_string() };
        let version = unsafe { CStr::from_ptr(info.version).to_string_lossy().to_string() };
        let description = unsafe { CStr::from_ptr(info.description).to_string_lossy().to_string() };

        Ok(Self {
            name,
            version,
            description,
            library_name: plugin_name.to_string(),
            libraries,
        })
    }

    /// Call a plugin function safely
    fn call_plugin_function<'a, T>(&self, libraries: &'a HashMap<String, Library>, func_name: &[u8]) -> BackworksResult<Symbol<'a, T>> {
        let lib = libraries.get(&self.library_name)
            .ok_or_else(|| BackworksError::Config(format!("Plugin library not found: {}", self.library_name)))?;

        unsafe {
            lib.get(func_name)
                .map_err(|e| BackworksError::Config(format!("Plugin function not found: {}", e)))
        }
    }
}

#[async_trait::async_trait]
impl BackworksPlugin for DynamicPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn initialize(&self, config: &Value) -> BackworksResult<()> {
        let libraries = self.libraries.read().await;
        let lib = libraries.get(&self.library_name)
            .ok_or_else(|| BackworksError::Config(format!("Plugin library not found: {}", self.library_name)))?;
        
        // Try to get the initialize function
        let init_func: Result<Symbol<extern "C" fn(*const c_char) -> i32>, _> = unsafe {
            lib.get(b"plugin_initialize")
        };
        
        if let Ok(init_func) = init_func {
            let config_str = config.to_string();
            let config_cstr = CString::new(config_str)
                .map_err(|e| BackworksError::Config(format!("Invalid config string: {}", e)))?;
            
            let result = init_func(config_cstr.as_ptr());
            if result != 0 {
                return Err(BackworksError::Config(format!("Plugin {} initialization failed", self.name)));
            }
        }
        // If no initialize function, that's OK - some plugins might not need it
        
        Ok(())
    }

    async fn shutdown(&self) -> BackworksResult<()> {
        let libraries = self.libraries.read().await;
        let lib = libraries.get(&self.library_name)
            .ok_or_else(|| BackworksError::Config(format!("Plugin library not found: {}", self.library_name)))?;
        
        // Try to get the shutdown function
        let shutdown_func: Result<Symbol<extern "C" fn() -> i32>, _> = unsafe {
            lib.get(b"plugin_shutdown")
        };
        
        if let Ok(shutdown_func) = shutdown_func {
            let result = shutdown_func();
            if result != 0 {
                return Err(BackworksError::Config(format!("Plugin {} shutdown failed", self.name)));
            }
        }
        // If no shutdown function, that's OK
        
        Ok(())
    }

    async fn health_check(&self) -> BackworksResult<PluginHealth> {
        // Default healthy status for dynamic plugins
        Ok(PluginHealth {
            status: HealthStatus::Healthy,
            message: format!("Dynamic plugin {} is loaded", self.name),
            details: std::collections::HashMap::new(),
        })
    }

    async fn process_endpoint_data(&self, endpoint: &str, method: &str, data: &str) -> BackworksResult<Option<String>> {
        let libraries = self.libraries.read().await;
        let lib = libraries.get(&self.library_name)
            .ok_or_else(|| BackworksError::Config(format!("Plugin library not found: {}", self.library_name)))?;
        
        // Try to get the process function
        let process_func: Result<Symbol<extern "C" fn(*const c_char, *const c_char, *const c_char) -> *const c_char>, _> = unsafe {
            lib.get(b"plugin_process_endpoint")
        };
        
        if let Ok(process_func) = process_func {
            let endpoint_cstr = CString::new(endpoint)
                .map_err(|e| BackworksError::Config(format!("Invalid endpoint string: {}", e)))?;
            let method_cstr = CString::new(method)
                .map_err(|e| BackworksError::Config(format!("Invalid method string: {}", e)))?;
            let data_cstr = CString::new(data)
                .map_err(|e| BackworksError::Config(format!("Invalid data string: {}", e)))?;
            
            let result = process_func(endpoint_cstr.as_ptr(), method_cstr.as_ptr(), data_cstr.as_ptr());
            
            if result.is_null() {
                return Ok(None);
            }
            
            let response = unsafe { CStr::from_ptr(result).to_string_lossy().to_string() };
            return Ok(Some(response));
        }
        
        // If no process function, this plugin doesn't handle endpoints
        Ok(None)
    }
}
