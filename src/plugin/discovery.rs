use std::path::Path;
use crate::error::{BackworksError, Result as BackworksResult};
use crate::config::PluginDiscoveryConfig;
use crate::plugin::dynamic::{DynamicPluginLoader, PluginMetadata};
use tokio::fs;

/// Plugin discovery service that can find external plugins in configured directories
pub struct PluginDiscovery {
    config: PluginDiscoveryConfig,
    loader: DynamicPluginLoader,
}

impl PluginDiscovery {
    pub fn new(config: PluginDiscoveryConfig) -> Self {
        let loader = DynamicPluginLoader::new();
        
        // Note: Plugin directories are configured in the discovery config
        // The loader will use default directories
        
        Self { config, loader }
    }
    
    /// Discover all available plugins in configured directories
    pub async fn discover_all_plugins(&self) -> BackworksResult<Vec<PluginMetadata>> {
        if !self.config.enabled {
            tracing::debug!("Plugin discovery is disabled");
            return Ok(Vec::new());
        }
        
        let mut all_plugins = Vec::new();
        
        for directory in &self.config.directories {
            if !directory.exists() {
                tracing::debug!("Plugin directory does not exist: {}", directory.display());
                continue;
            }
            
            let plugins = self.discover_plugins_in_directory(directory).await?;
            all_plugins.extend(plugins);
        }
        
        tracing::info!("Discovered {} external plugins", all_plugins.len());
        Ok(all_plugins)
    }
    
    /// Discover plugins in a specific directory
    fn discover_plugins_in_directory<'a>(&'a self, directory: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = BackworksResult<Vec<PluginMetadata>>> + Send + 'a>> {
        Box::pin(async move {
            let mut plugins = Vec::new();
            
            let mut entries = fs::read_dir(directory).await
                .map_err(|e| BackworksError::Io(e))?;
            
            while let Some(entry) = entries.next_entry().await
                .map_err(|e| BackworksError::Io(e))? {
                
                let path = entry.path();
                
                if path.is_file() && self.is_plugin_file(&path) {
                    match self.get_plugin_metadata(&path).await {
                        Ok(metadata) => {
                            tracing::debug!("Found plugin: {} at {}", metadata.name, path.display());
                            plugins.push(metadata);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to read plugin metadata from {}: {}", path.display(), e);
                        }
                    }
                } else if path.is_dir() && self.config.recursive {
                    // Recursively search subdirectories
                    let sub_plugins = self.discover_plugins_in_directory(&path).await?;
                    plugins.extend(sub_plugins);
                }
            }
            
            Ok(plugins)
        })
    }
    
    /// Get metadata for a specific plugin file
    async fn get_plugin_metadata(&self, path: &Path) -> BackworksResult<PluginMetadata> {
        // Extract metadata safely without keeping the library loaded
        self.extract_metadata_safely(path).await
    }
    
    /// Safely extract plugin metadata without keeping the library loaded
    async fn extract_metadata_safely(&self, path: &Path) -> BackworksResult<PluginMetadata> {
        use std::ffi::CStr;
        use std::os::raw::c_char;
        use libloading::{Library, Symbol};
        
        // This is similar to the loader's implementation but safer for discovery
        let lib = unsafe { Library::new(path) }
            .map_err(|e| BackworksError::Config(format!("Failed to load plugin for metadata: {}", e)))?;

        #[repr(C)]
        struct PluginInfo {
            name: *const c_char,
            version: *const c_char,
            description: *const c_char,
        }

        let get_info: Symbol<extern "C" fn() -> PluginInfo> = unsafe {
            lib.get(b"plugin_info")
                .map_err(|e| BackworksError::Config(format!("Plugin missing plugin_info function: {}", e)))?
        };

        let info = get_info();
        
        let name = unsafe { 
            CStr::from_ptr(info.name).to_string_lossy().to_string() 
        };
        let version = unsafe { 
            CStr::from_ptr(info.version).to_string_lossy().to_string() 
        };
        let description = unsafe { 
            CStr::from_ptr(info.description).to_string_lossy().to_string() 
        };

        Ok(PluginMetadata {
            name,
            version,
            description,
            path: path.to_path_buf(),
        })
        // Library is automatically dropped here, unloading the plugin
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
    
    /// Validate that a plugin file is loadable
    pub async fn validate_plugin<P: AsRef<Path>>(&self, path: P) -> BackworksResult<PluginMetadata> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(BackworksError::Config(format!("Plugin file does not exist: {}", path.display())));
        }
        
        if !self.is_plugin_file(path) {
            return Err(BackworksError::Config(format!("File is not a plugin library: {}", path.display())));
        }
        
        self.get_plugin_metadata(path).await
    }
}

/// Plugin registry for managing discovered plugins
pub struct PluginRegistry {
    plugins: Vec<PluginMetadata>,
    discovery: PluginDiscovery,
}

impl PluginRegistry {
    pub fn new(config: PluginDiscoveryConfig) -> Self {
        Self {
            plugins: Vec::new(),
            discovery: PluginDiscovery::new(config),
        }
    }
    
    /// Refresh the plugin registry by rediscovering plugins
    pub async fn refresh(&mut self) -> BackworksResult<()> {
        self.plugins = self.discovery.discover_all_plugins().await?;
        tracing::info!("Plugin registry refreshed with {} plugins", self.plugins.len());
        Ok(())
    }
    
    /// Get all discovered plugins
    pub fn get_all_plugins(&self) -> &[PluginMetadata] {
        &self.plugins
    }
    
    /// Find a plugin by name
    pub fn find_plugin(&self, name: &str) -> Option<&PluginMetadata> {
        self.plugins.iter().find(|p| p.name == name)
    }
    
    /// Validate a specific plugin
    pub async fn validate_plugin<P: AsRef<Path>>(&self, path: P) -> BackworksResult<PluginMetadata> {
        self.discovery.validate_plugin(path).await
    }
}
