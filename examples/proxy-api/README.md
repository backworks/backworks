# Proxy API Example

This example demonstrates Backworks' basic proxy capabilities using inline handlers.

## Project Structure

```
proxy-api/
├── package.json              # Project metadata and configuration
├── blueprints/
│   └── main.yaml             # API endpoint definitions with inline handlers
└── README.md                 # This file
```

## Features

- **GitHub API Proxy**: Route requests to GitHub API with inline transformations
- **HTTPBin Proxy**: Proxy to HTTPBin with simulated load balancing
- **Inline Handlers**: All logic defined directly in the blueprint YAML
- **Load Balancing Simulation**: Weighted random selection between targets
- **Health Monitoring**: Built-in health check endpoint

## Inline Handler Approach

This example uses **inline handlers only** to demonstrate:
- Simple proxy logic without external file dependencies
- Load balancing simulation within inline functions
- Request/response transformation in YAML
- Quick prototyping and configuration-driven development

All handlers are defined using YAML multi-line strings (`|`):

```yaml
- path: "/github/{path}"
  method: [GET, POST]
  handler: |
    async function handler(req, res) {
      // Inline proxy logic here
      return { status: 200, body: {...} };
    }
```

## Usage

### 1. Start the Server

```bash
# Navigate to the example directory
cd examples/proxy-api

# Start with auto-reload
npm run dev

# Or start normally  
npm start
```

### 2. Test the Endpoints

```bash
# Test GitHub API proxy
curl http://localhost:3010/github/users/octocat
curl http://localhost:3010/github/repos/octocat/Hello-World

# Test HTTPBin proxy with load balancing
curl http://localhost:3010/httpbin/ip
curl http://localhost:3010/httpbin/user-agent
curl http://localhost:3010/httpbin/headers

# Test POST requests
curl -X POST http://localhost:3010/httpbin/post \
  -H "Content-Type: application/json" \
  -d '{"test": "data"}'

# Health check
curl http://localhost:3010/health
```

### 3. View the Dashboard

Open http://localhost:3011 to see the Backworks dashboard with:
- Request metrics and response times
- Load balancing distribution
- Error rates and health status

1. **Path-based Routing**: Different endpoints proxy to different services
2. **Load Balancing**: Weighted round-robin between multiple backends
3. **Health Monitoring**: Automatic health checks and failover
4. **Request Capture**: Learn API schemas and patterns
5. **Resilience**: Retry logic and circuit breakers

## Expected Responses

### GitHub API Responses
- **`/github/users/octocat`** - Returns mock GitHub user profile
- **`/github/repos/octocat/Hello-World`** - Returns mock repository information
- **Headers added**: `X-Proxy-Target`, `X-Handler-Type`, `User-Agent`

### HTTPBin Responses  
- **`/httpbin/ip`** - Returns client IP information
- **`/httpbin/headers`** - Returns request headers
- **`/httpbin/status/404`** - Returns specified status code
- **Load balancing**: Randomly selects between primary (70%) and backup (30%)
- **Headers added**: `X-Proxy-Target`, `X-Target-URL`, `X-Load-Balancer`

## Configuration Highlights

### Package.json Configuration
```json
{
  "name": "proxy-api",
  "main": "blueprints/main.yaml",
  "backworks": {
    "entrypoint": "blueprints/main.yaml",
    "server": { "host": "0.0.0.0", "port": 3010 },
    "dashboard": { "enabled": true, "port": 3011 }
  }
}
```

### Inline Handler Benefits Demonstrated
1. **Zero External Dependencies**: Everything contained in blueprint YAML
2. **Quick Prototyping**: Easy to modify and test proxy logic
3. **Configuration-Driven**: Change behavior without touching separate files
4. **Simple Deployment**: Single blueprint file contains all logic

## Best Practices Shown

- **Error Handling**: Comprehensive try-catch blocks in inline handlers
- **Load Balancing**: Weighted random selection algorithm
- **Header Management**: Adding proxy identification headers
- **Status Code Handling**: Proper HTTP status code propagation
- **Request Simulation**: Mock responses for testing and development

This example is perfect for:
- Learning basic proxy concepts
- Quick API gateway prototyping
- Simple load balancing scenarios
- Configuration-driven proxy setups
