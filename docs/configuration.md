# 📝 Schematic Reference

Complete reference for Backworks service schematic files (`blueprint.yaml`).

## 🎯 Basic Schematic Structure

```yaml
# Required fields
name: "string"                    # Service name (required)
description: "string"             # Service description (optional)
version: "string"                 # Service version (optional)

# Server configuration
server:
  host: "0.0.0.0"                # Bind address (default: "0.0.0.0")
  port: 3000                     # API server port (default: 8080)

# Dashboard configuration
dashboard:
  enabled: true                  # Enable dashboard (default: false)
  port: 3001                     # Dashboard port (default: 3000)

# Execution mode
mode: "runtime"                  # Currently only "runtime" is supported

# API endpoints
endpoints: {}                    # Endpoint definitions (required)

# Advanced configuration (optional)
global_headers: {}               # Headers to add to all responses
logging: {}                      # Logging configuration
```

## 🔄 Execution Mode

Currently, only one execution mode is supported:

```yaml
mode: "runtime"                  # Execute JavaScript handlers
```

**Planned modes** (not yet implemented):
- `database` - Direct database operations
- `proxy` - Proxy to other services  
- `plugin` - Custom plugin execution

## 🛠️ Server Configuration

```yaml
server:
  host: "0.0.0.0"               # Bind address
                                # - "0.0.0.0" = All interfaces
                                # - "127.0.0.1" = Localhost only
                                # - Specific IP address
  
  port: 3000                    # Port number (1-65535)
```

**Defaults:**
- Host: `0.0.0.0`
- Port: `8080`

## 📊 Dashboard Configuration

```yaml
dashboard:
  enabled: true                 # Enable/disable dashboard
  port: 3001                   # Dashboard port number
```

**Features provided:**
- Real-time request metrics
- Endpoint monitoring
- Request logs
- System health status

## 🛠️ Endpoints Configuration

### Basic Endpoint Structure

```yaml
endpoints:
  endpoint_name:                 # Unique endpoint identifier
    path: "/api/path"           # URL path (required)
    methods: ["GET", "POST"]    # HTTP methods (default: ["GET"])
    description: "Description"   # Optional endpoint description
    runtime:                    # JavaScript handler (required for runtime mode)
      language: "javascript"    # Only "javascript" supported currently
      handler: |               # JavaScript function code
        function handler(req, res) {
          return {
            status: 200,
            body: { message: "Hello" }
          };
        }
```
      # OR inline handler:
      handler: |
        export default async (request, context) => {
          const { data } = request.body;
          // Custom processing logic
          return { processed: true, result: data };
        }
      
      # Runtime configuration
      timeout: 30                      # Timeout in seconds
      memory_limit: "512MB"           # Memory limit
      environment:                    # Environment variables
        NODE_ENV: "production"
        
    # Available APIs for this endpoint
    apis: ["stripe", "sendgrid"]
    
    # Parameter validation
    parameters:
      - name: "id"
        type: "integer"
        required: true
        minimum: 1
      - name: "data"
        type: "object"
        required: true
```

### Database Mode Endpoint
```yaml
endpoints:
  users_db:
    path: "/users"
    methods: ["GET", "POST", "PUT", "DELETE"]
    
    database:
      table: "users"                   # Table name
      auto_crud: true                  # Auto-generate CRUD operations
      
      # Custom queries
      queries:
        GET: "SELECT * FROM users WHERE active = true ORDER BY created_at DESC"
        POST: |
          INSERT INTO users (name, email, created_at) 
          VALUES (${name}, ${email}, NOW()) 
          RETURNING *
        "GET /users/{id}": "SELECT * FROM users WHERE id = ${id}"
        
      # Response transformation
      transform:
        list: "data"                   # Wrap list responses
        single: "user"                 # Wrap single responses
        
### Path Parameters

Use `{parameter}` syntax in paths:

