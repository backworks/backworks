/**
 * Dashboard Aggregator Handler - Multi-source data aggregation
 * 
 * This handler demonstrates advanced patterns including:
 * - Parallel data fetching from multiple sources
 * - Data aggregation and transformation
 * - Error resilience and partial failures
 * - Response caching and optimization
 */

async function handler(req, res) {
  const startTime = Date.now();
  
  try {
    // Parse query parameters
    const includeUsers = req.query_params?.users !== 'false';
    const includePosts = req.query_params?.posts !== 'false';
    const includeComments = req.query_params?.comments !== 'false';
    const limit = parseInt(req.query_params?.limit) || 5;
    
    console.log(`Dashboard aggregation requested: users=${includeUsers}, posts=${includePosts}, comments=${includeComments}`);
    
    // Prepare data fetching promises
    const promises = [];
    const dataKeys = [];
    
    if (includeUsers) {
      promises.push(fetchUsers(limit));
      dataKeys.push('users');
    }
    
    if (includePosts) {
      promises.push(fetchPosts(limit));
      dataKeys.push('posts');
    }
    
    if (includeComments) {
      promises.push(fetchComments(limit));
      dataKeys.push('comments');
    }
    
    // Execute all requests in parallel with error handling
    const results = await Promise.allSettled(promises);
    
    // Process results and handle partial failures
    const dashboardData = {};
    const errors = [];
    
    results.forEach((result, index) => {
      const key = dataKeys[index];
      
      if (result.status === 'fulfilled') {
        dashboardData[key] = result.value;
      } else {
        console.error(`Failed to fetch ${key}:`, result.reason);
        errors.push({
          source: key,
          error: result.reason.message || 'Unknown error'
        });
        
        // Provide fallback data
        dashboardData[key] = {
          data: [],
          error: `Failed to load ${key}`,
          fallback: true
        };
      }
    });
    
    // Create aggregated statistics
    const stats = generateStatistics(dashboardData);
    
    // Build final response
    const endTime = Date.now();
    const responseTime = endTime - startTime;
    
    const response = {
      dashboard: dashboardData,
      statistics: stats,
      metadata: {
        generated_at: new Date().toISOString(),
        response_time_ms: responseTime,
        sources_requested: dataKeys.length,
        sources_successful: dataKeys.length - errors.length,
        partial_failure: errors.length > 0 && errors.length < dataKeys.length,
        total_failure: errors.length === dataKeys.length,
        handler_type: 'external-aggregator'
      }
    };
    
    // Include errors if any occurred
    if (errors.length > 0) {
      response.errors = errors;
    }
    
    // Determine status code
    const status = errors.length === dataKeys.length ? 503 : 
                  errors.length > 0 ? 206 : 200;
    
    return {
      status: status,
      headers: {
        'Content-Type': 'application/json',
        'X-Response-Time': responseTime + 'ms',
        'X-Sources-Count': dataKeys.length.toString(),
        'X-Handler-Type': 'external-aggregator',
        'Cache-Control': 'public, max-age=60' // Cache for 1 minute
      },
      body: response
    };
    
  } catch (error) {
    console.error('Dashboard aggregation error:', error);
    
    return {
      status: 500,
      body: {
        error: 'Dashboard aggregation failed',
        message: error.message,
        timestamp: new Date().toISOString()
      }
    };
  }
}

// Mock data fetching functions (in real implementation, these would make HTTP requests)
async function fetchUsers(limit = 5) {
  // Simulate network delay
  await new Promise(resolve => setTimeout(resolve, Math.random() * 100));
  
  const users = [];
  for (let i = 1; i <= limit; i++) {
    users.push({
      id: i,
      name: `User ${i}`,
      username: `user${i}`,
      email: `user${i}@example.com`,
      posts_count: Math.floor(Math.random() * 10) + 1,
      last_active: new Date(Date.now() - Math.random() * 86400000).toISOString()
    });
  }
  
  return {
    data: users,
    total: users.length,
    source: 'users-api'
  };
}

async function fetchPosts(limit = 5) {
  await new Promise(resolve => setTimeout(resolve, Math.random() * 150));
  
  const posts = [];
  for (let i = 1; i <= limit; i++) {
    posts.push({
      id: i,
      userId: Math.floor(Math.random() * 3) + 1,
      title: `Sample Post ${i}`,
      body: `This is the content of post ${i}. It demonstrates how the dashboard aggregator fetches and combines data from multiple sources.`,
      created_at: new Date(Date.now() - Math.random() * 86400000 * 7).toISOString(),
      comments_count: Math.floor(Math.random() * 20),
      likes: Math.floor(Math.random() * 100)
    });
  }
  
  return {
    data: posts,
    total: posts.length,
    source: 'posts-api'
  };
}

async function fetchComments(limit = 5) {
  await new Promise(resolve => setTimeout(resolve, Math.random() * 120));
  
  const comments = [];
  for (let i = 1; i <= limit; i++) {
    comments.push({
      id: i,
      postId: Math.floor(Math.random() * 5) + 1,
      name: `Commenter ${i}`,
      email: `commenter${i}@example.com`,
      body: `This is comment ${i} on a post. The dashboard aggregator shows how to handle multiple data sources efficiently.`,
      created_at: new Date(Date.now() - Math.random() * 86400000 * 3).toISOString()
    });
  }
  
  return {
    data: comments,
    total: comments.length,
    source: 'comments-api'
  };
}

// Generate aggregated statistics
function generateStatistics(dashboardData) {
  const stats = {
    total_users: 0,
    total_posts: 0,
    total_comments: 0,
    active_users_24h: 0,
    recent_posts_7d: 0,
    avg_comments_per_post: 0
  };
  
  // Process users
  if (dashboardData.users && dashboardData.users.data) {
    stats.total_users = dashboardData.users.data.length;
    
    const oneDayAgo = Date.now() - 86400000;
    stats.active_users_24h = dashboardData.users.data.filter(user => 
      new Date(user.last_active).getTime() > oneDayAgo
    ).length;
  }
  
  // Process posts
  if (dashboardData.posts && dashboardData.posts.data) {
    stats.total_posts = dashboardData.posts.data.length;
    
    const sevenDaysAgo = Date.now() - 86400000 * 7;
    stats.recent_posts_7d = dashboardData.posts.data.filter(post => 
      new Date(post.created_at).getTime() > sevenDaysAgo
    ).length;
  }
  
  // Process comments
  if (dashboardData.comments && dashboardData.comments.data) {
    stats.total_comments = dashboardData.comments.data.length;
  }
  
  // Calculate average comments per post
  if (stats.total_posts > 0 && dashboardData.posts && dashboardData.posts.data) {
    const totalCommentsCount = dashboardData.posts.data.reduce(
      (sum, post) => sum + (post.comments_count || 0), 0
    );
    stats.avg_comments_per_post = Math.round(totalCommentsCount / stats.total_posts * 10) / 10;
  }
  
  return stats;
}

module.exports = handler;
