use clap::{Parser, Subcommand};
use std::path::PathBuf;

use backworks::{
    BackworksEngine, BackworksError, Result,
    config, capture, analyzer
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
    /// Initialize a new Backworks project
    Init {
        /// Project name
        name: String,
        
        /// Project template (hello-world, api, webapp)
        #[arg(short, long, default_value = "hello-world")]
        template: String,
    },
    
    /// Start the Backworks API server
    Start {
        /// Configuration file path (optional for project structure)
        #[arg(short, long)]
        config: Option<PathBuf>,
        
        /// Override the server port
        #[arg(short, long)]
        port: Option<u16>,
        
        /// Override the dashboard port
        #[arg(long)]
        dashboard_port: Option<u16>,
        
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
        
        /// Enable hot reload
        #[arg(short, long)]
        watch: bool,
    },
    
    /// Build the project for deployment
    Build {
        /// Target profile (development, production)
        #[arg(short, long, default_value = "development")]
        target: String,
        
        /// Security profile
        #[arg(short, long)]
        security: Option<String>,
        
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Migrate from single file to project structure
    Migrate {
        /// Source blueprint file
        #[arg(long)]
        from: PathBuf,
        
        /// Target format (package.json)
        #[arg(long, default_value = "package.json")]
        to: String,
    },
    
    /// Validate configuration
    Validate {
        /// Configuration file path (optional for project structure)
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    
    /// Analyze blueprint configuration with detailed feedback
    Analyze {
        /// Configuration file path (optional for project structure)
        #[arg(short, long)]
        config: Option<PathBuf>,
        
        /// Output format (text, json, yaml)
        #[arg(short, long, default_value = "text")]
        format: String,
        
        /// Output file (optional, defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
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
    let verbose = matches!(cli.command, Commands::Start { verbose: true, .. });
    init_logging(verbose);
    
    match cli.command {
        Commands::Init { name, template } => {
            init_project(name, template).await
        }
        Commands::Start { config, port, dashboard_port, verbose: _, watch } => {
            start_server(config, port, dashboard_port, watch).await
        }
        Commands::Build { target, security, output } => {
            build_project(target, security, output).await
        }
        Commands::Migrate { from, to } => {
            migrate_project(from, to).await
        }
        Commands::Validate { config } => {
            validate_config(config).await
        }
        Commands::Analyze { config, format, output } => {
            analyze_blueprint(config, Some(format), output).await
        }
        Commands::Capture { port, output, duration } => {
            start_capture_mode(port, output, duration).await
        }
        Commands::Generate { input, output } => {
            generate_config(input, output).await
        }
    }
}

async fn start_server(config_path: Option<PathBuf>, port: Option<u16>, dashboard_port: Option<u16>, watch: bool) -> Result<()> {
    println!("🚀 Starting Backworks...");
    
    // Load project configuration (auto-detects project vs single file)
    let (metadata, mut config) = config::load_project_config(config_path)?;
    
    if let Some(ref metadata) = metadata {
        println!("✅ Project loaded: {} v{}", metadata.name, metadata.version);
    } else {
        println!("✅ Configuration loaded: {}", config.name);
    }
    
    // Override ports if specified
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
    
    if watch {
        println!("👁️  Hot reload enabled");
        // TODO: Implement file watching
    }
    
    // Start the server
    engine.start().await?;
    
    Ok(())
}

async fn init_project(name: String, template: String) -> Result<()> {
    println!("🚀 Initializing new Backworks project: {}", name);
    
    // Create project directory
    let project_dir = PathBuf::from(&name);
    if project_dir.exists() {
        return Err(BackworksError::config(format!("Directory '{}' already exists", name)));
    }
    
    std::fs::create_dir_all(&project_dir)
        .map_err(|e| BackworksError::config(format!("Failed to create project directory: {}", e)))?;
    
    // Create project structure
    create_project_structure(&project_dir, &name, &template)?;
    
    println!("✅ Project '{}' created successfully!", name);
    println!("📁 Project structure:");
    println!("   {}/", name);
    println!("   ├── package.json");
    println!("   ├── blueprints/");
    println!("   │   └── main.yaml");
    println!("   ├── handlers/");
    println!("   │   └── echo.js");
    println!("   └── README.md");
    println!();
    println!("🚀 Get started:");
    println!("   cd {}", name);
    println!("   backworks start");
    
    Ok(())
}

fn create_project_structure(project_dir: &PathBuf, name: &str, template: &str) -> Result<()> {
    // Create package.json
    let metadata = create_project_metadata(name, template);
    let metadata_path = project_dir.join("package.json");
    std::fs::write(&metadata_path, metadata)
        .map_err(|e| BackworksError::config(format!("Failed to write package.json: {}", e)))?;
    
    // Create blueprints directory
    let blueprints_dir = project_dir.join("blueprints");
    std::fs::create_dir_all(&blueprints_dir)
        .map_err(|e| BackworksError::config(format!("Failed to create blueprints directory: {}", e)))?;
    
    // Create main blueprint
    let main_blueprint = create_main_blueprint(name, template);
    let main_path = blueprints_dir.join("main.yaml");
    std::fs::write(&main_path, main_blueprint)
        .map_err(|e| BackworksError::config(format!("Failed to write main.yaml: {}", e)))?;
    
    // Create README.md
    let readme = create_readme(name, template);
    let readme_path = project_dir.join("README.md");
    std::fs::write(&readme_path, readme)
        .map_err(|e| BackworksError::config(format!("Failed to write README.md: {}", e)))?;
    
    // Create handlers directory and external handler
    let handlers_dir = project_dir.join("handlers");
    std::fs::create_dir_all(&handlers_dir)
        .map_err(|e| BackworksError::config(format!("Failed to create handlers directory: {}", e)))?;
    
    // Create external echo handler
    let echo_handler = create_echo_handler(name);
    let echo_path = handlers_dir.join("echo.js");
    std::fs::write(&echo_path, echo_handler)
        .map_err(|e| BackworksError::config(format!("Failed to write echo.js: {}", e)))?;
    
    Ok(())
}

fn create_project_metadata(name: &str, template: &str) -> String {
    match template {
        "api" => format!(r#"{{
  "name": "{}",
  "version": "1.0.0",
  "description": "A REST API built with Backworks",
  "main": "blueprints/main.yaml",
  "scripts": {{
    "dev": "backworks start --watch",
    "build": "backworks build --target production",
    "test": "backworks test"
  }},
  "dependencies": {{
    "backworks-auth": "^1.0.0",
    "backworks-postgresql": "^2.1.0"
  }},
  "backworks": {{
    "entrypoint": "blueprints/main.yaml",
    "server": {{
      "host": "0.0.0.0",
      "port": 3000
    }},
    "dashboard": {{
      "enabled": true,
      "port": 3001
    }},
    "plugins": {{
      "backworks-auth": {{
        "config": {{
          "secret": "${{JWT_SECRET}}",
          "expiry": "24h"
        }},
        "hooks": ["before_request"]
      }}
    }}
  }}
}}"#, name),
        "webapp" => format!(r#"{{
  "name": "{}",
  "version": "1.0.0",
  "description": "A web application built with Backworks",
  "main": "blueprints/main.yaml",
  "scripts": {{
    "dev": "backworks start --watch",
    "build": "backworks build --target production",
    "export": "backworks export --format static"
  }},
  "backworks": {{
    "entrypoint": "blueprints/main.yaml",
    "blueprints": {{
      "main": "blueprints/main.yaml",
      "endpoints": "blueprints/endpoints/",
      "ui": "blueprints/ui/"
    }},
    "server": {{
      "host": "0.0.0.0",
      "port": 3000
    }},
    "dashboard": {{
      "enabled": true,
      "port": 3001
    }}
  }}
}}"#, name),
        _ => format!(r#"{{
  "name": "{}",
  "version": "1.0.0",
  "description": "A simple API built with Backworks",
  "main": "blueprints/main.yaml",
  "scripts": {{
    "dev": "backworks start --watch",
    "build": "backworks build",
    "test": "backworks test"
  }},
  "backworks": {{
    "entrypoint": "blueprints/main.yaml",
    "server": {{
      "host": "0.0.0.0",
      "port": 3000
    }},
    "dashboard": {{
      "enabled": true,
      "port": 3001
    }}
  }}
}}"#, name)
    }
}