```yaml
endpoints:
  user_detail:
    path: "/users/{id}"
    methods: ["GET"]
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          const userId = req.path_params.id;
          return {
            status: 200,
            body: { 
              id: parseInt(userId),
              name: `User ${userId}` 
            }
          };
        }
```

### Multiple Path Parameters

```yaml
endpoints:
  user_posts:
    path: "/users/{userId}/posts/{postId}"
    methods: ["GET"]
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          const { userId, postId } = req.path_params;
          return {
            status: 200,
            body: { 
              user_id: parseInt(userId),
              post_id: parseInt(postId),
              title: `Post ${postId} by User ${userId}`
            }
          };
        }
```

### HTTP Methods

Supported HTTP methods:

```yaml
endpoints:
  full_crud:
    path: "/items"
    methods: ["GET", "POST", "PUT", "DELETE", "PATCH"]
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          switch(req.method) {
            case 'GET':
              return { status: 200, body: { action: 'read' } };
            case 'POST':
              return { status: 201, body: { action: 'create', data: req.body } };
            case 'PUT':
              return { status: 200, body: { action: 'update', data: req.body } };
            case 'DELETE':
              return { status: 204, body: null };
            case 'PATCH':
              return { status: 200, body: { action: 'partial_update', data: req.body } };
            default:
              return { status: 405, body: { error: 'Method not allowed' } };
          }
        }
```

## 📝 JavaScript Handler Reference

### Request Object (req)

The `req` object contains:

```javascript
{
  method: "GET",                    // HTTP method
  path: "/users/123",              // Full request path
  path_params: { id: "123" },      // Path parameters from {id} syntax
  query_params: { page: "1" },     // Query string parameters
  headers: {                       // Request headers
    "content-type": "application/json",
    "user-agent": "curl/7.68.0"
  },
  body: { name: "John" }           // Parsed request body (JSON)
}
```

### Response Object

Your handler must return an object with:

```javascript
{
  status: 200,                     // HTTP status code (required)
  headers: {                       // Response headers (optional)
    "Content-Type": "application/json",
    "Cache-Control": "no-cache"
  },
  body: {                         // Response body (optional)
    message: "Success",
    data: [...],
    meta: { count: 10 }
  }
}
```

### Handler Examples

#### Simple GET endpoint
```javascript
function handler(req, res) {
  return {
    status: 200,
    body: { 
      message: "Hello, World!",
      timestamp: new Date().toISOString()
    }
  };
}
```

#### POST endpoint with validation
```javascript
function handler(req, res) {
  if (req.method !== 'POST') {
    return { status: 405, body: { error: 'Method not allowed' } };
  }
  
  if (!req.body || !req.body.name) {
    return { 
      status: 400, 
      body: { error: 'Name is required' } 
    };
  }
  
  return {
    status: 201,
    body: {
      message: 'Created successfully',
      data: {
        id: Math.floor(Math.random() * 1000),
        name: req.body.name,
        created_at: new Date().toISOString()
      }
    }
  };
}
```

#### Path parameters and query strings
```javascript
function handler(req, res) {
  const userId = req.path_params?.id;
  const includeDetails = req.query_params?.details === 'true';
  
  const user = {
    id: parseInt(userId),
    name: `User ${userId}`,
    email: `user${userId}@example.com`
  };
  
  if (includeDetails) {
    user.created_at = new Date().toISOString();
    user.last_login = new Date(Date.now() - 86400000).toISOString();
  }
  
  return { status: 200, body: user };
}
```

#### Error handling
```javascript
function handler(req, res) {
  try {
    // Simulate some processing
    if (Math.random() < 0.1) {
      throw new Error('Random error occurred');
    }
    
    return {
      status: 200,
      body: { success: true, data: 'Processed successfully' }
    };
  } catch (error) {
    return {
      status: 500,
      body: { 
        error: 'Internal server error',
        message: error.message,
        timestamp: new Date().toISOString()
      }
    };
  }
}
```

