# 🔧 Backworks Developer Guide

Complete guide for developers to understand, contribute to, and extend Backworks.

## 🎯 Project Overview

Backworks is a **declarative backend platform** that transforms service schematics into working backend APIs with built-in monitoring. The project prioritizes simplicity, developer experience, and rapid prototyping.

### Core Philosophy
- **Configuration over Code** - Simple APIs need zero coding
- **Backend as Schema** - Your schematic IS your backend
- **Developer Joy** - From idea to working API in under 5 minutes

## 🏗️ Architecture Overview

```
┌─────────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Service Schematic   │───▶│ Backworks Engine│───▶│  HTTP API +     │
│   (blueprint.yaml)  │    │                 │    │   Dashboard     │
└─────────────────────┘    └─────────────────┘    └─────────────────┘
```

### Core Components

1. **Schematic Parser** (`src/config.rs`)
   - Parses and validates service schematics
   - Defines endpoint structure and validation rules

2. **Runtime Engine** (`src/runtime.rs`)
   - Executes JavaScript handlers
   - Manages request/response processing

3. **HTTP Server** (`src/server.rs`)
   - Handles HTTP requests and routing
   - Integrates with the runtime engine

4. **Dashboard** (`src/dashboard.rs`)
   - Provides real-time monitoring interface
   - Serves API metrics and logs

5. **Main Engine** (`src/engine.rs`)
   - Orchestrates all components
   - Manages system lifecycle

## 🚀 Getting Started for Development

### Prerequisites
- Rust 1.70+ (latest stable recommended)
- Git
- Basic understanding of HTTP APIs and YAML

### Initial Setup
```bash
# Clone the repository
git clone https://github.com/devstroop/backworks
cd backworks

# Build the project
cargo build

# Run tests
cargo test

# Run with example
cargo run -- start --config examples/hello-world/api.yaml
```

### Development Workflow
```bash
# Make changes to source code
# ...

# Build and test
cargo build
cargo test

# Test with examples
cargo run -- start --config examples/hello-world/api.yaml

# In another terminal, test the API
curl http://localhost:3002/hello
```

## 📁 Project Structure

```
backworks/
├── src/                    # Core Rust implementation
│   ├── main.rs            # CLI entry point and argument parsing
│   ├── lib.rs             # Library exports and public API
│   ├── engine.rs          # Core orchestration engine
│   ├── config.rs          # YAML configuration parsing
│   ├── runtime.rs         # JavaScript execution engine
│   ├── server.rs          # HTTP server implementation
│   ├── dashboard.rs       # Dashboard HTTP endpoints
│   ├── error.rs           # Error types and handling
│   ├── proxy.rs           # Future: Proxy mode implementation
│   ├── database.rs        # Future: Database integration
│   └── plugins/           # Future: Plugin system
│       ├── mod.rs         # Plugin module exports
│       └── ai.rs          # Future: AI plugin
├── examples/              # Example configurations
│   ├── hello-world/       # Basic example
│   ├── blog-api/          # Complex CRUD example
│   └── task-manager/      # Advanced business logic
├── docs/                  # Documentation
│   ├── quick-start.md     # Getting started guide
│   └── configuration.md   # Configuration reference
├── tests/                 # Integration tests
├── Cargo.toml            # Rust dependencies and metadata
└── README.md             # Project overview
```

## 🔧 Key Implementation Details

### Configuration System

**File:** `src/config.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackworksConfig {
    pub name: String,
    pub mode: ExecutionMode,
    pub endpoints: HashMap<String, EndpointConfig>,
    pub server: ServerConfig,
    pub dashboard: Option<DashboardConfig>,
    // ... other fields
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    Runtime,    // Execute JavaScript handlers (current)
    Database,   // Direct database operations (planned)
    Proxy,      // Proxy to other services (planned)
    Plugin,     // Custom plugins (planned)
}
```

**Key Functions:**
- `load_config()` - Parses YAML into configuration struct
- Validation occurs during deserialization
- Supports environment variable substitution (planned)

### Runtime Execution

**File:** `src/runtime.rs`

The runtime system executes JavaScript handlers for endpoints:

```rust
pub struct RuntimeEngine {
    // JavaScript execution context
}

impl RuntimeEngine {
    pub async fn execute_handler(
        &self,
        handler_code: &str,
        request: &HttpRequest
    ) -> Result<HttpResponse> {
        // Execute JavaScript handler
        // Return structured response
    }
}
```

**JavaScript Interface:**
- Handlers receive `req` object with method, path, headers, body
- Must return object with `status`, optional `headers` and `body`
- Currently synchronous only (async support planned)

### HTTP Server

**File:** `src/server.rs`

Built on Axum framework for high performance:

```rust
pub async fn start_server(config: BackworksConfig) -> Result<()> {
    let app = Router::new()
        .route("/*path", any(handle_request))
        .layer(cors_layer())
        .with_state(app_state);
    
    // Start API server and dashboard
}

async fn handle_request(
    State(state): State<AppState>,
    req: Request<Body>
) -> Result<Response<Body>, StatusCode> {
    // Route to appropriate endpoint
    // Execute handler
    // Return response
}
```

### Dashboard System

**File:** `src/dashboard.rs`

Provides real-time monitoring and API introspection:

- **Endpoint:** `/api/system` - System information
- **Endpoint:** `/api/metrics` - Request metrics
- **Endpoint:** `/api/performance` - Performance data
- **Frontend:** Simple HTML/CSS/JavaScript (no complex frameworks)

## 🧪 Testing Strategy

