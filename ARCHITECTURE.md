# 🚀 Backworks: Blueprint-Agnostic Platform

## 🎯 **Core Concept**

**Backworks provides a simple, extensible engine that transforms functional blueprints into working systems. Framework support (Next.js, React, Vue, etc.) and advanced features are added through external libraries that attach to the core engine.**

```yaml
# You write this simple blueprint (blueprint.yaml)
name: "Customer Portal"

# Core API Layer (Built-in)
endpoints:
  customers:
    path: "/api/customers"
    methods: ["GET", "POST", "PUT", "DELETE"]
    runtime:
      handler: "return customerService.process(req)"

# UI Layer (Via External Libraries)
interface:
  provider: "backworks-nextjs"  # External library
  pages:
    dashboard:
      path: "/"
      components:
        - type: "customer_table"
          data_source: "/api/customers"

# Advanced Features (Via External Libraries)  
services:
  notifications:
    provider: "backworks-notifications"  # External library
    triggers: ["customer_created", "customer_updated"]
```

**Backworks Core IS simple** - The engine handles blueprints, runtime, and plugins. Everything else is external libraries.

---

## 📋 **What Backworks Does**

### ✅ **Core Engine (Built-in)**
- **Blueprint Processing** - Parse and validate YAML configurations
- **Runtime Execution** - JavaScript handlers for business logic
- **Plugin System** - Extensible architecture for adding capabilities
- **HTTP Server** - Basic API endpoints and routing
- **Studio Dashboard** - Built-in monitoring and configuration

### � **External Libraries (Community-Driven)**
- **UI Frameworks** - backworks-nextjs, backworks-react, backworks-vue
- **Database Providers** - backworks-postgresql, backworks-mongodb
- **Deployment Tools** - backworks-docker, backworks-kubernetes
- **Preview Renderers** - backworks-storybook, backworks-figma

### 🚀 **Library Ecosystem Strategy**
- **Simple Core** - Backworks engine handles blueprints and runtime only
- **External Extension** - Framework support added via npm/cargo libraries
- **Community-Driven** - Developers create and share framework integrations
- **Plugin Architecture** - Libraries attach to engine through standardized hooks

---

## 🏗️ **Architecture Principles**

### 1. **Simple Core, Extensible Ecosystem**
- Core engine handles blueprints, runtime, and basic HTTP only
- All framework support comes from external libraries
- Libraries attach to engine through standardized plugin interfaces

### 2. **Library-Driven Extensibility**
- **UI Frameworks** - External libraries like `backworks-nextjs`, `backworks-react`
- **Database Providers** - External libraries like `backworks-postgresql`, `backworks-mongodb`  
- **Preview Tools** - External libraries like `backworks-storybook`, `backworks-figma`
- **Deployment Tools** - External libraries like `backworks-docker`, `backworks-kubernetes`

### 3. **Blueprint-Agnostic Engine**
- Same blueprint syntax regardless of attached libraries
- Libraries interpret blueprint sections relevant to their functionality
- Core engine coordinates between attached libraries

### 4. **Developer Freedom**
- Choose which libraries to attach based on project needs
- Create custom libraries for specific frameworks or tools
- Mix and match libraries without core engine changes

---

## 🎮 **Quick Start - Core + Libraries**

```bash
# 1. Install Backworks core
cargo install backworks

# 2. Install framework libraries
npm install backworks-nextjs
npm install backworks-postgresql

# 3. Create blueprint with library providers
cat > my-app.yaml << EOF
name: "Task Manager"

# Core Engine (Built-in)
server:
  port: 3000
endpoints:
  tasks:
    path: "/api/tasks"
    methods: ["GET", "POST"]
    runtime:
      handler: |
        function handler(req, res) {
          return { tasks: [...] }
        }

# External Library: Next.js UI
interface:
  provider: "backworks-nextjs"
  pages:
    dashboard:
      path: "/"
      components:
        - type: "TaskList"
          props: { source: "/api/tasks" }

# External Library: PostgreSQL Database        
database:
  provider: "backworks-postgresql"
  tables:
    tasks:
      id: "serial primary key"
      title: "varchar(255)"
EOF

# 4. Run with attached libraries
backworks start --config my-app.yaml

# 5. Access components
curl http://localhost:3000/api/tasks  # Core API
open http://localhost:3001            # Next.js UI (via library)
open http://localhost:3002            # Studio (built-in)
```

