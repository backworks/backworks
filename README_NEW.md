# 🚀 Backworks

**Configuration-driven API platform that works backwards from your needs.**

## 🎯 **What is Backworks?**

Backworks transforms YAML configuration into working backend APIs.

**YAML → Working API**

```yaml
# Write this YAML configuration
name: "My API"
mode: "runtime"
endpoints:
  users:
    path: "/users"  
    methods: ["GET"]
    runtime:
      handler: "return { users: ['John', 'Jane'] }"
```

```bash
# Get this working API
backworks start --config api.yaml
curl http://localhost:3000/users
# → {"users": ["John", "Jane"]}
```

**Dashboard included:** `http://localhost:3001`

---

## ⚡ **Quick Start**

```bash
# 1. Try an example
cd examples/hello-world
backworks start --config api.yaml

# 2. Test the API
curl http://localhost:3000/hello

# 3. View dashboard
open http://localhost:3001
```

---

## 📋 **Core Features**

- **🎯 YAML-Driven** - Configuration becomes your backend
- **⚡ Runtime Execution** - JavaScript handlers for business logic  
- **📊 Built-in Dashboard** - Real-time API monitoring
- **🚀 Zero Dependencies** - One binary, runs anywhere
- **🔄 Hot Reload** - Changes reflect immediately

---

## 🎮 **Examples**

| Example | Description | Complexity |
|---------|-------------|------------|
| [`hello-world`](./examples/hello-world/) | Simplest possible API | ⭐ |
| [`blog-api`](./examples/blog-api/) | Blog with posts & comments | ⭐⭐⭐ |
| [`task-manager`](./examples/task-manager/) | Complete business app | ⭐⭐⭐⭐ |

Each example shows the **YAML → API** transformation in action.

---

## 🏗️ **Architecture**

```
YAML Config → Backworks Engine → HTTP API + Dashboard
```

- **Configuration-First** - Your YAML defines everything
- **Runtime Handlers** - JavaScript for custom logic
- **Integrated Monitoring** - Dashboard shows real-time metrics
- **Simple Deployment** - One process, two ports (API + Dashboard)

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed design principles.

---

## 📖 **Documentation**

- **[Quick Start Guide](./docs/quick-start.md)** - Get running in 5 minutes
- **[Configuration Reference](./docs/configuration.md)** - Complete YAML options  
- **[Examples Guide](./examples/README.md)** - Learn from examples
- **[Architecture Overview](./ARCHITECTURE.md)** - Design principles

---

## 🔧 **Installation**

```bash
# Build from source
git clone https://github.com/yourusername/backworks
cd backworks
cargo build --release

# Run an example
./target/release/backworks start --config examples/hello-world/api.yaml
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

**Current:** YAML → Runtime API  
**Future:** Database integration, Proxy mode, Plugin system

**Goal:** Make backend development as simple as writing configuration.

---

## 🤝 **Contributing**

1. Check out the [examples](./examples/) to understand the concept
2. Read [ARCHITECTURE.md](./ARCHITECTURE.md) for design principles  
3. Start with documentation improvements or example additions
4. Core features welcome with discussion first

---

**Backworks: Because APIs should be this simple.**