fn create_main_blueprint(name: &str, template: &str) -> String {
    match template {
        "api" => format!(r#"name: "{}"
description: "A REST API with authentication and database"

endpoints:
  health:
    path: "/health"
    methods: ["GET"]
    description: "Health check endpoint"
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {{
          return {{
            status: 200,
            body: {{ status: "ok", timestamp: new Date().toISOString() }}
          }};
        }}
  
  api_info:
    path: "/api/info"
    methods: ["GET"]
    description: "API information"
    middleware: ["auth"]
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {{
          return {{
            status: 200,
            body: {{ 
              name: "{}",
              version: "1.0.0",
              user: req.user
            }}
          }};
        }}
"#, name, name),
        "webapp" => format!(r#"name: "{}"
description: "A web application with API and UI"

includes:
  - "./endpoints/"
  - "./ui/"

globals:
  app_name: "{}"
  api_version: "v1"
"#, name, name),
        _ => format!(r#"name: "{}"
description: "A simple API demonstrating both inline and external handlers"

endpoints:
  hello:
    path: "/hello"
    methods: ["GET"]
    description: "Say hello to the world (inline handler)"
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {{
          return {{
            status: 200,
            body: {{ 
              message: "Hello from {}!",
              timestamp: new Date().toISOString()
            }}
          }};
        }}
  
  echo:
    path: "/echo"
    methods: ["POST"]
    description: "Echo back the request (external handler)"
    runtime:
      language: "javascript"
      handler: "./handlers/echo.js"
"#, name, name)
    }
}

fn create_readme(name: &str, template: &str) -> String {
    format!(r#"# {}

A {} built with Backworks.

## Quick Start

```bash
# Start the development server
backworks start

# Or with hot reload
backworks start --watch

# Test the API
curl http://localhost:3000/hello
```

## Development

```bash
# Validate configuration
backworks validate

# Build for production
backworks build --target production

# Run tests
backworks test
```

## API Documentation

Visit the built-in dashboard at http://localhost:3001 to explore the API interactively.

## Project Structure

- `backworks.json` - Project metadata and configuration
- `blueprints/` - API and application blueprints
- `blueprints/main.yaml` - Main application blueprint

## Deployment

```bash
# Build for production
backworks build --target production

# The built application will be in the target/ directory
```
"#, 
    name, 
    match template {
        "api" => "REST API",
        "webapp" => "web application", 
        _ => "API application"
    }
    )
}

async fn build_project(target: String, security: Option<String>, output: Option<PathBuf>) -> Result<()> {
    println!("🔨 Building project for target: {}", target);
    
    // Load project configuration
    let (metadata, config) = config::load_project_config(None)?;
    
    if let Some(ref metadata) = metadata {
        println!("✅ Project: {} v{}", metadata.name, metadata.version);
        
        // Check if target is enabled
        if let Some(target_config) = metadata.targets.get(&target) {
            if !target_config.enabled {
                println!("⚠️  Target '{}' is disabled in project configuration", target);
                return Ok(());
            }
        }
    }
    
    // Apply security profile if specified
    if let Some(security_profile) = security {
        println!("🔒 Applying security profile: {}", security_profile);
        // TODO: Implement security transformations
    }
    
    // Determine output directory
    let output_dir = output.unwrap_or_else(|| {
        metadata
            .as_ref()
            .and_then(|m| m.targets.get(&target))
            .and_then(|t| t.output.as_ref())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target").join(&target))
    });
    
    println!("📁 Output directory: {}", output_dir.display());
    
    // Create output directory
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| BackworksError::config(format!("Failed to create output directory: {}", e)))?;
    
    // TODO: Implement actual build process
    // For now, just copy the configuration
    let config_output = output_dir.join("config.json");
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| BackworksError::config(format!("Failed to serialize config: {}", e)))?;
    std::fs::write(&config_output, config_json)
        .map_err(|e| BackworksError::config(format!("Failed to write config: {}", e)))?;
    
    println!("✅ Build completed successfully!");
    println!("📦 Built files available in: {}", output_dir.display());
    
    Ok(())
}

async fn migrate_project(from: PathBuf, to: String) -> Result<()> {
    println!("🔄 Migrating from {} to project structure", from.display());
    
    // Load existing configuration
    let config = config::load_config(&from).await?;
    println!("✅ Loaded existing configuration: {}", config.name);
    
    // Create project metadata
    let project_name = config.name.clone().to_lowercase().replace(" ", "-");
    let metadata = format!(r#"{{
  "name": "{}",
  "version": "1.0.0",
  "description": "{}",
  "main": "blueprints/main.yaml",
  "scripts": {{
    "dev": "backworks start --watch",
    "build": "backworks build",
    "test": "backworks test"
  }},
  "backworks": {{
    "entrypoint": "blueprints/main.yaml",
    "server": {{
      "host": "{}",
      "port": {}
    }},
    "dashboard": {}
  }}
}}"#, 
        project_name,
        config.description.clone().unwrap_or_else(|| "Migrated from single file".to_string()),
        config.server.host,
        config.server.port,
        if config.dashboard.is_some() {
            serde_json::to_string(&config.dashboard).unwrap()
        } else {
            "null".to_string()
        }
    );
    
    // Create project structure
    std::fs::create_dir_all("blueprints")
        .map_err(|e| BackworksError::config(format!("Failed to create blueprints directory: {}", e)))?;
    
    // Write project metadata (default to package.json)
    let target_file = if to == "backworks.json" { "package.json" } else { &to };
    std::fs::write(target_file, metadata)
        .map_err(|e| BackworksError::config(format!("Failed to write {}: {}", target_file, e)))?;
    
    // Create main blueprint (simplified version of original)
    let main_blueprint = format!(r#"name: "{}"
description: "{}"

endpoints: {}
"#, 
        config.name,
        config.description.clone().unwrap_or_else(|| "Migrated configuration".to_string()),
        serde_yaml::to_string(&config.endpoints)
            .map_err(|e| BackworksError::config(format!("Failed to serialize endpoints: {}", e)))?
    );
    
    std::fs::write("blueprints/main.yaml", main_blueprint)
        .map_err(|e| BackworksError::config(format!("Failed to write main.yaml: {}", e)))?;
    
    println!("✅ Migration completed successfully!");
    println!("📁 Created:");
    println!("   {}", target_file);
    println!("   blueprints/main.yaml");
    println!();
    println!("🚀 Start the migrated project:");
    println!("   backworks start");
    
    Ok(())
}