**Result: Core engine + attached libraries = Complete application!**

---

## 🎯 **Blueprint-Agnostic Integration Strategy**

### **Same Blueprint, Multiple Integrations**
```yaml
# Single blueprint.yaml
name: "E-commerce Platform"

# API Integration
endpoints:
  products: { path: "/api/products", methods: ["GET", "POST"] }
  orders: { path: "/api/orders", methods: ["GET", "POST"] }

# UI Integration  
interface:
  pages:
    shop: { path: "/shop", components: [...] }
    admin: { path: "/admin", components: [...] }

# Microservice Integration
services:
  inventory: { type: "background", handler: "inventoryService" }
  notifications: { type: "event_driven", triggers: [...] }
  
# Database Integration
database:
  tables:
    products: { schema: {...} }
    orders: { schema: {...} }

# Infrastructure Integration
infrastructure:
  cache: { type: "redis", ttl: 3600 }
  queue: { type: "memory", workers: 4 }
  monitoring: { enabled: true, metrics: [...] }
```

### **Platform-Specific Runtime Execution**
```
Same blueprint.yaml → Multiple Runtime Implementations

Web Platform:
├── HTTP Server (APIs)
├── Static File Server (UI)
├── Background Workers (Services)
├── Database Connections
└── Monitoring Endpoints

Desktop Platform:
├── Native HTTP Server
├── Embedded Web View (UI)
├── Background Threads (Services)
├── Local Database
└── System Tray Integration

Mobile Platform:
├── REST Client (APIs)
├── Native UI Components
├── Background Tasks (Services)
├── Local Storage
└── Push Notifications

CLI Platform:
├── HTTP Client (APIs)
├── Terminal UI (Commands)
├── Background Processes
├── Config Files
└── Shell Integration
```

### **Current Reality**: We Already Have This Foundation
Our current architecture is perfectly positioned for blueprint-agnostic execution:

```yaml
# blueprint.yaml (Same file across ALL platforms)
name: "Task API"
endpoints:
  tasks:
    path: "/tasks"
    methods: ["GET", "POST", "PUT", "DELETE"]
    runtime:
      handler: |
        function handler(req, res) {
          // Universal business logic
          const tasks = getTasks()
          return { tasks }
        }
```

### **Future Extension**: Platform-Specific Runtimes
```bash
# Same blueprint.yaml, different runtime targets
backworks run --target web       # Current: Rust + Axum server
backworks run --target desktop   # Future: Embedded runtime in Tauri app
backworks run --target mobile    # Future: Runtime in Flutter/React Native  
backworks run --target cli       # Future: Runtime in CLI wrapper
```

### **Key Insight**: Runtime Execution, Not Code Generation
- **Same blueprint.yaml** shared across all platforms
- **Platform-specific runtimes** interpret the blueprint
- **JavaScript handlers** execute the same way everywhere
- **No code generation** - pure runtime interpretation

---

## 📁 **Project Structure**

```
backworks/
├── src/                    # Core Rust implementation
│   ├── main.rs            # CLI entry point
│   ├── engine.rs          # Core engine
│   ├── config.rs          # YAML configuration
│   ├── runtime.rs         # JavaScript execution
│   ├── dashboard.rs       # Built-in dashboard
│   └── server.rs          # HTTP server
├── examples/              # Clean, focused examples
│   ├── hello-world/       # Simplest possible API
│   ├── blog-api/          # Complex blog backend
│   └── task-manager/      # Business application
└── docs/                  # Documentation
```

---

## 🎯 **Use Cases**

### **Perfect For:**
- **Rapid API prototyping** - Get working APIs in minutes
- **Backend-as-Config** - No coding required for simple APIs
- **API mocking with logic** - More than static responses
- **Microservice backends** - Lightweight, focused services
- **Dev/Test environments** - Quick backend setup

### **Not Designed For:**
- **High-performance production** (yet)
- **Complex database operations** (use runtime handlers)
- **Existing API proxying** (future feature)

