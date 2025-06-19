use clap::{Parser, Subcommand};
use std::path::PathBuf;

use backworks::{
    BackworksEngine, BackworksError, Result,
    config, capture
};

#[derive(Parser)]
#[command(name = "backworks")]
#[command(about = "Configuration-driven API platform that works backwards")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Backworks API server
    Start {
        /// Configuration file path
        #[arg(short, long, default_value = "project.yaml")]
        config: PathBuf,
        
        /// Override the server port
        #[arg(short, long)]
        port: Option<u16>,
        
        /// Override the dashboard port
        #[arg(long)]
        dashboard_port: Option<u16>,
        
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Validate configuration file
    Validate {
        /// Configuration file path
        #[arg(short, long, default_value = "project.yaml")]
        config: PathBuf,
    },
    
    /// Initialize a new Backworks project
    Init {
        /// Project name
        name: Option<String>,
        
        /// Project template
        #[arg(short, long, default_value = "basic")]
        template: String,
    },
    
    /// Capture mode - listen and analyze existing APIs
    Capture {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        
        /// Output file for captured data
        #[arg(short, long, default_value = "captured.yaml")]
        output: PathBuf,
        
        /// Duration to capture (in seconds)
        #[arg(short, long)]
        duration: Option<u64>,
    },
    
    /// Generate configuration from captured data
    Generate {
        /// Input captured data file
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output configuration file
        #[arg(short, long, default_value = "generated.yaml")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    init_logging(matches!(cli.command, Commands::Start { verbose: true, .. }));
    
    match cli.command {
        Commands::Start { config, port, dashboard_port, verbose: _ } => {
            start_server(config, port, dashboard_port).await
        }
        Commands::Validate { config } => {
            validate_config(config).await
        }
        Commands::Init { name, template } => {
            init_project(name, template).await
        }
        Commands::Capture { port, output, duration } => {
            start_capture_mode(port, output, duration).await
        }
        Commands::Generate { input, output } => {
            generate_config(input, output).await
        }
    }
}

async fn start_server(mut config_path: PathBuf, port: Option<u16>, dashboard_port: Option<u16>) -> Result<()> {
    println!("🚀 Starting Backworks...");
    
    // Auto-detect configuration file if default doesn't exist
    if !config_path.exists() && config_path.file_name().unwrap_or_default() == "project.yaml" {
        let blueprint_path = PathBuf::from("blueprint.yaml");
        if blueprint_path.exists() {
            println!("📋 Using blueprint.yaml (project.yaml not found)");
            config_path = blueprint_path;
        }
    }
    
    // Load configuration
    let config = config::load_config(&config_path).await?;
    println!("✅ Configuration loaded: {}", config.name);
    
    // Override ports if specified
    let mut config = config;
    if let Some(p) = port {
        config.server.port = p;
    }
    if let Some(dp) = dashboard_port {
        if let Some(ref mut dashboard) = config.dashboard {
            dashboard.port = dp;
        }
    }
    
    // Initialize the engine
    let engine = BackworksEngine::new(config).await?;
    println!("✅ Backworks engine initialized");
    
    // Start the server
    engine.start().await?;
    
    Ok(())
}

async fn validate_config(mut config_path: PathBuf) -> Result<()> {
    println!("🔍 Validating configuration...");
    
    // Auto-detect configuration file if default doesn't exist
    if !config_path.exists() && config_path.file_name().unwrap_or_default() == "project.yaml" {
        let blueprint_path = PathBuf::from("blueprint.yaml");
        if blueprint_path.exists() {
            println!("📋 Using blueprint.yaml (project.yaml not found)");
            config_path = blueprint_path;
        }
    }
    
    match config::load_config(&config_path).await {
        Ok(config) => {
            println!("✅ Configuration is valid");
            println!("   Name: {}", config.name);
            println!("   Mode: {:?}", config.mode);
            println!("   Endpoints: {}", config.endpoints.len());
            if !config.plugins.is_empty() {
                println!("   Plugins: {} configured", config.plugins.len());
                for (name, plugin_config) in &config.plugins {
                    if plugin_config.enabled {
                        println!("     - {} (enabled)", name);
                    }
                }
            }
            if config.dashboard.is_some() {
                println!("   Dashboard: Enabled");
            }
            Ok(())
        }
        Err(e) => {
            println!("❌ Configuration validation failed: {}", e);
            Err(e)
        }
    }
}

async fn init_project(name: Option<String>, template: String) -> Result<()> {
    let project_name = name.unwrap_or_else(|| "my-api".to_string());
    println!("🎯 Initializing new Backworks project: {}", project_name);
    
    // Create basic configuration based on template
    let config_content = match template.as_str() {
        "basic" => create_basic_template(&project_name),
        "ai" => create_ai_template(&project_name),
        "database" => create_database_template(&project_name),
        "microservices" => create_microservices_template(&project_name),
        _ => {
            println!("❌ Unknown template: {}", template);
            return Err(BackworksError::Config("Unknown template".to_string()));
        }
    };
    
    // Write configuration file
    std::fs::write("project.yaml", config_content)?;
    println!("✅ Created project.yaml");
    
    // Create directories
    std::fs::create_dir_all("handlers")?;
    std::fs::create_dir_all("data")?;
    println!("✅ Created project structure");
    
    println!("\n🚀 Project initialized! Run 'backworks start' to begin.");
    Ok(())
}

async fn start_capture_mode(port: u16, output: PathBuf, duration: Option<u64>) -> Result<()> {
    println!("🎯 Starting capture mode on port {}", port);
    
    let capturer = capture::Capturer::new(port, output.to_string_lossy().to_string());
    
    if let Some(duration) = duration {
        println!("⏱️  Capturing for {} seconds...", duration);
        capturer.capture_for_duration(std::time::Duration::from_secs(duration)).await?;
    } else {
        println!("🔍 Capturing indefinitely (Ctrl+C to stop)...");
        capturer.capture_indefinitely().await?;
    }
    
    Ok(())
}

async fn generate_config(input: PathBuf, output: PathBuf) -> Result<()> {
    println!("🔄 Generating configuration from captured data...");
    
    let capturer = capture::Capturer::new(8080, output.to_string_lossy().to_string());
    capturer.generate_from_file(input, output).await?;
    
    println!("✅ Configuration generated successfully");
    Ok(())
}

fn init_logging(verbose: bool) {
    let level = if verbose { "debug" } else { "info" };
    
    tracing_subscriber::fmt()
        .with_env_filter(format!("backworks={}", level))
        .with_target(false)
        .init();
}

fn create_basic_template(name: &str) -> String {
    format!(r#"name: "{}"
description: "A basic API created with Backworks"
version: "1.0.0"

mode: "proxy"

endpoints:
  users:
    path: "/users"
    methods: ["GET", "POST"]
    proxy:
      target: "https://jsonplaceholder.typicode.com"
          
  user_detail:
    path: "/users/{{id}}"
    methods: ["GET", "PUT", "DELETE"]
    proxy:
      target: "https://jsonplaceholder.typicode.com"

dashboard:
  enabled: true
  port: 3000
"#, name)
}

fn create_ai_template(name: &str) -> String {
    format!(r#"name: "{}"
description: "An AI-enhanced API created with Backworks"
version: "1.0.0"

mode: "plugin"

ai:
  enabled: true
  features:
    - "pattern_recognition"
    - "schema_prediction"
    - "traffic_analysis"

endpoints:
  smart_endpoint:
    path: "/smart"
    methods: ["GET", "POST"]
    ai_enhanced: true
    mock:
      ai_generated: true

dashboard:
  enabled: true
  port: 3000
  features:
    - "flows"
    - "metrics"
    - "ai_insights"
"#, name)
}

fn create_database_template(name: &str) -> String {
    format!(r#"name: "{}"
description: "A database-driven API created with Backworks"
version: "1.0.0"

mode: "database"

database:
  type: "postgresql"
  connection_string_env: "DATABASE_URL"

endpoints:
  users:
    path: "/users"
    methods: ["GET", "POST", "PUT", "DELETE"]
    database:
      table: "users"
      auto_crud: true

dashboard:
  enabled: true
  port: 3000
"#, name)
}

fn create_microservices_template(name: &str) -> String {
    format!(r#"name: "{}"
description: "A microservices gateway created with Backworks"
version: "1.0.0"

mode: "proxy"

endpoints:
  user_service:
    path: "/users/*"
    proxy:
      target: "http://user-service:8081"
      strip_prefix: "/users"
      
  order_service:
    path: "/orders/*"
    proxy:
      target: "http://order-service:8082"

dashboard:
  enabled: true
  port: 3000
"#, name)
}
