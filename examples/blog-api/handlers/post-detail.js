/**
 * Post Detail Handler - Complex CRUD operations
 * 
 * This external handler demonstrates more complex logic for individual post operations.
 * Using an external file makes it easier to test and maintain.
 */

function handler(req, res) {
  const postId = parseInt(req.path_params.id);
  
  // Validate post ID
  if (isNaN(postId) || postId <= 0) {
    return {
      status: 400,
      body: { 
        error: 'Invalid post ID',
        message: 'Post ID must be a positive integer'
      }
    };
  }
  
  // Mock post data - in a real app, this would come from a database
  const post = {
    id: postId,
    title: `Post ${postId}: Getting Started with Backworks`,
    content: "Backworks makes it easy to create APIs from YAML configuration files. This comprehensive tutorial will guide you through creating your first API, setting up endpoints, and handling requests and responses.",
    author: {
      id: 1,
      name: "John Doe",
      email: "john@example.com"
    },
    created_at: "2025-06-19T10:00:00Z",
    updated_at: "2025-06-19T10:30:00Z",
    tags: ["tutorial", "getting-started"],
    published: true,
    view_count: Math.floor(Math.random() * 1000),
    meta: {
      handler_type: "external",
      last_modified: new Date().toISOString()
    }
  };
  
  switch (req.method) {
    case 'GET':
      return {
        status: 200,
        headers: {
          'Content-Type': 'application/json',
          'X-Handler-Source': 'external-file'
        },
        body: post
      };
      
    case 'PUT':
      if (!req.body) {
        return {
          status: 400,
          body: { 
            error: 'Request body required',
            message: 'PUT requests must include a request body'
          }
        };
      }
      
      // Validate required fields
      const allowedFields = ['title', 'content', 'tags', 'published'];
      const invalidFields = Object.keys(req.body).filter(key => !allowedFields.includes(key));
      
      if (invalidFields.length > 0) {
        return {
          status: 400,
          body: {
            error: 'Invalid fields',
            message: `Invalid fields: ${invalidFields.join(', ')}`,
            allowed_fields: allowedFields
          }
        };
      }
      
      const updatedPost = {
        ...post,
        title: req.body.title || post.title,
        content: req.body.content || post.content,
        tags: req.body.tags || post.tags,
        published: req.body.published !== undefined ? req.body.published : post.published,
        updated_at: new Date().toISOString()
      };
      
      return {
        status: 200,
        body: {
          message: 'Post updated successfully',
          post: updatedPost
        }
      };
      
    case 'DELETE':
      return {
        status: 204,
        headers: {
          'X-Deleted-Post-ID': postId.toString()
        },
        body: null
      };
      
    default:
      return {
        status: 405,
        body: {
          error: 'Method not allowed',
          allowed_methods: ['GET', 'PUT', 'DELETE']
        }
      };
  }
}

module.exports = handler;
