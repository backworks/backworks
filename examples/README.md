# 🎮 Backworks Examples

This directory contains practical examples showing how to use Backworks to create APIs with YAML configuration.

## 📁 Available Examples

| Example | Description | Complexity | Features Demonstrated |
|---------|-------------|------------|----------------------|
| [**hello-world**](./hello-world/) | Simplest possible API | ⭐ | Basic endpoints, JavaScript handlers |
| [**blog-api**](./blog-api/) | Blog with posts & comments | ⭐⭐⭐ | CRUD operations, path parameters |
| [**task-manager**](./task-manager/) | Complete business app | ⭐⭐⭐⭐ | Complex business logic, validation |

## 🚀 Running Examples

### Prerequisites
```bash
# Build Backworks from source
git clone https://github.com/devstroop/backworks
cd backworks
cargo build --release
```

### Run an Example
```bash
# Navigate to example directory
cd examples/hello-world

# Run the example
../../target/release/backworks start --config api.yaml

# Test the API
curl http://localhost:3002/hello

# View dashboard
open http://localhost:3003
```

## 📋 Example Details

### 🌟 Hello World
**File:** `hello-world/api.yaml`
**Ports:** API: 3002, Dashboard: 3003

The simplest possible Backworks API demonstrating:
- Basic endpoint configuration
- JavaScript handler functions
- Request/response handling
- Dashboard integration

**Endpoints:**
- `GET /hello` - Simple greeting with timestamp
- `POST /echo` - Echo back request data

### 📝 Blog API
**File:** `blog-api/api.yaml`
**Ports:** API: 3004, Dashboard: 3005

A more complex example showing:
- Multiple related endpoints
- Path parameters (`{id}`)
- Different HTTP methods
- Data relationships

**Endpoints:**
- `GET /posts` - List all blog posts
- `POST /posts` - Create new post
- `GET /posts/{id}` - Get specific post
- `GET /posts/{id}/comments` - Get post comments
- `POST /posts/{id}/comments` - Add comment to post

### 📋 Task Manager
**File:** `task-manager/api.yaml`
**Ports:** API: 3006, Dashboard: 3007

A comprehensive business application example demonstrating:
- Full CRUD operations
- Input validation
- Error handling
- Complex business logic
- Multiple resource types

**Endpoints:**
- `GET /tasks` - List tasks with filtering
- `POST /tasks` - Create new task
- `GET /tasks/{id}` - Get task details
- `PUT /tasks/{id}` - Update task
- `DELETE /tasks/{id}` - Delete task
- `POST /tasks/{id}/complete` - Mark task as complete
- `GET /users` - List users
- `GET /users/{id}/tasks` - Get user's tasks

## 🎯 Learning Path

### 1. Start with Hello World
```bash
cd examples/hello-world
../../target/release/backworks start --config api.yaml
```

**Learn:**
- Basic YAML structure
- Simple JavaScript handlers
- Request/response format
- Dashboard features

### 2. Progress to Blog API
```bash
cd examples/blog-api
../../target/release/backworks start --config api.yaml
```

**Learn:**
- Path parameters
- Multiple endpoints
- HTTP methods
- Data modeling

### 3. Master Task Manager
```bash
cd examples/task-manager
../../target/release/backworks start --config api.yaml
```

**Learn:**
- Complex business logic
- Input validation
- Error handling
- Resource relationships

## 🔧 Common Patterns

### Basic Endpoint Pattern
```yaml
endpoints:
  endpoint_name:
    path: "/path"
    methods: ["GET"]
    description: "What this endpoint does"
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          return {
            status: 200,
            body: { message: "Success" }
          };
        }
```

### Path Parameters Pattern
```yaml
endpoints:
  resource_detail:
    path: "/resources/{id}"
    methods: ["GET"]
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          const id = req.path_params.id;
          return {
            status: 200,
            body: { id: parseInt(id), name: `Resource ${id}` }
          };
        }
```

### CRUD Operations Pattern
```yaml
endpoints:
  items:
    path: "/items"
    methods: ["GET", "POST"]
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          if (req.method === 'GET') {
            return { status: 200, body: { items: [] } };
          } else if (req.method === 'POST') {
            return { 
              status: 201, 
              body: { 
                message: 'Created',
                item: req.body 
              } 
            };
          }
        }
```

