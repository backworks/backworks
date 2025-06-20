# Advanced Proxy API

This example demonstrates Backworks' advanced proxy capabilities with multiple handler approaches.

## Project Structure

```
advanced-proxy/
├── package.json              # Project metadata and configuration
├── blueprints/
│   └── main.yaml             # API endpoint definitions
├── handlers/                 # External JavaScript handlers
│   ├── posts-proxy.js        # Complex proxy with transformations
│   ├── dashboard-aggregator.js # Multi-source data aggregation
│   ├── request-transform.js  # Request transformation logic
│   ├── response-transform.js # Response transformation logic
│   └── post-transform.js     # Post-specific transformations
└── README.md                 # This file
```

## Handler Approaches

This example demonstrates **two complementary approaches** for handling requests:

### 1. External JavaScript Handlers (Recommended for Complex Logic)

For complex transformation logic, authentication, or business rules, use external JavaScript files:

```yaml
- path: "/posts/{id}"
  method: GET
  handler: "./handlers/posts-proxy.js"
  description: "Complex proxy with transformations"
```

**Benefits:**
- **Better Code Organization**: Keep complex logic in separate files
- **Full IDE Support**: Syntax highlighting, debugging, and IntelliSense
- **Easy Testing**: Unit test handlers independently
- **Reusability**: Share handlers across multiple endpoints
- **Version Control**: Track changes to business logic separately

**Examples in this project:**
- `handlers/posts-proxy.js` - Complex proxy with request/response transformations
- `handlers/dashboard-aggregator.js` - Multi-source data aggregation with error handling
- `handlers/request-transform.js` - Request header and parameter transformations
- `handlers/response-transform.js` - Response formatting and metadata injection

### 2. Inline JavaScript Functions (Great for Simple Logic)

For simple transformations or quick prototyping, define functions directly in the blueprint:

```yaml
- path: "/api/users"
  method: GET
  description: "Simple proxy with inline transformations"
  handler: |
    function handler(req, res) {
      // Simple transformation logic here
      return {
        status: 200,
        body: { message: "Transformed inline" }
      };
    }
```

**Benefits:**
- **Everything in One Place**: No external dependencies
- **Quick Prototyping**: Rapidly test ideas without file management
- **Configuration-Driven**: Easy to modify behavior through config changes
- **Simple Logic**: Perfect for basic transformations and responses

## API Endpoints

### External Handler Examples

#### 1. `GET /posts/{id}` - Advanced Proxy
- **Handler**: `handlers/posts-proxy.js`
- **Features**: Dynamic routing, request/response transformation, error handling
- **Demonstrates**: Complex proxy patterns with metadata injection

#### 2. `GET /api/dashboard` - Data Aggregation
- **Handler**: `handlers/dashboard-aggregator.js`
- **Features**: Parallel API calls, data merging, partial failure handling
- **Demonstrates**: Multi-source data aggregation patterns

#### 3. `GET /api/test` - Request Transformation
- **Handler**: `handlers/request-transform.js`
- **Features**: Header manipulation, authentication, parameter transformation
- **Demonstrates**: Comprehensive request preprocessing

### Inline Handler Examples

#### 4. `GET /api/users` - Simple Proxy
- **Handler**: Inline function in `blueprints/main.yaml`
- **Features**: Basic proxy logic with response transformation
- **Demonstrates**: Simple proxy patterns with inline transformations

#### 5. `GET /health` - Health Check
- **Handler**: Inline function in `blueprints/main.yaml`
- **Features**: System status reporting
- **Demonstrates**: Simple endpoint with static response

## Usage

### 1. Start the Server

```bash
# Navigate to the example directory
cd examples/advanced-proxy

# Start with auto-reload during development
npm run dev

# Or start normally
npm start
```

### 2. Test the Endpoints

```bash
# Test external handler - complex proxy
curl http://localhost:3006/posts/1

# Test external handler - data aggregation
curl "http://localhost:3006/api/dashboard?users=true&posts=true"

# Test inline handler - simple proxy
curl http://localhost:3006/api/users

# Test inline handler - health check
curl http://localhost:3006/health
```

### 3. View the Dashboard

Open http://localhost:3007 to see the Backworks dashboard with endpoint metrics and logs.

## Configuration Highlights

### Package.json Configuration
```json
{
  "name": "advanced-proxy-api",
  "main": "blueprints/main.yaml",
  "backworks": {
    "entrypoint": "blueprints/main.yaml",
    "server": { "host": "0.0.0.0", "port": 3006 },
    "dashboard": { "enabled": true, "port": 3007 }
  }
}
```

