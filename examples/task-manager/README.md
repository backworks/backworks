# Task Manager API

A complete task/todo management backend with teams, projects, and reporting.

## 🎯 What This Creates

**YAML Configuration** → **Full Task Management API**

### Core Features:
- ✅ **Task CRUD** - Create, read, update, delete tasks
- 📋 **Project Management** - Organize tasks into projects  
- 👥 **Team Management** - Assign tasks to team members
- 📊 **Productivity Reports** - Track progress and performance
- 🏷️ **Tags & Priorities** - Organize and prioritize work
- 💬 **Comments & Attachments** - Rich task details

### Endpoints:
- `GET /tasks` - List tasks with summary stats
- `POST /tasks` - Create new task
- `GET /tasks/{id}` - Get task details with comments
- `PUT /tasks/{id}` - Update task
- `DELETE /tasks/{id}` - Delete task
- `GET /projects` - List projects with progress
- `POST /projects` - Create new project
- `GET /team` - Team members and workload
- `GET /reports/productivity` - Performance analytics

## 🚀 Run It

```bash
# From the task-manager directory
backworks start --config api.yaml
```

## 🧪 Test It

```bash
# Get all tasks with summary
curl http://localhost:3000/tasks

# Get specific task with full details
curl http://localhost:3000/tasks/1

# Create a new task
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Fix bug in authentication", 
    "priority": "high",
    "assigned_to": "john@example.com"
  }'

# Get team workload
curl http://localhost:3000/team

# Get productivity report
curl http://localhost:3000/reports/productivity
```

## 📊 Dashboard

Visit http://localhost:3001 to see:
- Real-time task creation/completion metrics
- API endpoint usage patterns
- Team productivity insights

## 💡 Advanced Features Demonstrated

- **Complex nested data** (tasks with comments, attachments)
- **Business logic simulation** (completion rates, productivity scores)
- **Realistic workflows** (task status progression)
- **Team collaboration** (assignments, workload balancing)
- **Analytics & reporting** (trends, bottlenecks, performance)

This shows how Backworks can power sophisticated business applications!