async fn validate_config(config_path: Option<PathBuf>) -> Result<()> {
    println!("🔍 Validating configuration...");
    
    // Load configuration
    let (metadata, config) = config::load_project_config(config_path)?;
    
    if let Some(ref metadata) = metadata {
        println!("✅ Project metadata valid: {} v{}", metadata.name, metadata.version);
        
        // Validate project structure
        if !PathBuf::from(&metadata.entrypoint).exists() {
            println!("⚠️  Warning: Entrypoint file not found: {}", metadata.entrypoint);
        }
        
        // Validate blueprint references
        for (key, path) in &metadata.blueprints {
            let blueprint_path = PathBuf::from(path);
            if blueprint_path.is_dir() {
                if !blueprint_path.exists() {
                    println!("⚠️  Warning: Blueprint directory not found: {} ({})", key, path);
                }
            } else if !blueprint_path.exists() {
                println!("⚠️  Warning: Blueprint file not found: {} ({})", key, path);
            }
        }
    }
    
    // Validate blueprint configuration
    config::validate_config(&config)?;
    println!("✅ Configuration is valid!");
    
    Ok(())
}

// Add missing function stubs
fn init_logging(verbose: bool) {
    // Initialize basic logging for now
    if verbose {
        println!("🔍 Verbose logging enabled");
    }
}