### Error Handling Pattern
```yaml
endpoints:
  validated_endpoint:
    path: "/validate"
    methods: ["POST"]
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          if (!req.body || !req.body.name) {
            return {
              status: 400,
              body: { 
                error: 'Validation failed',
                message: 'Name is required' 
              }
            };
          }
          
          return {
            status: 200,
            body: { message: 'Valid', data: req.body }
          };
        }
```

## 🧪 Testing Examples

### Manual Testing with curl

```bash
# Test Hello World
curl http://localhost:3002/hello
curl -X POST http://localhost:3002/echo -H "Content-Type: application/json" -d '{"test": "data"}'

# Test Blog API
curl http://localhost:3004/posts
curl http://localhost:3004/posts/1
curl -X POST http://localhost:3004/posts -H "Content-Type: application/json" -d '{"title": "New Post", "content": "Post content"}'

# Test Task Manager
curl http://localhost:3006/tasks
curl http://localhost:3006/tasks?status=pending
curl -X POST http://localhost:3006/tasks -H "Content-Type: application/json" -d '{"title": "New Task", "description": "Task description"}'
```

### Using httpie (Alternative)
```bash
# Install httpie
pip install httpie

# Test endpoints
http GET localhost:3002/hello
http POST localhost:3002/echo test:='"data"'
```

## 📊 Dashboard Features

Each example includes a dashboard showing:

### Real-Time Metrics
- Request count per endpoint
- Average response times
- Status code distribution
- Error rates

### Request Logs
- Live request feed
- Request/response details
- Execution times
- Error tracking

### System Health
- Server uptime
- Memory usage
- Configuration info

## 🎨 Customizing Examples

### Modify Configuration
1. Edit the `api.yaml` file
2. Restart Backworks
3. Test your changes

### Add New Endpoints
```yaml
endpoints:
  # Add your new endpoint here
  my_endpoint:
    path: "/my-path"
    methods: ["GET"]
    runtime:
      language: "javascript"
      handler: |
        function handler(req, res) {
          return {
            status: 200,
            body: { message: "My custom endpoint" }
          };
        }
```

### Change Ports
```yaml
server:
  port: 8080    # Change API port

dashboard:
  port: 8081    # Change dashboard port
```

## 🚀 Next Steps

After working through the examples:

1. **Read Documentation** - Check [quick-start guide](../docs/quick-start.md)
2. **Configuration Reference** - Learn all options in [configuration.md](../docs/configuration.md)
3. **Build Your API** - Create your own YAML configuration
4. **Understand Architecture** - Review [ARCHITECTURE.md](../ARCHITECTURE.md)

## 💡 Tips & Best Practices

### Development Workflow
1. **Start Simple** - Begin with basic endpoints
2. **Test Frequently** - Use curl or browser
3. **Watch Dashboard** - Monitor real-time metrics
4. **Iterate Fast** - Modify and restart quickly

### JavaScript Handler Tips
```javascript
// Always handle different HTTP methods
function handler(req, res) {
  switch(req.method) {
    case 'GET': return handleGet(req);
    case 'POST': return handlePost(req);
    default: return { status: 405, body: { error: 'Method not allowed' } };
  }
}

// Validate input data
function handler(req, res) {
  if (!req.body?.requiredField) {
    return { status: 400, body: { error: 'Missing required field' } };
  }
  // Process valid data...
}

// Use path parameters
function handler(req, res) {
  const id = req.path_params?.id;
  if (!id) {
    return { status: 400, body: { error: 'ID required' } };
  }
  // Use the ID...
}
```

## 🤝 Contributing Examples

Want to contribute a new example?

1. Create a new directory under `examples/`
2. Add an `api.yaml` configuration file
3. Add a `README.md` explaining the example
4. Test thoroughly
5. Update this main examples README
6. Submit a pull request

**Example Ideas:**
- E-commerce API
- User authentication
- File upload handling
- WebSocket integration
- External API integration

---

Happy API building with Backworks! 🚀
