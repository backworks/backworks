# 🚀 Backworks

**Blueprint-agnostic platform that transforms functional workflows into native applications with platform-specific SDKs and implementations.**

## 🎯 **What is Backworks?**

Backworks transforms application workflows and functionalities into complete applications across any platform. You describe WHAT your application does (workflows, data flows, user interactions) - we provide the platform-specific SDKs and implementations for HOW it works on each target platform.

**Single Blueprint → Multi-Platform Applications**

```yaml
# Write this application blueprint (app.yaml)
name: "Task Manager"
version: "1.0.0"

# Core Application Logic
application:
  domain: "task_management"
  
  # Data Models
  models:
    task:
      id: "identifier"
      title: "text required"
      description: "text optional"
      status: "enum(pending,active,completed)"
      priority: "enum(low,medium,high)"
      created_at: "timestamp auto"
      updated_at: "timestamp auto"
  
  # Business Operations
  operations:
    list_tasks:
      input: { filters: "object optional" }
      output: { tasks: "array<task>" }
      logic: |
        // Generic business logic (language-agnostic)
        return storage.query('task', input.filters || {})
    
    create_task:
      input: { task: "task" }
      output: { task: "task" }
      logic: |
        task.id = generate_id()
        task.created_at = now()
        return storage.save('task', task)

# Target Platforms & Languages
targets:
  # Web Service (Current Implementation)
  web_service:
    language: "rust"
    runtime: "tokio"
    protocol: "http"
    port: 3000
    endpoints:
      - operation: "list_tasks"
        path: "/tasks"
        method: "GET"
      - operation: "create_task"
        path: "/tasks"
        method: "POST"
  
  # Desktop Application (Future)
  desktop_app:
    language: "rust"
    framework: "tauri"
    ui_library: "qwik"
    platform: ["windows", "macos", "linux"]
    interface:
      - operation: "list_tasks"
        component: "TaskList"
      - operation: "create_task"
        component: "TaskForm"
  
  # Mobile Application (Future)
  mobile_app:
    language: "dart"
    framework: "flutter"
    platform: ["ios", "android"]
    interface:
      - operation: "list_tasks"
        screen: "TaskListScreen"
      - operation: "create_task"
        screen: "CreateTaskScreen"
  
  # CLI Tool (Future)
  cli_tool:
    language: "go"
    framework: "cobra"
    commands:
      - operation: "list_tasks"
        command: "list"
        flags: ["--filter", "--format"]
      - operation: "create_task"
        command: "create"
        args: ["title", "description"]

# Infrastructure (Shared Across All Targets)
infrastructure:
  storage:
    type: "sqlite"
    connection: "./tasks.db"
    migrations: true
  
  config:
    environment_variables:
      - "DATABASE_URL"
      - "LOG_LEVEL"
```

```bash
# Generate all target applications
backworks generate --config app.yaml --target all

# Or generate specific targets
backworks generate --config app.yaml --target web_service
backworks generate --config app.yaml --target desktop_app
backworks generate --config app.yaml --target mobile_app
backworks generate --config app.yaml --target cli_tool

# Generated outputs:
# ./targets/web_service/    (Rust + Axum web service)
# ./targets/desktop_app/    (Rust + Tauri desktop app)
# ./targets/mobile_app/     (Flutter mobile app)
# ./targets/cli_tool/       (Go CLI application)
```

**Multi-Target Development** - Same business logic, different implementations for each platform

---

## ⚡ **Quick Start**

```bash
# 1. Clone and build
git clone https://github.com/devstroop/backworks
cd backworks
cargo build --release

# 2. Generate a complete application
cd examples/task-manager
../../target/release/backworks generate --config app.yaml --target web_service

# 3. Run the generated web service
cd targets/web_service
cargo run

# 4. Test your API
curl http://localhost:3000/tasks
```

### **Or try the current runtime mode:**
```bash
cd examples/hello-world
../../target/release/backworks start --config blueprint.yaml
curl http://localhost:3000/hello
```

---

## 📋 **Core Features**

### **🎯 Language-Agnostic Application Generation**
- **Universal Blueprints** - Define application logic once, generate anywhere
- **Multi-Platform Targets** - Web services, desktop apps, mobile apps, CLI tools
- **Code Generation** - Native code in Rust, Go, Dart, JavaScript, and more
- **Business Logic Reuse** - Same operations across all platforms

### **🚀 Current Backend Capabilities**  
- **Runtime Execution** - JavaScript handlers for business logic  
- **Database Integration** - SQL/NoSQL operations from YAML
- **Proxy Mode** - Forward to existing APIs with transformations
- **Plugin System** - Extensible architecture with hot-pluggable functionality