---

## 🔧 **Configuration Reference**

### **Basic Structure**
```yaml
name: "API Name"
description: "What this API does"

server:
  host: "0.0.0.0"
  port: 3000

dashboard:
  enabled: true
  port: 3001

mode: "runtime"  # Current supported mode

endpoints:
  endpoint_name:
    path: "/path"
    methods: ["GET", "POST"]
    description: "What this endpoint does"
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          // Your logic here
          return {
            status: 200,
            headers: { "Content-Type": "application/json" },
            body: { result: "success" }
          };
        }
```

### **Handler Function Signature**
```javascript
function handler(req, res) {
  // req.method, req.path, req.body, req.headers available
  // Return: { status, headers?, body }
  return {
    status: 200,
    headers: { "Content-Type": "application/json" },
    body: { message: "Hello" }
  };
}
```

---

## 📊 **Dashboard Features**

- **Real-time API metrics** - Request counts, response times
- **Endpoint monitoring** - Track usage patterns
- **System status** - Server health, uptime
- **Live request logs** - See API calls as they happen

Access at: `http://localhost:3001` (configurable)

---

## 🚀 **Next Steps**

1. **Try the examples** - Start with `examples/hello-world/`
2. **Build your API** - Create YAML config for your use case
3. **Add logic** - Use JavaScript handlers for business logic
4. **Monitor usage** - Check dashboard for insights

---

## 🎯 **Philosophy**

> **"Configuration over Code"**  
> Simple APIs should require zero coding.  
> Complex logic should be optional, not required.

> **"Backend as YAML"**  
> Your YAML file IS your backend.  
> No compilation, no deployment complexity.

> **"Developer Joy"**  
> From idea to working API in under 5 minutes.  
> Real-time feedback and monitoring built-in.

---

## 🚀 **Evolution Path: From Workflows to Universal Platform**

**Backworks is evolving from workflow-driven backends to blueprint-agnostic universal platform** with minimal overhead and zero breaking changes.

**See [UNIVERSAL_ARCHITECTURE.md](./UNIVERSAL_ARCHITECTURE.md) for the complete technical vision.**

### **Current State → Future Vision**
```yaml
# Today: Workflow-driven backend (working perfectly)
endpoints:
  users:
    path: "/users"
    methods: ["GET", "POST"]
    description: "User management workflow"  # ← Already describing functionality
    runtime:
      handler: "return { users: [...] }"     # ← Already workflow logic
```

```yaml
# Future: Same workflow + Multi-platform targets (additive)
endpoints:
  users:
    path: "/users"
    methods: ["GET", "POST"] 
    description: "User management workflow"
    runtime:
      handler: "return { users: [...] }"

# New section: Platform-specific implementations
targets:
  web_service: { enabled: true }              # ← Current implementation
  desktop_app: { enabled: false }             # ← Future: Tauri desktop
  mobile_app: { enabled: false }              # ← Future: Flutter mobile
  cli_tool: { enabled: false }                # ← Future: Go CLI
```

**Same functionality description → Multiple native implementations**

### **Minimal Overhead Architecture**

1. **Current Engine** continues working exactly as it does
2. **Current Blueprints** remain 100% compatible
3. **Platform SDKs** added as separate generation layer
4. **Optional Targets** enable multi-platform generation when needed

---

## 🔄 **Blueprint Evolution (Minimal Overhead)**

### **Current Blueprint Structure (Already Workflow-Oriented)**
```yaml
# Current: hello-world/blueprint.yaml
name: "Hello World API"
description: "The simplest possible Backworks API"

server:
  host: "0.0.0.0"
  port: 3002

dashboard:
  enabled: true
  port: 3003

mode: "runtime"

endpoints:
  hello:
    path: "/hello"
    methods: ["GET"]
    description: "Say hello"  # ← Already describing WHAT it does
    runtime:
      language: "javascript"
      handler: |                # ← Already workflow logic
        function handler(req, res) {
          return {
            status: 200,
            body: { message: "Hello, World!" }
          };
        }
```

