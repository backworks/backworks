/**
 * Posts Request Transform Handler
 * 
 * This handler demonstrates request transformation
 * specifically for the posts endpoint.
 */

function transform(request) {
  console.log('Transforming posts request:', request.method, request.url);
  
  // Initialize headers if not present
  request.headers = request.headers || {};
  
  // Add posts-specific headers
  request.headers['X-Posts-API'] = 'v1';
  request.headers['X-Content-Source'] = 'jsonplaceholder';
  request.headers['Accept'] = 'application/json';
  
  // Transform based on HTTP method
  switch (request.method) {
    case 'GET':
      // Add query parameters for pagination
      request.query = request.query || {};
      if (!request.query._limit) {
        request.query._limit = '10'; // Default limit
      }
      if (!request.query._sort) {
        request.query._sort = 'id';
        request.query._order = 'desc'; // Most recent first
      }
      break;
      
    case 'POST':
      // Add timestamp to new posts
      if (request.body && typeof request.body === 'object') {
        request.body.created_at = new Date().toISOString();
        request.body.status = 'published';
        request.body.source = 'backworks-proxy';
      }
      request.headers['Content-Type'] = 'application/json';
      break;
      
    case 'PUT':
      // Add update timestamp
      if (request.body && typeof request.body === 'object') {
        request.body.updated_at = new Date().toISOString();
        request.body.modified_by = 'backworks-proxy';
      }
      request.headers['Content-Type'] = 'application/json';
      break;
  }
  
  // Add request tracking
  request.headers['X-Request-ID'] = generateRequestId();
  request.headers['X-Request-Time'] = new Date().toISOString();
  
  console.log('Posts request transformed for method:', request.method);
  
  return request;
}

function generateRequestId() {
  return 'posts_req_' + Date.now() + '_' + Math.random().toString(36).substr(2, 6);
}

module.exports = transform;