### **🔌 Plugin Ecosystem**
- **Authentication** - JWT, OAuth, API keys, session management
- **Caching** - Memory, Redis, distributed caching strategies
- **Rate Limiting** - Request throttling, DDoS protection
- **Analytics** - Metrics, logging, performance monitoring
- **Custom Plugins** - Build domain-specific extensions

### **🎨 Future Platform Support**
- **Desktop Applications** - Same blueprint → Native desktop apps
- **Mobile Applications** - Same blueprint → iOS/Android apps
- **CLI Tools** - Same blueprint → Command-line interfaces
- **Microservices** - Same blueprint → Distributed service mesh
- **WebAssembly** - Same blueprint → WASM applications
- **Theme System** - Customizable design systems

### **⚡ Developer Experience**
- **Zero Dependencies** - Single binary with integrated web interface
- **Hot Reload** - Changes reflect immediately across the stack
- **Visual Designer** - Drag-and-drop blueprint creation
- **Built-in Testing** - API and UI testing tools included

---

## 🎮 **Blueprint Templates**

| Template | Type | Description | Complexity |
|---------|------|-------------|------------|
| [`hello-world`](./examples/hello-world/) | Backend | Simplest possible API | ⭐ |
| [`blog-api`](./examples/blog-api/) | Backend | Blog with posts & comments | ⭐⭐⭐ |
| [`task-manager`](./examples/task-manager/) | Backend | Complete business API | ⭐⭐⭐⭐ |
| [`task-manager-fullstack`](./examples/task-manager-fullstack/) | Full-Stack | Complete task app with UI | ⭐⭐⭐⭐⭐ |
| [`e-commerce-platform`](./examples/e-commerce-platform/) | Full-Stack | Multi-service e-commerce | ⭐⭐⭐⭐⭐ |

### **🔥 Unified vs. Distributed Blueprints**

**Single File (Unified):**
```yaml
# app.yaml - Everything in one place
name: "My Application"
mode: "full-stack"
api: { endpoints: {...} }
interface: { pages: {...} }
infrastructure: { database: {...} }
```

**Multi-File (Distributed):**
```yaml
# app.yaml - Main orchestration
name: "My Application" 
mode: "full-stack"
includes:
  - "./api/endpoints.yaml"
  - "./interface/pages.yaml" 
  - "./infrastructure/config.yaml"
  - "./shared/schemas.yaml"

# api/endpoints.yaml
endpoints:
  resources:
    path: "/api/resources"
    methods: ["GET", "POST", "PUT", "DELETE"]
    # ... detailed endpoint config

# interface/pages.yaml
pages:
  dashboard:
    path: "/"
    components:
      - type: "data_table"
        # ... UI component config

# infrastructure/config.yaml
database:
  type: "postgresql"
  connection: "postgresql://user:pass@localhost/myapp"
storage:
  type: "s3"
  bucket: "myapp-uploads"
```

### **📁 Blueprint Organization**

**Single File (Simple Applications):**
```yaml
# app.yaml - Everything in one place
name: "Task Manager"
application:
  models: { ... }
  operations: { ... }
targets:
  web_service: { ... }
  desktop_app: { ... }
infrastructure: { ... }
```

**Distributed Files (Complex Applications):**
```yaml
# app.yaml - Main orchestration
name: "E-commerce Platform"
includes:
  - "./models/schemas.yaml"      # Data models
  - "./operations/business.yaml" # Business logic  
  - "./targets/platforms.yaml"   # Platform configs
  - "./infrastructure/config.yaml" # Infrastructure

# models/schemas.yaml
models:
  product:
    id: "identifier"
    name: "text required"
    price: "decimal required"
  user:
    id: "identifier"
    email: "email unique"

# operations/business.yaml  
operations:
  list_products:
    input: { category: "text optional" }
    output: { products: "array<product>" }
  create_order:
    input: { user_id: "identifier", items: "array" }
    output: { order: "order" }

# targets/platforms.yaml
targets:
  web_service:
    language: "rust"
    endpoints: [...]
  mobile_app:
    language: "dart"
    screens: [...]
```

---

## 🏗️ **Architecture**

```
Functional Blueprint → Backworks Engine → Platform-Specific Applications
                                       ├── Web Service (Backworks SDK for Rust)
                                       ├── Desktop App (Backworks SDK for Tauri)
                                       ├── Mobile App (Backworks SDK for Flutter)  
                                       ├── CLI Tool (Backworks SDK for Go)
                                       └── Future Platform SDKs...
```

### **🎯 Blueprint-Agnostic Philosophy**

**Functionality-First Design:**
- Describe WHAT your application does (workflows, data, interactions)
- Platform-agnostic business logic and data models
- UI/UX workflows defined by behavior, not implementation

**Platform-Specific SDKs:**
- Each platform gets a native Backworks SDK
- SDKs translate workflows into platform-optimal implementations
- No shared runtime - each app is fully native

