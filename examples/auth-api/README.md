# Auth API Example

This example demonstrates how to use the Backworks Auth Plugin to:

1. Register users
2. Authenticate users with username/password
3. Use JWT tokens for authorization
4. Protect endpoints with middleware
5. Implement role-based access control

## Authentication Plugin Features

The Backworks Auth Plugin provides:

- **User Management**: Create and authenticate users
- **Role-Based Access Control**: Assign roles to users
- **JWT-Based Authentication**: Secure token generation and validation
- **Middleware Support**: Protect endpoints based on path, method, and roles

## Getting Started

1. Install dependencies:
```
npm install
```

2. Run the API:
```
npm start
```

## API Endpoints

### Authentication

- **POST /api/auth/register** - Register a new user
  ```json
  {
    "username": "testuser",
    "password": "password123",
    "email": "user@example.com",
    "full_name": "Test User"
  }
  ```

- **POST /api/auth/login** - Login with username and password
  ```json
  {
    "username": "testuser",
    "password": "password123"
  }
  ```

- **GET /api/auth/validate** - Validate a token (requires Authorization header)

### Protected Resources

- **GET /api/resources** - Get all resources (requires authentication)
- **POST /api/resources** - Create a resource (requires admin role)
- **GET /api/resources/:id** - Get a resource by ID (requires authentication)
- **PUT /api/resources/:id** - Update a resource (requires admin role)
- **DELETE /api/resources/:id** - Delete a resource (requires admin role)

## Using Authentication

After logging in, you'll receive a JWT token. Include this token in the Authorization header for protected endpoints:

```
Authorization: Bearer YOUR_TOKEN_HERE
```

## Middleware Configuration

This example includes two middleware configurations:

1. **authenticate** - Applied to all `/api/resources*` paths
   - Requires any user role (either "user" or "admin")
   - Validates the JWT token
   - Attaches the user to the request

2. **admin_only** - Applied to POST, PUT, DELETE methods on `/api/resources`
   - Restricts access to users with the "admin" role
   - Ensures only admins can create, update, or delete resources

### Configuring Middleware in Blueprint

```yaml
middlewares:
  - name: authenticate
    plugin: auth
    config:
      path: "/api/resources*"
      roles: ["user", "admin"]
  
  - name: admin_only
    plugin: auth
    config:
      path: "/api/resources"
      methods: ["POST", "PUT", "DELETE"]
      roles: ["admin"]
```