async fn analyze_blueprint(config: Option<PathBuf>, format: Option<String>, output: Option<PathBuf>) -> Result<()> {
    println!("🔍 Analyzing blueprint configuration...");
    
    // Load configuration
    let (metadata, config) = config::load_project_config(config)?;
    
    if let Some(ref metadata) = metadata {
        println!("✅ Project: {} v{}", metadata.name, metadata.version);
    }
    
    println!("📊 Analysis Results:");
    println!("   Name: {}", config.name);
    println!("   Mode: {:?}", config.mode);
    println!("   Endpoints: {}", config.endpoints.len());
    
    for (name, endpoint) in &config.endpoints {
        println!("     - {} ({})", name, endpoint.path);
    }
    
    if !config.plugins.is_empty() {
        println!("   Plugins: {}", config.plugins.len());
        for (name, plugin_config) in &config.plugins {
            if plugin_config.enabled {
                println!("     - {} (enabled)", name);
            }
        }
    }
    
    if let Some(output_path) = output {
        println!("📝 Writing analysis to {}", output_path.display());
        // TODO: Implement analysis output
    }
    
    Ok(())
}

async fn start_capture_mode(port: u16, output: PathBuf, duration: Option<u64>) -> Result<()> {
    println!("📡 Starting capture mode on port {}...", port);
    println!("📝 Output will be saved to: {}", output.display());
    
    if let Some(d) = duration {
        println!("⏱️  Capturing for {} seconds", d);
    } else {
        println!("⏱️  Capturing indefinitely (press Ctrl+C to stop)");
    }
    
    // TODO: Implement actual capture functionality
    println!("⚠️  Capture mode not yet implemented");
    
    Ok(())
}

async fn generate_config(input: PathBuf, output: PathBuf) -> Result<()> {
    println!("🔧 Generating configuration from captured data...");
    println!("📥 Input: {}", input.display());
    println!("📤 Output: {}", output.display());
    
    // TODO: Implement config generation from captured data
    println!("⚠️  Config generation not yet implemented");
    
    Ok(())
}

fn create_echo_handler(name: &str) -> String {
    format!(r#"/** Echo Handler - External JavaScript Handler Example
 * 
 * This demonstrates how to use external JavaScript files for handlers
 * instead of inline code in the YAML blueprint.
 */

function handler(req, res) {{
  // Log the incoming request for demonstration
  console.log(`Echo endpoint called: ${{req.method}} ${{req.path}}`);
  
  // Echo back the request with additional metadata
  return {{
    status: 200,
    headers: {{ 
      "Content-Type": "application/json",
      "X-Handler-Type": "external-js"
    }},
    body: {{
      echo: req.body,
      metadata: {{
        method: req.method,
        path: req.path,
        headers: req.headers,
        timestamp: new Date().toISOString(),
        handler_source: "external-file",
        project: "{}"
      }}
    }}
  }};
}}

module.exports = handler;
"#, name)
}