### **Future Extension (Same Blueprint + Platform Targets)**
```yaml
# Future: blueprint.yaml + platform targets
name: "Hello World Application"
description: "Multi-platform hello world example"

# Current section stays the same
server:
  host: "0.0.0.0"
  port: 3002

dashboard:
  enabled: true
  port: 3003

mode: "runtime"  # Current implementation

endpoints:
  hello:
    path: "/hello"
    methods: ["GET"]
    description: "Say hello"
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          return {
            status: 200,
            body: { message: "Hello, World!" }
          };
        }

# New section: Platform targets (future)
targets:
  web_service:
    enabled: true    # Uses current implementation
    
  desktop_app:
    enabled: false   # Future: Generate Tauri app
    language: "rust"
    framework: "tauri"
    
  mobile_app:
    enabled: false   # Future: Generate Flutter app
    language: "dart"
    framework: "flutter"
    
  cli_tool:
    enabled: false   # Future: Generate Go CLI
    language: "go"
    framework: "cobra"
```

### **Evolution Strategy: Zero Breaking Changes**

1. **Current blueprints continue working exactly as they do now**
2. **Add optional `targets` section for future multi-platform generation**
3. **Default target is always `web_service` (current behavior)**
4. **Future targets are opt-in and additive**

---

## 🔌 **Extensible Plugin Architecture**

The blueprint-agnostic platform remains fully extensible through our robust plugin system:

```yaml
# blueprint.yaml - Plugin integration
name: "E-commerce Platform"

# Core application functionality
endpoints:
  products:
    path: "/products"
    methods: ["GET", "POST", "PUT", "DELETE"]
    runtime:
      handler: "productService.handler"

# Plugin ecosystem extends capabilities
plugins:
  # Authentication plugin
  auth:
    enabled: true
    type: "jwt"
    config:
      secret: "${JWT_SECRET}"
      expiry: "24h"
    hooks: ["before_request"]
  
  # Rate limiting plugin
  rate_limiter:
    enabled: true
    type: "redis"
    config:
      max_requests: 100
      window: "1h"
    hooks: ["before_request"]
  
  # Caching plugin
  cache:
    enabled: true
    type: "memory"
    config:
      ttl: 300
      max_size: "100MB"
    hooks: ["after_response"]
  
  # Analytics plugin
  analytics:
    enabled: true
    type: "metrics"
    config:
      endpoint: "http://analytics.internal"
    hooks: ["after_response"]
  
  # Custom business plugin
  inventory:
    enabled: true
    type: "custom"
    path: "./plugins/inventory.js"
    config:
      warehouse_api: "http://warehouse.internal"
    hooks: ["before_request", "after_response"]
```

**Plugin Benefits:**
- **Same plugins work across ALL platforms** (web, desktop, mobile, CLI)
- **Runtime-agnostic** - No code generation needed
- **Hot-pluggable** - Enable/disable without restarts
- **Composable** - Mix and match functionality
- **Community-driven** - Shared plugin ecosystem

**Plugin Hooks Available:**
- `before_request` - Authentication, validation, rate limiting
- `after_response` - Caching, analytics, logging
- `on_error` - Error handling, notifications
- `on_startup` - Initialization, health checks
- `on_shutdown` - Cleanup, graceful shutdown

---

## 🚀 **Plugin-Enhanced Runtime Strategy**

Our plugin system seamlessly integrates with the blueprint-agnostic runtime:

```rust
// Platform-specific runtime with shared plugin system
pub struct BackworksRuntime {
    config: Blueprint,
    plugin_manager: PluginManager,
    platform_adapter: Box<dyn PlatformAdapter>,
}

impl BackworksRuntime {
    // Same runtime core across ALL platforms
    pub async fn execute_endpoint(&self, request: Request) -> Response {
        // 1. Plugin hooks - before_request
        self.plugin_manager.before_request(&mut request).await?;
        
        // 2. Execute blueprint logic (platform-agnostic)
        let response = self.execute_handler(&request).await?;
        
        // 3. Plugin hooks - after_response
        self.plugin_manager.after_response(&mut response).await?;
        
        // 4. Platform-specific adaptation
        self.platform_adapter.adapt_response(response)
    }
}
```

