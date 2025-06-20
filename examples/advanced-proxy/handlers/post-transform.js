/**
 * Post Transform Handler
 * 
 * This handler demonstrates post-specific transformations
 * for JSONPlaceholder posts API.
 */

function transform(response) {
  console.log('Transforming post response:', response.status);
  
  // Add custom headers
  response.headers = response.headers || {};
  response.headers['X-Post-Transformer'] = 'active';
  response.headers['X-Content-Type'] = 'enhanced-post';
  response.headers['Cache-Control'] = 'public, max-age=300';
  
  // Transform post data
  if (response.body && typeof response.body === 'object') {
    const post = response.body;
    
    // Enhance post with computed fields
    const enhancedPost = {
      ...post,
      // Add computed metadata
      word_count: post.body ? post.body.split(' ').length : 0,
      reading_time: post.body ? Math.ceil(post.body.split(' ').length / 200) : 0,
      excerpt: post.body ? post.body.substring(0, 150) + '...' : '',
      
      // Add URLs
      urls: {
        self: `/posts/${post.id}`,
        user: `/users/${post.userId}`,
        comments: `/posts/${post.id}/comments`
      },
      
      // Add engagement metrics (simulated)
      engagement: {
        views: Math.floor(Math.random() * 1000) + 100,
        likes: Math.floor(Math.random() * 50) + 5,
        shares: Math.floor(Math.random() * 20) + 1,
        comments_count: Math.floor(Math.random() * 15) + 2
      },
      
      // Add categorization
      category: categorizePost(post.title, post.body),
      
      // Add timestamps
      created_at: new Date(Date.now() - Math.random() * 86400000 * 30).toISOString(),
      updated_at: new Date(Date.now() - Math.random() * 86400000 * 7).toISOString()
    };
    
    // Wrap in response envelope
    response.body = {
      post: enhancedPost,
      meta: {
        transformed: true,
        transformer: 'post-transform-handler',
        transformation_time: new Date().toISOString(),
        enhancements: [
          'word_count',
          'reading_time',
          'excerpt',
          'urls',
          'engagement_metrics',
          'categorization',
          'timestamps'
        ]
      }
    };
  }
  
  console.log('Post transformation completed');
  
  return response;
}

// Helper function to categorize posts
function categorizePost(title, body) {
  const text = (title + ' ' + body).toLowerCase();
  
  if (text.includes('tech') || text.includes('computer') || text.includes('software')) {
    return 'technology';
  } else if (text.includes('business') || text.includes('market') || text.includes('finance')) {
    return 'business';
  } else if (text.includes('health') || text.includes('medical') || text.includes('fitness')) {
    return 'health';
  } else if (text.includes('travel') || text.includes('vacation') || text.includes('trip')) {
    return 'travel';
  } else if (text.includes('food') || text.includes('recipe') || text.includes('cooking')) {
    return 'food';
  } else {
    return 'general';
  }
}

module.exports = transform;