**Workflow Translation Benefits:**
- **Native Experience** - Each platform feels native and optimized
- **Developer Freedom** - Choose best platform for each use case
- **Blueprint Reuse** - Same functionality across all platforms
- **SDK Evolution** - Platform SDKs evolve independently
- **Customization** - Modify generated code as needed

### **🧩 Core Principles**
- **Declarative-First** - Configuration defines everything
- **Component Reusability** - Share components across projects
- **Hot Reload** - Changes reflect across the entire stack
- **Plugin Architecture** - Extensible for any framework or service

---

## 📖 **Documentation**

- **[Quick Start Guide](./docs/quick-start.md)** - Get running in 5 minutes
- **[Schematic Reference](./docs/configuration.md)** - Complete YAML options  
- **[Examples Guide](./examples/README.md)** - Learn from examples
- **[Architecture Overview](./ARCHITECTURE.md)** - Design principles

---

## 🔧 **Installation**

### **Build from Source**
```bash
git clone https://github.com/backworks/backworks
cd backworks
cargo build --release

# The binary will be available at ./target/release/backworks
```

### **Run an Example**
```bash
./target/release/backworks start --config examples/hello-world/blueprint.yaml
```

---

## 🎯 **Use Cases**

### **Perfect For:**
- **API Prototyping** - Get working APIs instantly
- **Backend Mocking** - More than static responses  
- **Microservices** - Lightweight, focused backends
- **Dev/Test Environments** - Quick backend setup

### **Philosophy:**
> **"Configuration over Code"** - Simple APIs need zero coding  
> **"Backend as YAML"** - Your config IS your backend  
> **"Developer Joy"** - Idea to API in under 5 minutes

---

## 🚀 **What's Next?**

**Current Status:** Runtime mode with JavaScript handlers ✅  
**In Development:** Proxy mode, Database integration, Blueprint analyzer  
**Future Vision:** Multi-platform code generation, Language-agnostic development

### **🎯 Roadmap Phases**

**Phase 1: Robust Foundation (Current)**
- ✅ Runtime mode with JavaScript execution
- ✅ Proxy mode with transformations
- ✅ Blueprint analyzer with suggestions
- 🚧 Database mode completion
- 🚧 Studio modernization

**Phase 2: Code Generation Engine**
- 🔮 Language-agnostic blueprint parsing
- 🔮 Code generation templates (Rust, Go, Dart, etc.)
- 🔮 Platform-specific optimizations
- 🔮 Template marketplace for custom generators

**Phase 3: Multi-Platform Ecosystem**
- 🔮 Desktop application generation (Tauri, Electron)
- 🔮 Mobile application generation (Flutter, React Native)
- 🔮 CLI tool generation (Cobra, Clap, Click)
- 🔮 WebAssembly targets

**Phase 4: Universal Development Platform**
- 🔮 Visual blueprint designer
- 🔮 Real-time multi-platform preview
- 🔮 Deployment automation
- 🔮 Cross-platform testing framework

**Goal:** One blueprint, infinite applications - anywhere, any language, any platform.

---

## 🤝 **Contributing**

1. Check out the [examples](./examples/) to understand current capabilities
2. Read [ARCHITECTURE.md](./ARCHITECTURE.md) for design principles  
3. Read [IMPLEMENTATION_AUDIT.md](./IMPLEMENTATION_AUDIT.md) for current status
4. Start with documentation improvements or example additions
5. Multi-platform generation ideas and templates welcome!

---

**Backworks: Because application development should be this universal.**

### **🔧 Blueprint Compiler Approach**
The most practical approach: **Organized multi-file projects** with **target-specific compilation** and **built-in security**.

```yaml
# Main blueprint.yaml orchestrates everything
name: "E-commerce Platform"
includes:
  - "./endpoints/products.yaml"
  - "./plugins/security.yaml"
  - "./database/schemas.yaml"
  - "./ui/components.yaml"

# Compile for specific targets with security
targets:
  web_api:
    includes: ["endpoints/*", "plugins/security.yaml", "database/*"]
    excludes: ["ui/*"]
    security_profile: "production"
  
  mobile_app:
    includes: ["endpoints/products.yaml", "ui/mobile/*"]
    excludes: ["database/*", "plugins/admin.yaml"]
    security_profile: "mobile"
```

```bash
# Compile target-specific blueprints with security
backworks compile --config blueprint.yaml --target web_api --security production
backworks compile --config blueprint.yaml --target mobile_app --security mobile

# Deploy optimized, secure blueprints
backworks start --config ./compiled/web_api.yaml
backworks start --config ./compiled/mobile_app.yaml
```

**Compiler Benefits:**
- **Security by Design** - Strip secrets, add protection per target
- **Performance** - Only include what each platform needs
- **Organization** - Multi-file projects, single source of truth
- **Attack Surface Reduction** - Each target gets minimal required components
