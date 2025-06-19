# 🎯 Backworks: Definitive Architecture & Direction

**Created: June 19, 2025**  
**Status: ACTIVE - This is the current direction**

---

## 📋 **What Backworks IS (Current)**

### **Core Identity**
- **YAML-Driven Backend Creator** - Transform configuration into working APIs
- **Not a Proxy** - Backworks IS your backend
- **Integrated Solution** - API + Dashboard in one process
- **Developer-First** - Simple, fast, joyful experience

### **How It Works**
```
1. Write YAML config → 2. Run backworks → 3. Get working API + Dashboard
```

### **Current Features**
- ✅ YAML configuration parsing
- ✅ Runtime mode with JavaScript handlers  
- ✅ HTTP API server
- ✅ Built-in dashboard (simplified HTML/JS)
- ✅ Real-time metrics
- ✅ Multiple example patterns

---

## 🚫 **What Backworks is NOT (Current)**

- ❌ **Proxy server** (that's future phase)
- ❌ **API gateway** (not the focus)
- ❌ **Complex framework** (simplicity first)
- ❌ **Production-scale system** (development/prototyping focus)

---

## 🗂️ **File Structure (Current)**

```
backworks/
├── README.md              # ✅ Simple, clear intro
├── ARCHITECTURE.md        # ✅ Detailed design principles  
├── DIRECTION.md           # ✅ Current direction (this file)
├── PRIORITIES.md          # ✅ Development priorities
├── ISSUES.md              # ✅ Issue tracking
├── src/                   # ✅ Core Rust implementation
│   ├── main.rs           # CLI entry point
│   ├── engine.rs         # Core orchestration
│   ├── config.rs         # YAML parsing
│   ├── runtime.rs        # JavaScript execution
│   ├── dashboard.rs      # Simple HTML dashboard
│   └── server.rs         # HTTP server
├── examples/              # ✅ Clean, focused examples
│   ├── README.md         # Example overview
│   ├── hello-world/      # Simplest API
│   ├── blog-api/         # Complex example
│   └── task-manager/     # Business app
├── docs/                  # ✅ Minimal documentation
│   ├── README.md         # Documentation overview
│   ├── quick-start.md    # Get running in 5 minutes
│   └── configuration.md  # YAML reference
├── tests/                 # ✅ Integration tests
└── archived-docs/         # 🗂️ All legacy content
```

---

## 🎯 **Execution Modes (Current)**

### **Active Mode**
- **`runtime`** - Execute JavaScript handlers for endpoints

### **Future Modes** (Not Implemented)
- `database` - Direct database operations
- `proxy` - Capture existing APIs  
- `plugin` - Custom extensions

---

## 📝 **Configuration Format (Current)**

```yaml
name: "API Name"
description: "What this does"

server:
  host: "0.0.0.0" 
  port: 3000

dashboard:
  enabled: true
  port: 3001

mode: "runtime"  # Only supported mode currently

endpoints:
  endpoint_name:
    path: "/path"
    methods: ["GET", "POST"]
    description: "What this does"
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          return {
            status: 200,
            headers: { "Content-Type": "application/json" },
            body: { message: "Hello" }
          };
        }
```

---

## 🚀 **Commands (Current)**

```bash
# Build
cargo build --release

# Run with config
./target/release/backworks start --config path/to/config.yaml

# Test endpoints
curl http://localhost:3000/endpoint

# View dashboard  
open http://localhost:3001
```

---

## 📊 **Dashboard (Current)**

- **Simple HTML/JS** (no Leptos/WASM complexity)
- **Real-time metrics** via API endpoints
- **Built-in to main process** (not separate server)
- **Developer-focused** (not production monitoring)

### Dashboard APIs:
- `GET /api/system` - System info
- `GET /api/metrics` - Endpoint metrics
- `GET /api/performance` - Performance data

---

## 🎯 **Philosophy & Principles**

1. **Configuration over Code** - Simple APIs need zero coding
2. **Backend as YAML** - Your config IS your backend  
3. **Developer Joy** - Idea to working API in under 5 minutes
4. **Simplicity First** - No unnecessary complexity
5. **Integrated Experience** - Everything works together

---

## 🚀 **Future Direction (Next Phases)**

### **Phase 2: Database Integration**
- Direct YAML → Database operations
- Auto-CRUD generation
- Schema migrations

### **Phase 3: Proxy & Capture**  
- Capture existing APIs on-the-fly
- Mirror and enhance existing services
- API evolution tracking

### **Phase 4: Production Features**
- Performance optimization
- Advanced monitoring
- Deployment tools

---

## 🔒 **What NOT to Change**

1. **Core concept** - YAML → Backend API
2. **Simplicity principle** - Keep it simple for basic use cases
3. **Integrated dashboard** - Don't separate the monitoring
4. **Developer focus** - This is for rapid development, not production scale

---

## ⚠️ **When Confused, Remember:**

> **"Backworks transforms YAML into working backend APIs"**

- If it's not YAML → API, it's not core Backworks
- If it's complex for simple use cases, it's wrong direction  
- If it requires separate processes/servers, reconsider
- If developers can't get started in 5 minutes, simplify

---

**This document represents the current, agreed-upon direction for Backworks.**  
**Refer here when making architectural decisions.**
