# Hello World API

The simplest possible Backworks API demonstrating the project-based architecture with `backworks.json` metadata.

## Project Structure

```
hello-world/
├── backworks.json          # Project metadata
├── blueprints/
│   └── main.yaml          # Main blueprint with endpoints
└── README.md
```

## 🎯 What This Does

**Project-Based API** → **Working Backend Service**

This creates two endpoints:
- `GET /hello` - Returns a greeting with timestamp
- `POST /echo` - Echoes back your data

## 🚀 Quick Start

```bash
# Navigate to the project
cd hello-world

# Start the API (uses backworks.json automatically)
backworks start

# Test the endpoints
curl http://localhost:3002/hello
curl -X POST http://localhost:3002/echo -H "Content-Type: application/json" -d '{"test": "data"}'

# Access the built-in dashboard
open http://localhost:3003
```

## Development

```bash
# Start with auto-reload
backworks start --watch

# Validate the blueprint
backworks validate

# Run tests
backworks test
```

## Migration from Single File

If you have an existing `blueprint.yaml` file, you can easily migrate:

```bash
# Convert existing blueprint
backworks migrate --from blueprint.yaml --to backworks.json

# Or continue using the old format
backworks start --config blueprint.yaml
```

This example demonstrates how the project-based approach provides better organization while maintaining the same simplicity for basic use cases.

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

This example shows how a simple service schematic becomes a fully functional API server!