**Platform Adapters with Plugin Support:**
- **Web Server** - HTTP requests/responses + web-specific plugins
- **Desktop App** - UI events/actions + desktop-specific plugins  
- **Mobile App** - Touch/gestures + mobile-specific plugins
- **CLI Tool** - Commands/args + CLI-specific plugins
- **Microservice** - Service mesh + distributed plugins

**Plugin Categories:**
- **Core Plugins** - Auth, caching, logging, metrics (universal)
- **Platform Plugins** - UI components, native integrations
- **Business Plugins** - Domain-specific logic, external APIs
- **Infrastructure Plugins** - Databases, message queues, storage

---

## 🔧 **Blueprint Compiler Strategy**

Instead of sharing the entire blueprint across all targets, we use a **blueprint compiler** that trims and optimizes blueprints for specific deployment targets with built-in security.

```yaml
# main blueprint.yaml (Full specification)
name: "E-commerce Platform"
version: "1.0.0"

# Multiple organized YAML files in the same project
includes:
  - "./endpoints/products.yaml"
  - "./endpoints/orders.yaml"
  - "./endpoints/customers.yaml"
  - "./plugins/security.yaml"
  - "./plugins/business.yaml"
  - "./database/schemas.yaml"
  - "./ui/components.yaml"
  - "./infrastructure/config.yaml"

# Compilation targets with specific requirements
targets:
  web_api:
    compile_profile: "server"
    includes: ["endpoints/*", "plugins/security.yaml", "database/*"]
    excludes: ["ui/*", "plugins/desktop.yaml"]
    security:
      strip_secrets: true
      obfuscate_internals: true
      enable_rate_limiting: true
  
  desktop_app:
    compile_profile: "client"
    includes: ["endpoints/products.yaml", "ui/*", "plugins/auth.yaml"]
    excludes: ["database/*", "plugins/admin.yaml"]
    security:
      local_auth_only: true
      encrypt_config: true
      sandbox_plugins: true
  
  mobile_app:
    compile_profile: "mobile"
    includes: ["endpoints/products.yaml", "endpoints/orders.yaml", "ui/mobile/*"]
    excludes: ["database/*", "plugins/server.yaml", "ui/desktop/*"]
    security:
      api_key_auth: true
      certificate_pinning: true
      runtime_protection: true
  
  admin_panel:
    compile_profile: "admin"
    includes: ["endpoints/*", "plugins/*", "database/*", "ui/admin/*"]
    excludes: ["ui/mobile/*", "ui/desktop/*"]
    security:
      admin_auth_required: true
      audit_logging: true
      ip_whitelist: true
```

### **🛡️ Security-Enhanced Compilation**

The blueprint compiler introduces security at compilation time, not runtime:

```bash
# Compile for different targets with security profiles
backworks compile --config blueprint.yaml --target web_api --security production
backworks compile --config blueprint.yaml --target desktop_app --security client
backworks compile --config blueprint.yaml --target mobile_app --security mobile
backworks compile --config blueprint.yaml --target admin_panel --security admin

# Generated target-specific blueprints:
# ./compiled/web_api.yaml      - Server endpoints only, secrets stripped
# ./compiled/desktop_app.yaml  - Client-safe config, encrypted secrets
# ./compiled/mobile_app.yaml   - Mobile-optimized, API keys only
# ./compiled/admin_panel.yaml  - Full access, audit logging enabled
```

**Security Compilation Features:**

```rust
// Blueprint compiler with security profiles
pub struct BlueprintCompiler {
    source_blueprint: Blueprint,
    security_profiles: HashMap<String, SecurityProfile>,
}

impl BlueprintCompiler {
    pub fn compile(&self, target: &str, security_level: &str) -> CompiledBlueprint {
        let mut compiled = Blueprint::new();
        let profile = self.security_profiles.get(security_level)?;
        
        // 1. Filter components based on target requirements
        compiled.endpoints = self.filter_endpoints_for_target(target);
        compiled.plugins = self.filter_plugins_for_target(target);
        compiled.database = self.filter_database_for_target(target);
        
        // 2. Apply security transformations
        self.apply_security_profile(&mut compiled, profile);
        
        // 3. Optimize for target platform
        self.optimize_for_platform(&mut compiled, target);
        
        compiled
    }
    
    fn apply_security_profile(&self, blueprint: &mut Blueprint, profile: &SecurityProfile) {
        // Strip sensitive data
        if profile.strip_secrets {
            blueprint.remove_database_credentials();
            blueprint.remove_api_keys();
            blueprint.remove_internal_endpoints();
        }
        
        // Inject security plugins
        if profile.enable_rate_limiting {
            blueprint.plugins.insert("rate_limiter", default_rate_limiter());
        }
        
        // Add authentication requirements
        if profile.require_auth {
            blueprint.add_auth_middleware_to_all_endpoints();
        }
        
        // Obfuscate internal details
        if profile.obfuscate_internals {
            blueprint.rename_internal_identifiers();
            blueprint.remove_debug_info();
        }
    }
}
```

