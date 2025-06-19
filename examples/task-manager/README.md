# Task Manager

A comprehensive task management application built with Backworks, demonstrating the project-based architecture with `backworks.json` metadata and organized blueprint files.

## Project Structure

```
task-manager/
├── backworks.json              # Project metadata (like package.json)
├── blueprints/                 # Blueprint YAML files
│   ├── main.yaml              # Main application blueprint
│   ├── database.yaml          # Database schemas and configuration
│   ├── plugins.yaml           # Plugin configurations
│   ├── endpoints/             # API endpoint definitions
│   │   ├── tasks.yaml         # Task management endpoints
│   │   └── auth.yaml          # Authentication endpoints
│   └── ui/                    # UI component definitions
│       └── dashboard.yaml     # Dashboard UI components
├── migrations/                # Database migrations
├── static/                    # Static assets
└── target/                    # Compiled outputs
```

## 🚀 Features

### API Endpoints
- **Task Management**: Create, read, update, delete tasks
- **User Authentication**: Register, login, logout, profile
- **Task Categories**: Organize tasks with categories
- **File Attachments**: Attach files to tasks

### Built-in UI Components
- **Dashboard**: Overview with statistics and task table
- **Task Form**: Create and edit tasks with validation
- **Authentication**: Login and registration forms
- **Responsive Design**: Works on desktop and mobile

### Plugins & Middleware
- **JWT Authentication**: Secure API endpoints
- **Rate Limiting**: Prevent API abuse
- **CORS**: Cross-origin request handling
- **Logging**: Structured request/response logging
- **Validation**: Request body validation
- **Caching**: Response caching for performance
- **Health Checks**: System health monitoring
- **Metrics**: Prometheus metrics collection

### Database Features
- **PostgreSQL**: Relational database with proper schemas
- **Migrations**: Database version control
- **Indexes**: Optimized query performance
- **Foreign Keys**: Data integrity constraints
- **Triggers**: Automatic timestamp updates
- **Seeding**: Initial test data
- **Business Logic** - Complex workflows and state management
- **Pagination Support** - Handle large datasets efficiently
- **Rich Data Models** - Detailed task and user information

## 📋 Endpoints

### Tasks
- `GET /tasks` - List tasks with filtering and pagination
  - Query params: `?status=pending`, `?priority=high`, `?assigned_to=email`, `?search=keyword`, `?page=1&limit=10`
- `POST /tasks` - Create new task (with validation)
- `GET /tasks/{id}` - Get detailed task information
- `PUT /tasks/{id}` - Update task (with validation)
- `DELETE /tasks/{id}` - Delete task
- `POST /tasks/{id}/complete` - Mark task as completed

### Users & Team
- `GET /users` - List team members with workload stats
- `GET /users/{id}/tasks` - Get tasks assigned to specific user

## 🏃‍♂️ Running the Example

```bash
# From the backworks root directory
../../target/release/backworks start --config api.yaml

# The API will be available at:
# - API: http://localhost:3006
# - Dashboard: http://localhost:3007
```

## 🧪 Testing the API

### Get all tasks
```bash
curl http://localhost:3006/tasks
```

### Filter tasks by status
```bash
curl "http://localhost:3006/tasks?status=pending"
```

### Search tasks
```bash
curl "http://localhost:3006/tasks?search=backworks"
```

### Get tasks with pagination
```bash
curl "http://localhost:3006/tasks?page=1&limit=5"
```

### Create a new task
```bash
curl -X POST http://localhost:3006/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Implement user authentication",
    "description": "Add JWT-based authentication to the API",
    "priority": "high",
    "assigned_to": "john@example.com",
    "assigned_name": "John Doe",
    "tags": ["security", "authentication"],
    "estimated_hours": 12,
    "due_date": "2025-06-25T18:00:00Z"
  }'
```

### Get specific task
```bash
curl http://localhost:3006/tasks/1
```

### Update a task
```bash
curl -X PUT http://localhost:3006/tasks/1 \
  -H "Content-Type: application/json" \
  -d '{
    "status": "completed",
    "actual_hours": 8
  }'
```

### Mark task as completed
```bash
curl -X POST http://localhost:3006/tasks/1/complete
```

### Get team members
```bash
curl http://localhost:3006/users
```

### Get tasks for specific user
```bash
curl http://localhost:3006/users/1/tasks
```

## 📊 Dashboard

Visit http://localhost:3007 to see:
- Real-time task metrics and status distribution
- Team workload and productivity insights
- Request/response logs for all API calls
- Performance monitoring and error tracking

## 🎯 Learning Points

This example demonstrates advanced API patterns:

1. **Complex Filtering** - Multiple query parameters for data filtering
2. **Input Validation** - Comprehensive validation with detailed error messages
3. **Business Logic** - Status transitions, completion tracking, workload calculation
4. **Data Relationships** - Users, tasks, comments, and assignments
5. **Pagination** - Handling large datasets with page/limit parameters
6. **Error Handling** - Proper HTTP status codes and structured error responses
7. **State Management** - Task status transitions and completion tracking

## 🔧 Configuration Highlights

### Complex Validation
```javascript
// Multiple validation checks with detailed error messages
const validStatuses = ['pending', 'in_progress', 'completed'];
if (req.body.status && !validStatuses.includes(req.body.status)) {
  return {
    status: 400,
    body: { 
      error: 'Validation failed',
      message: 'Status must be one of: ' + validStatuses.join(', '),
      fields: ['status']
    }
  };
}
```

### Advanced Filtering
```javascript
// Multiple filter conditions
let filteredTasks = tasks;
if (status) filteredTasks = filteredTasks.filter(t => t.status === status);
if (priority) filteredTasks = filteredTasks.filter(t => t.priority === priority);
if (assignee) filteredTasks = filteredTasks.filter(t => t.assigned_to === assignee);
```

### Pagination Implementation
```javascript
// Pagination with page and limit
const page = parseInt(req.query_params?.page) || 1;
const limit = parseInt(req.query_params?.limit) || 10;
const offset = (page - 1) * limit;
const paginatedTasks = filteredTasks.slice(offset, offset + limit);
```

### Rich Response Data
```javascript
// Comprehensive response with metadata
return {
  status: 200,
  body: {
    tasks: paginatedTasks,
    pagination: {
      page: page,
      limit: limit,
      total: filteredTasks.length,
      pages: Math.ceil(filteredTasks.length / limit)
    },
    summary: {
      total: tasks.length,
      completed: tasks.filter(t => t.status === 'completed').length,
      in_progress: tasks.filter(t => t.status === 'in_progress').length,
      pending: tasks.filter(t => t.status === 'pending').length
    }
  }
};
```

## 🚀 Next Steps

1. **Add Authentication** - Implement user login and JWT tokens
2. **Database Integration** - Replace mock data with real database
3. **Real-time Updates** - Add WebSocket support for live updates
4. **File Uploads** - Support task attachments
5. **Email Notifications** - Send task assignment and due date reminders
6. **Build Your Own** - Use this as a template for your business applications

## 💡 Pro Tips

### Business Logic Patterns
- **State Validation** - Always validate state transitions
- **Data Consistency** - Ensure related data stays in sync
- **Error Context** - Provide specific error messages and field information
- **Audit Trail** - Track when and who made changes

### API Design Best Practices
- **Consistent Naming** - Use clear, predictable endpoint names
- **Proper HTTP Methods** - Use appropriate verbs for different operations
- **Status Codes** - Return meaningful HTTP status codes
- **Response Structure** - Keep response formats consistent across endpoints

This example shows how Backworks can handle complex business applications with sophisticated logic, making it suitable for real-world use cases beyond simple prototyping.
