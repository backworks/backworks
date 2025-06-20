# 📝 Blog API

A comprehensive blog backend API demonstrating **both inline and external handler approaches** with Backworks.

## 🎯 What This Demonstrates

This example shows a realistic blog API with mixed handler strategies:

### 📄 **Inline Handlers**
- **`/posts`** - Simple CRUD operations for blog posts
- **`/authors`** - Static author data (perfect for inline)

### 📁 **External Handlers**
- **`/posts/{id}`** - Complex post operations (`handlers/post-detail.js`)
- **`/posts/{post_id}/comments`** - Comment management (`handlers/comments.js`)
- **`/search`** - Advanced search logic (`handlers/search.js`)

## �️ Project Structure

```
blog-api/
├── package.json              # npm-style project metadata
├── blueprints/
│   └── main.yaml             # API blueprint with mixed handlers
├── handlers/
│   ├── post-detail.js        # Complex CRUD operations
│   ├── comments.js           # Comment management
│   └── search.js             # Advanced search functionality
└── README.md
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