### Blueprint Structure
- **External handlers**: Referenced as `"./handlers/filename.js"`
- **Inline handlers**: Defined using YAML `|` multi-line strings
- **Mixed approaches**: Use both in the same project as needed

## Best Practices Demonstrated

1. **Handler Selection**:
   - Use external handlers for complex logic (>20 lines)
   - Use inline handlers for simple transformations (<10 lines)

2. **Error Handling**:
   - Comprehensive error handling in external handlers
   - Graceful degradation for proxy failures

3. **Code Organization**:
   - Separate concerns: proxy logic, transformations, aggregation
   - Clear naming conventions for handlers

4. **Performance**:
   - Parallel API calls in dashboard aggregator
   - Efficient request/response transformations

This example serves as a comprehensive template for building production-ready API gateways with Backworks.

### 1. External JavaScript Handlers

For complex transformation logic, use external JavaScript files:

```yaml
transform:
  request:
    handler: "./handlers/request-transform.js"
  response:
    handler: "./handlers/response-transform.js"
```

**Benefits:**
- Better code organization and reusability
- Full IDE support with syntax highlighting
- Easy testing and version control
- Suitable for complex business logic

**Examples:**
- `handlers/request-transform.js` - Adds authentication, headers, and request tracking
- `handlers/response-transform.js` - Transforms responses with metadata and analysis
- `handlers/post-transform.js` - Post-specific enhancements and categorization

### 2. Inline JavaScript Functions

For simple transformations, define functions directly in the YAML:

```yaml
transform:
  response:
    handler: |
      function transform(response) {
        response.headers['X-Transformed'] = 'true';
        response.body.transformed_at = new Date().toISOString();
        return response;
      }
```

**Benefits:**
- Everything in one file - no external dependencies
- Quick and simple for basic transformations
- Good for configuration-driven scenarios
- Immediate visibility of transformation logic

### 3. Mixed Approach

Combine both approaches based on complexity:

```yaml
transform:
  request:
    handler: "./handlers/complex-request-transform.js"  # External for complex logic
  response:
    handler: |                                         # Inline for simple transforms
      function transform(response) {
        response.headers['X-Simple-Transform'] = 'true';
        return response;
      }
```

## API Endpoints

### 1. `/api/test` - External Handlers Demo
- **Request Transform**: Adds authentication, tracking headers
- **Response Transform**: Wraps response with metadata and analysis
- Demonstrates complex external transformation logic

### 2. `/posts/{id}` - Post-Specific Transforms
- **Response Transform**: Enhances posts with reading time, engagement metrics, categorization
- Shows domain-specific transformation logic

### 3. `/api/users` - Inline Functions Demo
- **Request Transform**: Adds proxy headers (inline function)
- **Response Transform**: User profile enhancements (inline function)
- Demonstrates simple inline transformation approach

### 4. `/api/posts` - Mixed Approach Demo
- **Request Transform**: Complex request handling (external file)
- **Response Transform**: Simple response formatting (inline function)
- Shows how to combine both approaches effectively

## Usage

1. Start the Backworks server:
```bash
cd /path/to/backworks
cargo run -- examples/advanced-proxy/blueprint.yaml
```

2. Test the endpoints:

### Test JSON transformation
```bash
curl http://localhost:3000/test/json
```

### Test path rewriting and query transformations
```bash
curl "http://localhost:3000/posts/1?page=1&size=5&debug=true"
```

### Test POST with body transformation
```bash
curl -X POST http://localhost:3000/posts \
  -H "Content-Type: application/json" \
  -d '{"user_id": 123, "title": "Test Post", "password": "secret123"}'
```

### Test status code mapping
```bash
curl http://localhost:3000/status/404
curl http://localhost:3000/status/500
```

## Expected Transformations

### Request Transformations Applied:
- **Headers**: `X-API-Version: 2.0`, `X-Client: Backworks` added
- **Security**: `Cookie` and sensitive headers removed
- **Paths**: `/api/v1` prefix stripped, `/api/v2` prefix added
- **Query**: `page`→`offset`, `size`→`limit`, `debug` removed
- **Body**: `user_id`→`userId`, `password` field removed, `timestamp` added

### Response Transformations Applied:
- **Status**: 404→200, 500→503 (configurable mapping)
- **Headers**: CORS headers added, server headers removed
- **Body**: Wrapped in standard format with metadata

## Configuration Highlights

The `blueprint.yaml` shows:
- Multi-target proxy configuration
- Comprehensive transformation rules
- Health check and load balancing setup
- Security header management
- Response standardization

This example serves as a template for building production-ready API gateways with Backworks.