**Security Profiles:**

```yaml
# security/profiles.yaml
profiles:
  production:
    strip_secrets: true
    obfuscate_internals: true
    enable_rate_limiting: true
    require_https: true
    audit_logging: true
    
  client:
    local_auth_only: true
    encrypt_local_config: true
    sandbox_plugins: true
    disable_admin_endpoints: true
    
  mobile:
    api_key_auth: true
    certificate_pinning: true
    runtime_protection: true
    minimal_surface: true
    
  development:
    strip_secrets: false
    enable_debug: true
    disable_rate_limiting: true
    verbose_logging: true
```

---

## 🔌 **External Library Architecture**

### **Library Attachment Strategy**

External libraries attach to the Backworks engine to provide framework-specific functionality:

```bash
# Install framework libraries
npm install backworks-nextjs
npm install backworks-postgresql
npm install backworks-storybook

# Or with Cargo
cargo add backworks-nextjs
cargo add backworks-postgresql
```

```yaml
# blueprint.yaml - Libraries provide additional capabilities
name: "E-commerce Platform"

# Core Engine (Built-in)
endpoints:
  products:
    path: "/api/products"
    methods: ["GET", "POST", "PUT", "DELETE"]
    runtime:
      handler: "productService.handler"

# External Library: Next.js Integration
interface:
  provider: "backworks-nextjs"
  config:
    version: "14"
    typescript: true
    tailwind: true
  pages:
    shop:
      path: "/shop"
      components:
        - type: "ProductGrid"
          props: { source: "/api/products" }

# External Library: PostgreSQL Integration  
database:
  provider: "backworks-postgresql"
  config:
    connection_url: "${DATABASE_URL}"
    migrations: true
  tables:
    products:
      id: "serial primary key"
      name: "varchar(255)"

# External Library: Storybook Preview
preview:
  provider: "backworks-storybook"
  config:
    port: 6006
    components: ["ProductGrid", "ProductForm"]
```

### **Library Development Kit**

```rust
// backworks-nextjs/lib.rs - Example external library
use backworks_core::{Library, Blueprint, Engine, EngineHooks};

pub struct NextJSLibrary {
    config: NextJSConfig,
}

impl Library for NextJSLibrary {
    fn name(&self) -> &str { "backworks-nextjs" }
    
    fn attach(&self, engine: &mut Engine) -> Result<(), Error> {
        // Register hooks for interface sections
        engine.register_hook("interface", Box::new(self.clone()))?;
        Ok(())
    }
    
    fn process_blueprint_section(&self, section: &str, config: &Value) -> Result<(), Error> {
        match section {
            "interface" => self.generate_nextjs_app(config),
            _ => Ok(())
        }
    }
    
    fn generate_nextjs_app(&self, config: &Value) -> Result<(), Error> {
        // Generate Next.js pages, components, API routes
        // based on blueprint configuration
        Ok(())
    }
}
```

### **Available Libraries (Community Ecosystem)**

| Library | Purpose | Provider | Blueprint Section |
|---------|---------|----------|-------------------|
| `backworks-nextjs` | Next.js frontend | Community | `interface` |
| `backworks-react` | React SPA | Community | `interface` |
| `backworks-vue` | Vue.js frontend | Community | `interface` |
| `backworks-svelte` | Svelte frontend | Community | `interface` |
| `backworks-postgresql` | PostgreSQL database | Community | `database` |
| `backworks-mongodb` | MongoDB database | Community | `database` |
| `backworks-redis` | Redis caching | Community | `cache` |
| `backworks-docker` | Docker deployment | Community | `deployment` |
| `backworks-kubernetes` | K8s deployment | Community | `deployment` |
| `backworks-storybook` | Component preview | Community | `preview` |
| `backworks-figma` | Design sync | Community | `design` |

