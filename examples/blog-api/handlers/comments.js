/**
 * Comments Handler - Post comments management
 * 
 * This external handler manages comments for blog posts.
 * It demonstrates validation, filtering, and response formatting.
 */

function handler(req, res) {
  const postId = parseInt(req.path_params.post_id);
  
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
  
  // Mock comments data
  const comments = [
    {
      id: 1,
      content: "Great tutorial! This really helped me understand how Backworks works.",
      author: "Jane Smith",
      author_email: "jane@example.com",
      post_id: postId,
      created_at: "2025-06-19T11:00:00Z",
      likes: 5,
      replies: 2
    },
    {
      id: 2,
      content: "Very helpful, thanks for sharing this knowledge!",
      author: "Bob Wilson",
      author_email: "bob@example.com",
      post_id: postId,
      created_at: "2025-06-19T11:30:00Z",
      likes: 3,
      replies: 0
    },
    {
      id: 3,
      content: "Looking forward to more tutorials like this.",
      author: "Alice Johnson",
      author_email: "alice@example.com",
      post_id: postId,
      created_at: "2025-06-19T12:00:00Z",
      likes: 2,
      replies: 1
    }
  ];
  
  switch (req.method) {
    case 'GET':
      // Support pagination
      const page = parseInt(req.query_params?.page) || 1;
      const limit = parseInt(req.query_params?.limit) || 10;
      const offset = (page - 1) * limit;
      
      const paginatedComments = comments.slice(offset, offset + limit);
      
      return {
        status: 200,
        headers: {
          'X-Total-Comments': comments.length.toString(),
          'X-Page': page.toString(),
          'X-Handler-Source': 'external-file'
        },
        body: {
          comments: paginatedComments,
          pagination: {
            current_page: page,
            per_page: limit,
            total_comments: comments.length,
            total_pages: Math.ceil(comments.length / limit)
          },
          post_id: postId
        }
      };
      
    case 'POST':
      // Validate required fields
      if (!req.body?.content || !req.body?.author) {
        return {
          status: 400,
          body: { 
            error: 'Validation failed',
            message: 'Content and author are required',
            required_fields: ['content', 'author']
          }
        };
      }
      
      // Validate content length
      if (req.body.content.length < 5) {
        return {
          status: 400,
          body: {
            error: 'Content too short',
            message: 'Comment content must be at least 5 characters long'
          }
        };
      }
      
      if (req.body.content.length > 1000) {
        return {
          status: 400,
          body: {
            error: 'Content too long',
            message: 'Comment content must not exceed 1000 characters'
          }
        };
      }
      
      // Generate email if not provided
      const generateEmail = (name) => {
        return name.toLowerCase().replace(/\s+/g, '.') + '@example.com';
      };
      
      const newComment = {
        id: comments.length + 1,
        content: req.body.content.trim(),
        author: req.body.author.trim(),
        author_email: req.body.author_email || generateEmail(req.body.author),
        post_id: postId,
        created_at: new Date().toISOString(),
        likes: 0,
        replies: 0
      };
      
      return {
        status: 201,
        headers: {
          'Location': `/posts/${postId}/comments/${newComment.id}`,
          'X-Handler-Source': 'external-file'
        },
        body: {
          message: 'Comment added successfully',
          comment: newComment
        }
      };
      
    default:
      return {
        status: 405,
        body: {
          error: 'Method not allowed',
          allowed_methods: ['GET', 'POST']
        }
      };
  }
}

module.exports = handler;
