# 🚀 Backworks

**Declarative backend platform that transforms service schematics into working APIs.**

## 🎯 **What is Backworks?**

Backworks turns simple service schematics into fully functional backend APIs with built-in monitoring.

**Schematic → Working API + Dashboard**

```yaml
# Write this service schematic (blueprint.yaml)
name: "My API"
mode: "runtime"
server:
  port: 3000
dashboard:
  enabled: true
  port: 3001
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
backworks start --config blueprint.yaml
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

# 2. Build the Studio (web interface)
cd studio
npm install
npm run build
cd ..

# 3. Try an example schematic
cd examples/hello-world
../../target/release/backworks start --config blueprint.yaml

# 4. Test the API
curl http://localhost:3002/hello

# 5. View Studio dashboard
open http://localhost:3003
```

---

## 📋 **Core Features**

- **🎯 Declarative Design** - Service schematics become your backend
- **⚡ Runtime Execution** - JavaScript handlers for business logic  
- **🎨 Studio Interface** - Visual blueprint designer and API testing tools
- **📊 Built-in Dashboard** - Real-time API monitoring and request logs
- **🚀 Zero Dependencies** - Single Rust binary with integrated web interface
- **🔄 Hot Reload** - Blueprint changes reflect immediately
- **🛡️ Error Handling** - Robust error handling and status reporting

---

## 🎮 **Blueprint Templates**

| Template | Description | Complexity |
|---------|-------------|------------|
| [`hello-world`](./examples/hello-world/) | Simplest possible API | ⭐ |
| [`blog-api`](./examples/blog-api/) | Blog with posts & comments | ⭐⭐⭐ |
| [`task-manager`](./examples/task-manager/) | Complete business app | ⭐⭐⭐⭐ |

Each template shows the **Service Schematic → API** transformation in action.

---

## 🏗️ **Architecture**

```
Service Schematic (Blueprint) → Backworks Engine → HTTP API + Dashboard
```

- **Declarative-First** - Your service design defines everything
- **Runtime Handlers** - JavaScript for custom business logic
- **Integrated Monitoring** - Dashboard shows real-time metrics and logs
- **Simple Deployment** - One process, two ports (API + Dashboard)
- **Plugin Architecture** - Extensible design for future enhancements

**Current Implementation:** Runtime mode with JavaScript execution
**Planned Features:** Database integration, Proxy mode, Plugin system

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
**In Development:** Configuration validation, better error handling  
**Future Roadmap:** Database integration, Proxy mode, Plugin system

**Goal:** Make backend development as simple as writing configuration.

---

## 🤝 **Contributing**

1. Check out the [examples](./examples/) to understand the current capabilities
2. Read [ARCHITECTURE.md](./ARCHITECTURE.md) for design principles  
3. Read [DIRECTION.md](./DIRECTION.md) for current development direction
4. Start with documentation improvements or example additions
5. Core features welcome with discussion first

---

**Backworks: Because APIs should be this simple.**
