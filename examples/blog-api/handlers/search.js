/**
 * Search Handler - Advanced search functionality
 * 
 * This external handler demonstrates complex search logic with filtering,
 * ranking, and aggregation that would be difficult to maintain inline.
 */

function handler(req, res) {
  const query = req.query_params?.q || '';
  const type = req.query_params?.type || 'all'; // 'posts', 'comments', or 'all'
  const limit = parseInt(req.query_params?.limit) || 10;
  
  if (!query) {
    return {
      status: 400,
      body: { 
        error: 'Search query required',
        message: 'Use ?q=search_term to search',
        examples: [
          '/search?q=backworks',
          '/search?q=tutorial&type=posts',
          '/search?q=helpful&type=comments&limit=5'
        ]
      }
    };
  }
  
  if (query.length < 2) {
    return {
      status: 400,
      body: {
        error: 'Query too short',
        message: 'Search query must be at least 2 characters long'
      }
    };
  }
  
  // Mock search implementation with relevance scoring
  const calculateRelevance = (text, searchQuery) => {
    const lowerText = text.toLowerCase();
    const lowerQuery = searchQuery.toLowerCase();
    
    // Exact match gets highest score
    if (lowerText.includes(lowerQuery)) {
      const position = lowerText.indexOf(lowerQuery);
      // Earlier in text = higher relevance
      return Math.max(0.5, 1 - (position / text.length));
    }
    
    // Word match gets medium score
    const words = lowerQuery.split(' ');
    const matchedWords = words.filter(word => lowerText.includes(word));
    return matchedWords.length / words.length * 0.7;
  };
  
  const mockPosts = [
    {
      id: 1,
      title: "Getting Started with Backworks",
      excerpt: "Backworks makes it easy to create APIs from YAML configuration...",
      author: "John Doe",
      created_at: "2025-06-19T10:00:00Z",
      tags: ["tutorial", "getting-started"]
    },
    {
      id: 2,
      title: "Advanced YAML Patterns",
      excerpt: "Learn how to structure complex APIs using advanced patterns...",
      author: "Jane Smith",
      created_at: "2025-06-19T09:00:00Z",
      tags: ["advanced", "yaml"]
    },
    {
      id: 3,
      title: "Building Real-time APIs",
      excerpt: "Explore how to build real-time APIs with Backworks...",
      author: "John Doe",
      created_at: "2025-06-19T08:00:00Z",
      tags: ["real-time", "integration"]
    }
  ];
  
  const mockComments = [
    {
      id: 1,
      content: "Great tutorial! This really helped me understand how Backworks works.",
      post_title: "Getting Started with Backworks",
      author: "Jane Smith",
      created_at: "2025-06-19T11:00:00Z"
    },
    {
      id: 2,
      content: "Very helpful guide for beginners!",
      post_title: "Getting Started with Backworks",
      author: "Bob Wilson",
      created_at: "2025-06-19T11:30:00Z"
    },
    {
      id: 3,
      content: "Advanced patterns are exactly what I needed.",
      post_title: "Advanced YAML Patterns",
      author: "Alice Johnson",
      created_at: "2025-06-19T12:00:00Z"
    }
  ];
  
  let results = { posts: [], comments: [] };
  
  // Search posts
  if (type === 'all' || type === 'posts') {
    results.posts = mockPosts
      .map(post => ({
        ...post,
        relevance: Math.max(
          calculateRelevance(post.title, query),
          calculateRelevance(post.excerpt, query),
          calculateRelevance(post.tags.join(' '), query)
        )
      }))
      .filter(post => post.relevance > 0)
      .sort((a, b) => b.relevance - a.relevance)
      .slice(0, limit);
  }
  
  // Search comments
  if (type === 'all' || type === 'comments') {
    results.comments = mockComments
      .map(comment => ({
        ...comment,
        relevance: calculateRelevance(comment.content, query)
      }))
      .filter(comment => comment.relevance > 0)
      .sort((a, b) => b.relevance - a.relevance)
      .slice(0, limit);
  }
  
  const totalResults = results.posts.length + results.comments.length;
  const searchTime = Math.floor(Math.random() * 50) + 10; // Mock search time
  
  return {
    status: 200,
    headers: {
      'X-Search-Time-Ms': searchTime.toString(),
      'X-Total-Results': totalResults.toString(),
      'X-Handler-Source': 'external-file'
    },
    body: {
      query: query,
      type: type,
      results: results,
      meta: {
        total_results: totalResults,
        search_time_ms: searchTime,
        has_more: totalResults >= limit,
        suggestions: totalResults === 0 ? ['tutorial', 'backworks', 'api'] : []
      }
    }
  };
}

module.exports = handler;
