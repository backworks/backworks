# 🌟 Hello World API

The simplest possible Backworks example demonstrating **two handler approaches**.

## 🎯 What This Demonstrates

**Service Schematic** → **Working Backend API**

This example shows both handler approaches:

### 📄 **Inline Handler** (`/hello`)
- Handler code written directly in the YAML blueprint
- Perfect for simple, short logic
- Great for prototyping and small utilities

### 📁 **External Handler** (`/echo`)  
- Handler code in separate JavaScript file (`handlers/echo.js`)
- Better for complex logic and reusability
- Easier to test and maintain
- Follows modern project structure

## 📁 Project Structure

```
hello-world/
├── package.json          # Project metadata with Backworks config
├── blueprints/
│   └── main.yaml         # API blueprint with both handler types
├── handlers/
│   └── echo.js          # External JavaScript handler
└── README.md
```

## 🔄 Handler Approaches Compared

| Aspect | Inline Handler | External Handler |
|--------|----------------|------------------|
| **Location** | Inside YAML blueprint | Separate `.js` file |
| **Best For** | Simple logic, prototypes | Complex logic, production |
| **Maintainability** | Good for small code | Better for larger code |
| **Reusability** | Limited to single endpoint | Can be shared/imported |
| **Testing** | Harder to unit test | Easy to unit test |
| **IDE Support** | Limited syntax highlight | Full JavaScript support |

## 🚀 Run It

```bash
# From the hello-world directory
backworks start
```

## 🧪 Test It

```bash
# Hello endpoint (inline handler)
curl http://localhost:3002/hello

# Echo endpoint (external handler)
curl -X POST http://localhost:3002/echo \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice", "message": "Hello from client!"}'
```

## 📊 Dashboard

Visit http://localhost:3003 to see:
- Live request metrics
- Endpoint status
- Real-time logs
- API configuration

## 💡 Key Concepts

- **mode: "runtime"** - Uses JavaScript handlers for dynamic responses
- **runtime handlers** - JavaScript functions that process requests
- **endpoints** - Define your API structure and behavior
- **No external dependencies** - Backworks IS your backend

This example shows how a simple service schematic becomes a fully functional API server!