### **Benefits of Library Architecture**

1. **Simple Core** - Backworks engine stays focused and lightweight
2. **Framework Freedom** - Choose any frontend framework via libraries
3. **Community-Driven** - Developers create and maintain framework integrations
4. **No Vendor Lock-in** - Switch libraries without changing blueprints
5. **Extensible** - Add support for any tool/framework via libraries

---

### **🎨 Built-in Declarative UI Support**

Beyond external libraries, Backworks core includes **minimal-overhead declarative UI** using the same plugin architecture - just add an export method:

```yaml
# Standard blueprint with built-in UI capabilities
name: "Task Manager"

# API Layer (Core functionality)
endpoints:
  tasks:
    path: "/api/tasks"
    methods: ["GET", "POST", "PUT", "DELETE"]
    runtime:
      handler: "return taskService.process(req)"

# UI Layer (Built-in declarative UI)
interface:
  framework: "builtin"  # Uses core UI plugin
  export: "static"      # Export method: static, spa, ssr
  
  pages:
    dashboard:
      path: "/"
      layout: "grid"
      components:
        - type: "data_table"
          title: "Tasks"
          source: "/api/tasks"
          columns:
            - field: "title"
              type: "text"
              sortable: true
            - field: "status"
              type: "badge"
              filterable: true
          actions: ["create", "edit", "delete"]
        
        - type: "form_panel"
          title: "Add Task"
          target: "/api/tasks"
          method: "POST"
          fields:
            - name: "title"
              type: "text"
              required: true
            - name: "description"
              type: "textarea"
            - name: "status"
              type: "select"
              options: ["pending", "active", "completed"]

# Plugin architecture extends both API and UI
plugins:
  auth:
    enabled: true
    config:
      provider: "jwt"
    # Plugin works for both API endpoints and UI components
    affects: ["endpoints", "interface"]
  
  theme:
    enabled: true
    config:
      style: "modern"
      colors: "blue"
    # UI-specific plugin using same architecture
    affects: ["interface"]
```

**Built-in UI Plugin Architecture:**
```rust
// Core UI plugin using same plugin system
pub struct BuiltinUIPlugin {
    renderer: ComponentRenderer,
    exporter: StaticExporter,
}

impl BackworksPlugin for BuiltinUIPlugin {
    fn name(&self) -> &str { "builtin_ui" }
    
    // Uses same plugin hooks as other plugins
    async fn process_endpoint(&self, endpoint: &str, request: &Request) -> BackworksResult<Option<Response>> {
        if endpoint.starts_with("/ui/") {
            // Render UI components using blueprint definitions
            let component = self.renderer.render_from_blueprint(endpoint, &self.blueprint)?;
            return Ok(Some(Response::html(component)));
        }
        Ok(None)
    }
    
    // Export method for static generation
    pub async fn export_static(&self, output_dir: &str) -> BackworksResult<()> {
        self.exporter.generate_static_site(output_dir, &self.blueprint).await
    }
}
```

**Export Methods Available:**
```bash
# Static site generation
backworks export --config blueprint.yaml --format static --output ./dist

# Single Page Application 
backworks export --config blueprint.yaml --format spa --output ./dist

# Server-Side Rendered
backworks export --config blueprint.yaml --format ssr --output ./dist

# Progressive Web App
backworks export --config blueprint.yaml --format pwa --output ./dist

# Component Library (for external integration)
backworks export --config blueprint.yaml --format components --output ./components
```

**Minimal Overhead Benefits:**
- **Same Plugin System** - UI uses existing plugin architecture
- **No Additional Runtime** - Built into core engine
- **Standard Components** - Pre-built, optimized UI components
- **Export Flexibility** - Multiple output formats from same blueprint
- **Progressive Enhancement** - Start simple, add external libraries later
