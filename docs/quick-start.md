# ⚡ Quick Start Guide

Get your first Backworks API running in under 5 minutes!

## 🚀 Installation

### Build from Source
```bash
git clone https://github.com/devstroop/backworks
cd backworks
cargo build --release
```

The binary will be available at `./target/release/backworks`

## 🎯 Your First API in 2 Minutes

### 1. Create Service Schematic
Create a `blueprint.yaml` file:

```yaml
name: "My First API"
description: "A simple user management API"

server:
  host: "0.0.0.0"
  port: 3000

dashboard:
  enabled: true
  port: 3001

mode: "runtime"

endpoints:
  hello:
    path: "/hello"
    methods: ["GET"]
    description: "Say hello to the world"
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          return {
            status: 200,
            headers: { "Content-Type": "application/json" },
            body: {
              message: "Hello, World!",
              timestamp: new Date().toISOString(),
              method: req.method,
              path: req.path
            }
          };
        }

  users:
    path: "/users"
    methods: ["GET", "POST"]
    description: "Manage users"
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          const users = [
            { id: 1, name: "John Doe", email: "john@example.com" },
            { id: 2, name: "Jane Smith", email: "jane@example.com" }
          ];
          
          if (req.method === "GET") {
            return {
              status: 200,
              body: { users: users, count: users.length }
            };
          } else if (req.method === "POST") {
            const newUser = {
              id: users.length + 1,
              name: req.body.name || "New User",
              email: req.body.email || "user@example.com"
            };
            return {
              status: 201,
              body: { message: "User created", user: newUser }
            };
          }
        }

  user_detail:
    path: "/users/{id}"
    methods: ["GET"]
    description: "Get user by ID"
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          const userId = req.path_params?.id || "1";
          return {
            status: 200,
            body: {
              id: parseInt(userId),
              name: `User ${userId}`,
              email: `user${userId}@example.com`,
              retrieved_at: new Date().toISOString()
            }
          };
        }
```

### 2. Start Your API
```bash
./target/release/backworks start --config blueprint.yaml
```

You should see output like:
```
🚀 Starting Backworks... 
✅ Service schematic loaded: My First API
✅ Backworks engine initialized
🌐 API server running on http://0.0.0.0:3000
📊 Dashboard available at http://0.0.0.0:3001
```

### 3. Test Your API
```bash
# Test the hello endpoint
curl http://localhost:3000/hello

# Get all users
curl http://localhost:3000/users

# Get specific user
curl http://localhost:3000/users/123

# Create a new user
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice Cooper", "email": "alice@example.com"}'
```

### 4. View Dashboard
Open your browser to `http://localhost:3001` to see:
- Real-time API metrics
- Request logs
- Endpoint status
- System performance

## � Understanding the Configuration

### Basic Structure
```yaml
name: "API Name"                    # Required: API identifier
description: "What this API does"   # Optional: API description

server:                             # Server configuration
  host: "0.0.0.0"                  # Bind address
  port: 3000                       # API port

dashboard:                          # Dashboard configuration
  enabled: true                    # Enable dashboard
  port: 3001                       # Dashboard port

mode: "runtime"                     # Execution mode (currently only "runtime")

endpoints:                          # API endpoints
  endpoint_name:                    # Endpoint identifier
    path: "/api/path"              # URL path
    methods: ["GET", "POST"]        # HTTP methods
    description: "What this does"   # Optional description
    runtime:                        # JavaScript handler
      language: "javascript"       # Only JavaScript supported currently
      handler: |                   # JavaScript function
        function handler(req, res) {
          // Your logic here
          return {
            status: 200,
            headers: { "Content-Type": "application/json" },
            body: { message: "Success" }
          };
        }
```

### Handler Function Details

Your JavaScript handler receives a `req` object with:
- `req.method` - HTTP method (GET, POST, etc.)
- `req.path` - Request path
- `req.path_params` - Path parameters (e.g., {id} in /users/{id})
- `req.body` - Request body (parsed JSON for POST/PUT)
- `req.headers` - Request headers
- `req.query_params` - Query string parameters

Return an object with:
- `status` - HTTP status code (required)
- `headers` - Response headers (optional)
- `body` - Response body (optional)

## 📋 Commands Reference

### Start API Server
```bash
# Start with default config (project.yaml)
./target/release/backworks start

# Start with custom config
./target/release/backworks start --config my-api.yaml

# Override ports
./target/release/backworks start --port 8080 --dashboard-port 8081

# Enable verbose logging
./target/release/backworks start --verbose
```

### Validate Configuration
```bash
# Validate configuration file
./target/release/backworks validate --config my-api.yaml
```

### Initialize New Project
```bash
# Create new project with basic template
./target/release/backworks init my-new-api

# Create with specific template
./target/release/backworks init my-api --template basic
```

## 🎨 Dashboard Features

The built-in dashboard provides:

### Real-Time Monitoring
- **Request Count** - Total requests per endpoint
- **Response Times** - Average and percentile response times
- **Status Codes** - Distribution of HTTP status codes
- **Error Rates** - Success/failure ratios

### Request Logs
- **Live Request Feed** - See requests as they happen
- **Request Details** - Method, path, status, response time
- **Error Tracking** - Failed requests with error details

### System Health
- **Server Status** - Uptime and health
- **Memory Usage** - Resource consumption
- **Configuration Info** - Current settings

Access the dashboard at `http://localhost:3001` (or your configured dashboard port).

## 🚀 Next Steps

1. **Explore Examples** - Check the [examples](../examples/) directory for more patterns
2. **Read Configuration Reference** - Learn all [configuration options](./configuration.md)
3. **Understand Architecture** - Review the [architecture documentation](../ARCHITECTURE.md)
4. **Check Current Direction** - See [development direction](../DIRECTION.md)
5. **Try Advanced Patterns** - Experiment with complex JavaScript handlers

## 💡 Pro Tips

### Development Workflow
1. **Start Simple** - Begin with basic endpoints
2. **Test Frequently** - Use curl or your favorite HTTP client
3. **Monitor Dashboard** - Watch real-time metrics
4. **Iterate Fast** - Modify configuration and restart

### Common Patterns
```javascript
// Handle different HTTP methods
function handler(req, res) {
  switch(req.method) {
    case 'GET':
      return { status: 200, body: { message: 'Getting data' } };
    case 'POST':
      return { status: 201, body: { message: 'Created', data: req.body } };
    default:
      return { status: 405, body: { error: 'Method not allowed' } };
  }
}

// Use path parameters
function handler(req, res) {
  const id = req.path_params?.id;
  return {
    status: 200,
    body: { id: id, message: `Retrieved item ${id}` }
  };
}

// Handle errors gracefully
function handler(req, res) {
  try {
    // Your logic here
    return { status: 200, body: { success: true } };
  } catch (error) {
    return { 
      status: 500, 
      body: { error: 'Internal server error', message: error.message } 
    };
  }
}
```

## 🤝 Getting Help

- 📚 **Documentation**: Browse the full [documentation](./README.md)
-  **Issues**: Report bugs on [GitHub Issues](https://github.com/devstroop/backworks/issues)
- 💡 **Features**: Request features via GitHub Issues
- 📖 **Examples**: Check out the [examples directory](../examples/)

## ⚠️ Current Limitations

- **JavaScript Only**: Currently only JavaScript handlers are supported
- **Single Mode**: Only "runtime" mode is implemented
- **No Database**: Direct database integration not yet available
- **No Plugins**: Plugin system is planned but not implemented

See [DIRECTION.md](../DIRECTION.md) for the current development roadmap.

---

Start building amazing APIs with Backworks today! 🚀
