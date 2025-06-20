/**
 * Request Transform Handler
 * 
 * This handler demonstrates how to transform outgoing requests
 * before they are sent to the target server.
 */

function transform(request) {
  console.log('Transforming outgoing request:', request.method, request.url);
  
  // Add authentication headers
  request.headers = request.headers || {};
  request.headers['Authorization'] = 'Bearer demo-token';
  request.headers['X-API-Key'] = 'backworks-proxy';
  request.headers['X-Request-ID'] = generateRequestId();
  request.headers['X-Forwarded-By'] = 'backworks-proxy';
  
  // Add timestamp
  request.headers['X-Request-Time'] = new Date().toISOString();
  
  // Transform query parameters
  if (request.query) {
    // Add tracking parameters
    request.query.source = 'backworks';
    request.query.version = '1.0';
  }
  
  // Log transformed request
  console.log('Request transformed, headers added:', Object.keys(request.headers).length);
  
  return request;
}

function generateRequestId() {
  return 'req_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
}

module.exports = transform;
