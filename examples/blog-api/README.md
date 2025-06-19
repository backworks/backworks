# Blog API

A complete blog backend created with YAML.

## 🎯 What This Creates

**YAML Configuration** → **Full Blog API**

### Endpoints:
- `GET /posts` - List all blog posts
- `POST /posts` - Create a new post
- `GET /posts/{id}` - Get specific post with comments
- `PUT /posts/{id}` - Update a post
- `DELETE /posts/{id}` - Delete a post
- `GET /posts/{post_id}/comments` - Get comments for a post
- `POST /posts/{post_id}/comments` - Add a comment
- `GET /authors` - List all authors
- `GET /search` - Search posts and comments

## 🚀 Run It

```bash
# From the blog-api directory
backworks start --config api.yaml
```

## 🧪 Test It

```bash
# Get all posts
curl http://localhost:3000/posts

# Get a specific post with comments
curl http://localhost:3000/posts/1

# Create a new post
curl -X POST http://localhost:3000/posts \
  -H "Content-Type: application/json" \
  -d '{"title": "My New Post", "content": "Post content..."}'

# Get authors
curl http://localhost:3000/authors

# Search
curl "http://localhost:3000/search?q=backworks"
```

## 📊 Dashboard

Visit http://localhost:3001 to see:
- Request metrics for each endpoint
- Real-time API usage
- Performance insights

## 💡 Key Features

- **Complex data structures** (nested objects, arrays)
- **Path parameters** (`/posts/{id}`)
- **Multiple HTTP methods** per endpoint
- **Realistic mock data** that looks like a real blog
- **Relationships** between posts, comments, and authors

This demonstrates how Backworks can handle sophisticated APIs with just YAML!
