# Blog API Example

A comprehensive blog backend API demonstrating multiple endpoints, path parameters, and CRUD operations.

## 🚀 Features

- **Posts Management** - Create, read, update, delete blog posts
- **Comments System** - Add and view comments on posts
- **Author Profiles** - View blog author information
- **Search Functionality** - Search through posts and comments
- **Filtering & Querying** - Filter posts by tags, published status
- **Input Validation** - Proper error handling and validation

## 📋 Endpoints

### Posts
- `GET /posts` - List all posts
  - Query params: `?tag=tutorial`, `?include_unpublished=true`
- `POST /posts` - Create new post
- `GET /posts/{id}` - Get specific post
- `PUT /posts/{id}` - Update post
- `DELETE /posts/{id}` - Delete post

### Comments
- `GET /posts/{post_id}/comments` - Get comments for a post
- `POST /posts/{post_id}/comments` - Add comment to post

### Authors
- `GET /authors` - List all authors

### Search
- `GET /search?q=keyword` - Search posts and comments

## 🏃‍♂️ Running the Example

```bash
# From the backworks root directory
../../target/release/backworks start --config api.yaml

# The API will be available at:
# - API: http://localhost:3004
# - Dashboard: http://localhost:3005
```

## 🧪 Testing the API

### Get all posts
```bash
curl http://localhost:3004/posts
```

### Get posts with specific tag
```bash
curl "http://localhost:3004/posts?tag=tutorial"
```

### Create a new post
```bash
curl -X POST http://localhost:3004/posts \
  -H "Content-Type: application/json" \
  -d '{
    "title": "My New Blog Post",
    "content": "This is the content of my new blog post...",
    "author_name": "New Author",
    "tags": ["example", "api"],
    "published": true
  }'
```

### Get specific post
```bash
curl http://localhost:3004/posts/1
```

### Update a post
```bash
curl -X PUT http://localhost:3004/posts/1 \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated Post Title",
    "content": "Updated content...",
    "published": true
  }'
```

### Get comments for a post
```bash
curl http://localhost:3004/posts/1/comments
```

### Add a comment
```bash
curl -X POST http://localhost:3004/posts/1/comments \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Great post! Thanks for sharing.",
    "author": "Comment Author",
    "author_email": "commenter@example.com"
  }'
```

### Search
```bash
curl "http://localhost:3004/search?q=backworks"
```

### Get authors
```bash
curl http://localhost:3004/authors
```

## 📊 Dashboard

Visit http://localhost:3005 to see:
- Real-time request metrics
- Endpoint usage statistics
- Request/response logs
- API performance data

## 🎯 Learning Points

This example demonstrates:

1. **Path Parameters** - Using `{id}` and `{post_id}` in URLs
2. **Query Parameters** - Filtering and search functionality
3. **HTTP Methods** - GET, POST, PUT, DELETE operations
4. **Input Validation** - Checking required fields and data format
5. **Error Handling** - Returning appropriate status codes and messages
6. **Data Relationships** - Posts have comments and authors
7. **Complex Logic** - Filtering, searching, and data manipulation

## 🔧 Configuration Highlights

### Path Parameters
```yaml
endpoints:
  post_detail:
    path: "/posts/{id}"
    # Access via req.path_params.id
```

### Multiple HTTP Methods
```yaml
endpoints:
  posts:
    methods: ["GET", "POST"]
    # Handle different methods with if/else
```

### Validation Example
```javascript
if (!req.body?.title || !req.body?.content) {
  return {
    status: 400,
    body: { 
      error: 'Validation failed',
      message: 'Title and content are required' 
    }
  };
}
```

## 🚀 Next Steps

1. **Extend the API** - Add user authentication, categories, etc.
2. **Add more validation** - Email format, content length limits
3. **Implement pagination** - Add limit/offset to GET endpoints
4. **Add more endpoints** - Tags management, user profiles
5. **Try other examples** - Check out the [task-manager example](../task-manager/)
