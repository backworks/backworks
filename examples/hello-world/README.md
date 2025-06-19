# 🌟 Hello World API

The simplest possible Backworks example - pure YAML → working API.

## 🎯 What This Does

**YAML Configuration** → **Working Backend API**

This creates two endpoints:
- `GET /hello` - Returns a greeting with timestamp
- `POST /echo` - Echoes back your data

## 🚀 Run It

```bash
# From the hello-world directory
backworks start --config api.yaml
```

## 🧪 Test It

```bash
# Hello endpoint
curl http://localhost:3002/hello

# Echo endpoint  
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

This example shows how a simple YAML file becomes a fully functional API server!
