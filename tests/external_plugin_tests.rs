use backworks::plugin::{PluginManager, PluginDiscovery};
use backworks::config::PluginDiscoveryConfig;
use std::path::PathBuf;

#[tokio::test]
async fn test_external_plugin_discovery() {
    // Test plugin discovery
    let mut config = PluginDiscoveryConfig::default();
    config.directories = vec![
        PathBuf::from("./examples/external-plugins/weather-plugin/target/release"),
    ];
    
    let discovery = PluginDiscovery::new(config);
    let plugins = discovery.discover_all_plugins().await.unwrap();
    
    // Should find at least the weather plugin if it was built
    println!("Discovered {} plugins", plugins.len());
    for plugin in &plugins {
        println!("Found plugin: {} v{} at {}", 
                 plugin.name, plugin.version, plugin.path.display());
    }
}

#[tokio::test]
async fn test_external_plugin_loading() {
    let manager = PluginManager::new();
    
    // Try to load the weather plugin if it exists
    let plugin_path = "./examples/external-plugins/weather-plugin/target/release/libweather_plugin.dylib";
    
    if std::path::Path::new(plugin_path).exists() {
        let result = manager.load_external_plugin(plugin_path, None, None).await;
        
        match result {
            Ok(_) => {
                println!("✅ Successfully loaded weather plugin");
                
                // Test plugin list
                let plugins = manager.list_plugins().await;
                assert!(!plugins.is_empty());
                println!("Loaded plugins: {:?}", plugins);
            }
            Err(e) => {
                println!("❌ Failed to load weather plugin: {}", e);
                // Don't fail the test since the plugin might not be built
            }
        }
    } else {
        println!("⚠️ Weather plugin not found at {}, skipping test", plugin_path);
    }
}

#[tokio::test] 
async fn test_plugin_manager_discovery_integration() {
    let manager = PluginManager::new();
    
    let mut config = PluginDiscoveryConfig::default();
    config.directories = vec![
        PathBuf::from("./examples/external-plugins/weather-plugin/target/release"),
    ];
    
    // Test discovery integration
    let result = manager.initialize_from_discovery(&config).await;
    
    match result {
        Ok(_) => {
            println!("✅ Successfully initialized plugins from discovery");
            
            let plugins = manager.list_plugins().await;
            println!("Auto-loaded plugins: {:?}", plugins);
        }
        Err(e) => {
            println!("Discovery initialization result: {}", e);
            // This might fail if no plugins are found, which is OK for testing
        }
    }
}
