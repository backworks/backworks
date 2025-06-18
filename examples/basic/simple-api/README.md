# Simple User API

A basic REST API demonstrating core Backworks concepts using mock data.

## Overview

This example shows how to create a simple user management API with Backworks in under 2 minutes. It demonstrates:

- Basic endpoint configuration
- Mock data responses
- Dynamic path parameters
- Different HTTP methods
- Simple dashboard visualization

## Features Demonstrated

- ✅ **Mock Mode** - Instant API with static data
- ✅ **Dynamic Responses** - Path and query parameters
- ✅ **Multiple HTTP Methods** - GET, POST, PUT, DELETE
- ✅ **Visual Dashboard** - Real-time API visualization
- ✅ **Request Logging** - See all requests in dashboard

## Quick Start

1. **Start the API**:
   ```bash
   cd examples/basic/simple-api
   backworks start
   ```

2. **API is running**:
   - API: http://localhost:8080
   - Dashboard: http://localhost:3000

3. **Test the endpoints**:
   ```bash
   # Get all users
   curl http://localhost:8080/users
   
   # Get specific user
   curl http://localhost:8080/users/1
   
   # Create new user
   curl -X POST http://localhost:8080/users \
     -H "Content-Type: application/json" \
     -d '{"name": "Alice Johnson", "email": "alice@example.com"}'
   
   # Update user
   curl -X PUT http://localhost:8080/users/1 \
     -H "Content-Type: application/json" \
     -d '{"name": "John Updated", "email": "john.updated@example.com"}'
   
   # Delete user
   curl -X DELETE http://localhost:8080/users/1
   ```

## Expected Responses

### GET /users
```json
[
  {
    "id": 1,
    "name": "John Doe",
    "email": "john@example.com",
    "created_at": "2025-01-15T10:00:00Z",
    "status": "active"
  },
  {
    "id": 2,
    "name": "Jane Smith", 
    "email": "jane@example.com",
    "created_at": "2025-01-16T11:30:00Z",
    "status": "active"
  }
]
```

### GET /users/1
```json
{
  "id": 1,
  "name": "John Doe",
  "email": "john@example.com",
  "created_at": "2025-01-15T10:00:00Z",
  "status": "active",
  "profile": {
    "bio": "Software developer",
    "location": "San Francisco, CA"
  }
}
```

### POST /users
```json
{
  "id": 3,
  "name": "Alice Johnson",
  "email": "alice@example.com", 
  "created_at": "2025-01-18T15:45:32Z",
  "status": "active",
  "message": "User created successfully"
}
```

## What You'll Learn

1. **Basic Configuration** - How to set up a simple API with YAML
2. **Mock Responses** - Using static and dynamic mock data
3. **Path Parameters** - Extracting data from URL paths
4. **HTTP Methods** - Handling GET, POST, PUT, DELETE
5. **Response Templates** - Creating consistent API responses
6. **Dashboard Usage** - Monitoring your API in real-time

## Configuration Breakdown

The `backworks.yaml` file demonstrates:

```yaml
# API metadata
name: "simple_user_api"
description: "A simple user management API"

# All endpoints use mock mode by default
mode: "mock"

# Define API endpoints
endpoints:
  # List all users
  users:
    path: "/users"
    methods: ["GET", "POST"]
    # ... configuration
    
  # Individual user operations  
  user_detail:
    path: "/users/{id}"
    methods: ["GET", "PUT", "DELETE"] 
    # ... configuration
```

## Dashboard Features

Open http://localhost:3000 to see:

- **Flow Diagram** - Visual representation of API requests
- **Request Logs** - Real-time log of all API calls
- **Performance Metrics** - Response times and request counts
- **Endpoint Overview** - List of available endpoints

## Next Steps

After mastering this example, try:

1. **[CRUD Operations](../crud-operations/)** - More advanced CRUD patterns
2. **[External APIs](../../integrations/external-apis/)** - Connect to external services
3. **[AI-Powered API](../../ai-enhanced/ai-powered-api/)** - Add AI intelligence

## Customization Ideas

Try modifying the configuration:

1. **Add More Endpoints**:
   ```yaml
   endpoints:
     user_posts:
       path: "/users/{id}/posts"
       mock:
         data: [{"id": 1, "title": "Hello World", "content": "..."}]
   ```

2. **Add Query Parameters**:
   ```yaml
   endpoints:
     search_users:
       path: "/users/search"
       mock:
         data:
           query: "${query.q}"
           results: [{"id": 1, "name": "Found User"}]
   ```

3. **Dynamic Data Generation**:
   ```yaml
   mock:
     data:
       id: "${random_int(1000, 9999)}"
       created_at: "${now()}"
       uuid: "${random_uuid()}"
   ```

This simple example provides a foundation for understanding Backworks concepts before moving to more advanced features!
