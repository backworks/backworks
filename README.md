# 🚀 Backworks

**Declarative backend platform that transforms service schematics into working APIs.**

## 🎯 **What is Backworks?**

Backworks turns project-based service definitions into fully functional backend APIs with built-in monitoring.

**Project Structure → Working API + Dashboard**

```json
// backworks.json - Project metadata
{
  "name": "my-api",
  "version": "1.0.0",
  "entrypoint": "blueprints/main.yaml",
  "server": { "port": 3000 },
  "dashboard": { "enabled": true, "port": 3001 }
}
```

```yaml
# blueprints/main.yaml - API definition
name: "My API"
endpoints:
  users:
    path: "/users"  
    methods: ["GET"]
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          return {
            status: 200,
            body: { users: ['John', 'Jane'] }
          };
        }
```

```bash
# Get this working API
backworks start
curl http://localhost:3000/users
# → {"users": ["John", "Jane"]}
```

**Dashboard included:** `http://localhost:3001`

---

## ⚡ **Quick Start**

```bash
# 1. Clone and build
git clone https://github.com/devstroop/backworks
cd backworks
cargo build --release

# 2. Create a new project
./target/release/backworks init my-api
cd my-api

# 3. Start the API (uses backworks.json automatically)
../target/release/backworks start

# 4. Test the API
curl http://localhost:3000/hello

# 5. View dashboard
open http://localhost:3001
```

### **Try an Existing Example**
```bash
# Navigate to an example
cd examples/hello-world

# Start with project structure
../../target/release/backworks start

# Or use legacy single file (backward compatible)
../../target/release/backworks start --config blueprint.yaml
```

---

## 📋 **Core Features**

- **🎯 Project-Based Architecture** - Organized blueprints with metadata
- **⚡ Runtime Execution** - JavaScript handlers for business logic  
- **🔌 Plugin System** - Dependencies managed like npm packages
- **📊 Built-in Dashboard** - Real-time API monitoring and request logs
- **🚀 Zero Dependencies** - Single Rust binary with integrated web interface
- **🔄 Hot Reload** - Blueprint changes reflect immediately
- **🛡️ Security Profiles** - Target-specific compilation with secret management
- **📱 Multi-Target** - Web API, desktop app, mobile app from same blueprint

---

## 🎮 **Blueprint Templates**

| Template | Description | Complexity |
|---------|-------------|------------|
| [`hello-world`](./examples/hello-world/) | Simplest possible API | ⭐ |
| [`blog-api`](./examples/blog-api/) | Blog with posts & comments | ⭐⭐⭐ |
| [`task-manager`](./examples/task-manager/) | Complete business app | ⭐⭐⭐⭐ |

Each template shows the **Project Structure → API** transformation in action.

---

## 🏗️ **Architecture**

```
Project Structure (backworks.json + blueprints/) → Backworks Engine → HTTP API + Dashboard
```

- **Project-Based** - Organized blueprints with metadata (like npm/cargo projects)
- **Plugin Dependencies** - External capabilities via dependency management
- **Multi-File Blueprints** - Organized by feature (endpoints/, database.yaml, ui/)
- **Runtime Handlers** - JavaScript for custom business logic
- **Security Compilation** - Target-specific builds with secret management
- **Integrated Monitoring** - Dashboard shows real-time metrics and logs

**Current Implementation:** Project-based runtime with plugin system
**Backward Compatible:** Single blueprint.yaml files still supported

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
# New project-based approach
cd examples/hello-world
../../target/release/backworks start

# Legacy single file (backward compatible)
../../target/release/backworks start --config blueprint.yaml
```

---

## 🎯 **Use Cases**

### **Perfect For:**
- **API Prototyping** - Get working APIs instantly
- **Backend Mocking** - More than static responses  
- **Microservices** - Lightweight, focused backends
- **Dev/Test Environments** - Quick backend setup

### **Philosophy:**
> **"Projects over Files"** - Organized structure scales better  
> **"Metadata over Magic"** - Explicit configuration in backworks.json  
> **"Plugins over Frameworks"** - Extend via dependencies, not coupling  
> **"Developer Joy"** - Idea to API in under 5 minutes

---

## 🚀 **What's Next?**

**Current Status:** Project-based architecture with plugin system ✅  
**In Development:** Security compilation, multi-target builds  
**Future Roadmap:** UI framework integration, marketplace plugins, cloud deployment

**Goal:** Make backend development as organized and extensible as modern frontend development.

---

## 🤝 **Contributing**

1. Check out the [examples](./examples/) to understand the current capabilities
2. Read [ARCHITECTURE.md](./ARCHITECTURE.md) for design principles  
3. Read [DIRECTION.md](./DIRECTION.md) for current development direction
4. Start with documentation improvements or example additions
5. Core features welcome with discussion first

---

**Backworks: Because APIs should be this simple.**
