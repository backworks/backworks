# Hello World API

The simplest possible Backworks example.

## 🎯 What This Does

**YAML** → **Working API**

This creates two endpoints:
- `GET /hello` - Returns a greeting
- `POST /echo` - Echoes back data

## 🚀 Run It

```bash
# From the hello-world directory
backworks start --config api.yaml
```

## 🧪 Test It

```bash
# Hello endpoint
curl http://localhost:3000/hello

# Echo endpoint  
curl -X POST http://localhost:3000/echo \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice"}'
```

## 📊 Dashboard

Visit http://localhost:3001 to see:
- Live request metrics
- Endpoint status
- Real-time logs

## 💡 Key Concepts

- **mode: "mock"** - Backworks creates the endpoints
- **endpoints** - Define your API structure
- **mock_responses** - What each endpoint returns

This is **not a proxy** - Backworks **is** your backend!