### Unit Tests
```bash
# Run all tests
cargo test

# Run specific test module
cargo test config

# Run with output
cargo test -- --nocapture
```

### Integration Tests
Located in `tests/` directory:
- `proxy_basic_test.rs` - Basic proxy functionality
- `capture_integration_tests.rs` - Capture mode testing
- `proxy_capture_integration_tests.rs` - Combined proxy/capture

### Manual Testing
```bash
# Test with examples
cargo run -- start --config examples/hello-world/api.yaml

# Test endpoints
curl http://localhost:3002/hello
curl -X POST http://localhost:3002/echo -d '{"test": "data"}' -H "Content-Type: application/json"

# Check dashboard
open http://localhost:3003
```

## 🎯 Current Implementation Status

### ✅ Completed Features
- [x] YAML configuration parsing
- [x] JavaScript runtime execution
- [x] HTTP server with routing
- [x] Built-in dashboard with metrics
- [x] Path parameter support (`/users/{id}`)
- [x] Multiple HTTP methods per endpoint
- [x] Request/response logging
- [x] Error handling and validation
- [x] CLI with multiple commands

### 🚧 In Progress
- [ ] Better error messages and validation
- [ ] Performance optimizations
- [ ] Enhanced dashboard features

### 📋 Planned Features
- [ ] Database integration (PostgreSQL, MySQL, SQLite)
- [ ] Proxy mode for existing APIs
- [ ] Plugin system for extensibility
- [ ] Multiple language support (Python, etc.)
- [ ] Async JavaScript handlers
- [ ] WebSocket support
- [ ] Authentication and authorization
- [ ] Rate limiting and security features

## 🤝 Contributing Guidelines

### Code Style
- Follow Rust standard formatting (`cargo fmt`)
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Write tests for new functionality

### Pull Request Process
1. **Fork and Clone** - Fork the repo and create feature branch
2. **Implement Changes** - Write code following project conventions
3. **Add Tests** - Ensure new functionality is tested
4. **Update Documentation** - Add/update relevant documentation
5. **Submit PR** - Create pull request with clear description

### Example Contribution Areas

#### Easy (Good First Issues)
- Add more example configurations
- Improve error messages
- Add validation for configuration fields
- Write additional tests

#### Medium
- Enhance dashboard with new metrics
- Implement new CLI commands
- Add configuration file templates
- Improve JavaScript runtime features

#### Advanced
- Implement database integration
- Add proxy mode functionality
- Design plugin system architecture
- Add WebSocket support

## 🐛 Debugging and Troubleshooting

### Common Issues

#### Configuration Parsing Errors
```bash
# Enable verbose logging
cargo run -- start --config api.yaml --verbose

# Validate configuration
cargo run -- validate --config api.yaml
```

#### JavaScript Handler Errors
- Check handler syntax (must be valid JavaScript)
- Ensure return object has required `status` field
- Verify request object usage (`req.method`, `req.body`, etc.)

#### Port Conflicts
```bash
# Check what's using the port
lsof -i :3000

# Use different ports
cargo run -- start --config api.yaml --port 8080 --dashboard-port 8081
```

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=debug cargo run -- start --config api.yaml

# Enable trace logging (very verbose)
RUST_LOG=trace cargo run -- start --config api.yaml
```

### Performance Profiling
```bash
# Build with release optimizations
cargo build --release

# Run with profiling
cargo run --release -- start --config api.yaml
```

## 📚 Development Resources

### Learning Resources
- [Rust Book](https://doc.rust-lang.org/book/) - Essential Rust learning
- [Axum Documentation](https://docs.rs/axum/) - Web framework used
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial) - Async runtime
- [Serde Guide](https://serde.rs/) - Serialization framework

### Useful Commands
```bash
# Check code formatting
cargo fmt --check

# Run clippy for linting
cargo clippy

# Check dependencies
cargo tree

# Update dependencies
cargo update

# Build documentation
cargo doc --open
```

### IDE Setup

#### VS Code Extensions
- `rust-analyzer` - Essential Rust language support
- `CodeLLDB` - Debugging support
- `crates` - Dependency management

#### Vim/Neovim
- Use `rust-analyzer` with LSP client
- Consider `rust.vim` for syntax highlighting

## 🚀 Future Roadmap

### Phase 1: Stability (Current)
- Robust configuration validation
- Better error handling and messages
- Performance optimizations
- Comprehensive testing

### Phase 2: Database Integration
- PostgreSQL, MySQL, SQLite support
- Auto-CRUD generation from schema
- Migration management
- Connection pooling

### Phase 3: Advanced Features
- Proxy mode for existing APIs
- Multiple language runtime support
- Plugin system architecture
- Authentication and authorization

### Phase 4: Production Ready
- Horizontal scaling support
- Advanced monitoring and alerting
- Load testing and optimization
- Deployment tooling

## 💡 Development Tips

### Best Practices
1. **Start Small** - Begin with simple changes and build up
2. **Test Early** - Write tests as you develop features
3. **Document Changes** - Update docs for any user-facing changes
4. **Follow Conventions** - Stick to established code patterns
5. **Ask Questions** - Open issues for clarification when needed

### Code Organization
- Keep functions focused and single-purpose
- Use appropriate error types and propagation
- Organize imports and use consistent naming
- Separate concerns between modules

### Performance Considerations
- Avoid unnecessary allocations in hot paths
- Use async/await appropriately
- Consider caching for frequently accessed data
- Profile before optimizing

---

This guide should help you understand Backworks architecture and contribute effectively. For questions, please open an issue or start a discussion!

**Happy coding! 🚀**