## 🔧 Advanced Configuration

### Global Headers

Add headers to all responses:

```yaml
global_headers:
  "X-API-Version": "1.0"
  "X-Powered-By": "Backworks"
  "Access-Control-Allow-Origin": "*"
```

### Logging Configuration

```yaml
logging:
  level: "info"                  # debug, info, warn, error
  format: "json"                 # json, text
  file: "./backworks.log"        # Optional log file
```

**Log levels:**
- `debug` - Detailed debugging information
- `info` - General information (default)
- `warn` - Warning messages
- `error` - Error messages only

## 📋 Complete Example

Here's a comprehensive configuration example:

```yaml
name: "Todo API"
description: "Simple todo management API"
version: "1.0.0"

server:
  host: "0.0.0.0"
  port: 3000

dashboard:
  enabled: true
  port: 3001

mode: "runtime"

global_headers:
  "X-API-Version": "1.0"
  "Access-Control-Allow-Origin": "*"

logging:
  level: "info"
  format: "json"

endpoints:
  # List todos
  todos:
    path: "/todos"
    methods: ["GET", "POST"]
    description: "Manage todos"
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          const todos = [
            { id: 1, title: "Learn Backworks", completed: false },
            { id: 2, title: "Build an API", completed: true }
          ];
          
          if (req.method === 'GET') {
            const status = req.query_params?.status;
            let filteredTodos = todos;
            
            if (status === 'completed') {
              filteredTodos = todos.filter(t => t.completed);
            } else if (status === 'pending') {
              filteredTodos = todos.filter(t => !t.completed);
            }
            
            return {
              status: 200,
              body: {
                todos: filteredTodos,
                count: filteredTodos.length
              }
            };
          } else if (req.method === 'POST') {
            if (!req.body?.title) {
              return {
                status: 400,
                body: { error: 'Title is required' }
              };
            }
            
            const newTodo = {
              id: todos.length + 1,
              title: req.body.title,
              completed: false,
              created_at: new Date().toISOString()
            };
            
            return {
              status: 201,
              body: { message: 'Todo created', todo: newTodo }
            };
          }
        }

  # Single todo operations
  todo_detail:
    path: "/todos/{id}"
    methods: ["GET", "PUT", "DELETE"]
    description: "Single todo operations"
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          const todoId = parseInt(req.path_params.id);
          
          if (req.method === 'GET') {
            return {
              status: 200,
              body: {
                id: todoId,
                title: `Todo ${todoId}`,
                completed: Math.random() > 0.5,
                created_at: new Date().toISOString()
              }
            };
          } else if (req.method === 'PUT') {
            return {
              status: 200,
              body: {
                id: todoId,
                title: req.body.title || `Todo ${todoId}`,
                completed: req.body.completed || false,
                updated_at: new Date().toISOString()
              }
            };
          } else if (req.method === 'DELETE') {
            return {
              status: 204,
              body: null
            };
          }
        }

  # Health check
  health:
    path: "/health"
    methods: ["GET"]
    description: "API health check"
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          return {
            status: 200,
            body: {
              status: "healthy",
              timestamp: new Date().toISOString(),
              uptime: process.uptime()
            }
          };
        }
```

## ⚠️ Current Limitations

- **JavaScript Only**: Only JavaScript handlers are supported
- **No Async/Await**: Handlers are synchronous functions
- **No External Libraries**: Cannot import external npm packages
- **Limited Built-ins**: Only basic JavaScript features available
- **No File System**: Cannot read/write files from handlers
- **No Network Calls**: Cannot make HTTP requests from handlers

## 🚀 Future Enhancements

Planned configuration features:
- Database integration
- External API calls
- Plugin system
- Multiple language support
- Async handler support
- Environment variable substitution

---

For more examples, check the [examples directory](../examples/) and [quick-start guide](./quick-start.md).
