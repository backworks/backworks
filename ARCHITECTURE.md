# 🚀 Backworks: Declarative Backend Platform

## 🎯 **Core Concept**

**Backworks transforms service schematics into working backend APIs.**

```yaml
# You write this service schematic (blueprint.yaml)
endpoints:
  users:
    path: "/users"
    methods: ["GET", "POST"]
    runtime:
      handler: "return { users: [...] }"
```

```bash
# Get this working API
curl http://localhost:3000/users
```

**Backworks IS your backend** - not a proxy, not a mock server.

---

## 📋 **What Backworks Does**

### ✅ **Current Core Features**
- **Schematics → API Endpoints** - Define endpoints, get working HTTP API
- **Runtime Execution** - JavaScript handlers execute your business logic
- **Real-time Dashboard** - Monitor API usage, performance, and metrics
- **Integrated Server** - One command starts API + Dashboard

### 🔄 **Future Phases** (Not Current)
- **Proxy Mode** - Capture existing APIs on-the-fly
- **Database Integration** - Direct YAML → Database operations
- **Plugin System** - Extend with custom functionality

---

## 🏗️ **Architecture Principles**

### 1. **Simplicity First**
- One YAML file = Complete backend
- No complex build processes
- No external dependencies for basic use

### 2. **YAML-Driven Everything**
- Configuration defines behavior
- No code required for simple APIs
- JavaScript handlers for complex logic

### 3. **Integrated Experience**
- API server + Dashboard in one process
- Real-time metrics out of the box
- Developer-friendly from day one

---

## 🎮 **Quick Start**

```bash
# 1. Create API definition
cat > my-api.yaml << EOF
name: "My API"
server:
  port: 3000
dashboard:
  enabled: true
  port: 3001
mode: "runtime"
endpoints:
  hello:
    path: "/hello"
    methods: ["GET"]
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          return {
            status: 200,
            body: { message: "Hello, World!" }
          };
        }
EOF

# 2. Run it
backworks start --config my-api.yaml

# 3. Use it
curl http://localhost:3000/hello
# Dashboard: http://localhost:3001
```

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

**Backworks: Because APIs should be this simple.**
