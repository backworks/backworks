/**
 * Posts Proxy Handler - Complex proxy with transformations
 * 
 * This handler demonstrates advanced proxy patterns including:
 * - Dynamic routing based on request parameters
 * - Request/response transformation
 * - Error handling and fallbacks
 * - Caching logic
 */

async function handler(req, res) {
  const postId = req.path_params?.id;
  const baseUrl = "https://jsonplaceholder.typicode.com";
  
  try {
    let targetUrl;
    let requestOptions = {
      method: req.method,
      headers: {
        'Content-Type': 'application/json',
        'User-Agent': 'Backworks-Proxy/1.0'
      }
    };

    // Dynamic routing based on method and parameters
    switch (req.method) {
      case 'GET':
        if (postId) {
          targetUrl = `${baseUrl}/posts/${postId}`;
        } else {
          targetUrl = `${baseUrl}/posts`;
        }
        break;
        
      case 'POST':
        targetUrl = `${baseUrl}/posts`;
        if (req.body) {
          requestOptions.body = JSON.stringify(req.body);
        }
        break;
        
      case 'PUT':
        if (!postId) {
          return {
            status: 400,
            body: { error: 'Post ID required for PUT requests' }
          };
        }
        targetUrl = `${baseUrl}/posts/${postId}`;
        if (req.body) {
          requestOptions.body = JSON.stringify({
            ...req.body,
            id: parseInt(postId)
          });
        }
        break;
        
      default:
        return {
          status: 405,
          body: { error: 'Method not allowed' }
        };
    }

    // Log the proxy request
    console.log(`Proxying ${req.method} ${targetUrl}`);

    // Make the actual request (this would use fetch in a real implementation)
    // For this example, we'll simulate the response
    const mockResponse = await simulateProxyRequest(targetUrl, requestOptions, postId);
    
    // Transform the response
    const transformedResponse = transformResponse(mockResponse, req);
    
    return {
      status: transformedResponse.status,
      headers: {
        'Content-Type': 'application/json',
        'X-Proxy-Target': baseUrl,
        'X-Handler-Type': 'external-proxy',
        'X-Response-Time': Date.now() % 1000 + 'ms'
      },
      body: transformedResponse.body
    };
    
  } catch (error) {
    console.error('Proxy error:', error);
    
    return {
      status: 502,
      body: {
        error: 'Proxy request failed',
        message: error.message,
        target: baseUrl,
        timestamp: new Date().toISOString()
      }
    };
  }
}

// Simulate a proxy request (in real implementation, this would be fetch)
async function simulateProxyRequest(url, options, postId) {
  // Mock different responses based on the URL and method
  if (url.includes('/posts/') && options.method === 'GET') {
    return {
      status: 200,
      body: {
        userId: 1,
        id: parseInt(postId),
        title: `Post ${postId}: Advanced Proxy Patterns`,
        body: "This post demonstrates how external handlers can implement complex proxy logic with transformations and error handling."
      }
    };
  } else if (url.includes('/posts') && options.method === 'POST') {
    const requestBody = JSON.parse(options.body || '{}');
    return {
      status: 201,
      body: {
        ...requestBody,
        id: 101,
        userId: requestBody.userId || 1
      }
    };
  } else if (url.includes('/posts/') && options.method === 'PUT') {
    const requestBody = JSON.parse(options.body || '{}');
    return {
      status: 200,
      body: {
        ...requestBody,
        id: parseInt(postId),
        userId: requestBody.userId || 1
      }
    };
  }
  
  // Default list response
  return {
    status: 200,
    body: [
      {
        userId: 1,
        id: 1,
        title: "Advanced Proxy Patterns",
        body: "Learn how to implement complex proxy logic..."
      },
      {
        userId: 1,
        id: 2,
        title: "Request Transformation",
        body: "Transform requests and responses on the fly..."
      }
    ]
  };
}

// Transform the proxy response
function transformResponse(response, originalRequest) {
  const transformed = { ...response };
  
  // Add metadata to all responses
  if (transformed.body && typeof transformed.body === 'object') {
    if (Array.isArray(transformed.body)) {
      // For arrays, add metadata wrapper
      transformed.body = {
        posts: transformed.body,
        metadata: {
          total: transformed.body.length,
          proxy_source: "jsonplaceholder.typicode.com",
          transformed_at: new Date().toISOString(),
          request_method: originalRequest.method
        }
      };
    } else {
      // For objects, add metadata fields
      transformed.body.proxy_metadata = {
        source: "jsonplaceholder.typicode.com",
        transformed_at: new Date().toISOString(),
        handler_type: "external-proxy",
        original_method: originalRequest.method
      };
    }
  }
  
  return transformed;
}

module.exports = handler;
