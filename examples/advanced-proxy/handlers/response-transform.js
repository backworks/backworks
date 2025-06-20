/**
 * Response Transform Handler
 * 
 * This handler demonstrates how to transform responses
 * from the target server before sending them to the client.
 */

function transform(response) {
  console.log('Transforming response:', response.status);
  
  // Transform headers
  response.headers = response.headers || {};
  response.headers['X-Transformed'] = 'true';
  response.headers['X-Transform-Type'] = 'external';
  response.headers['X-Transform-Time'] = new Date().toISOString();
  response.headers['X-Proxy-Engine'] = 'backworks';
  
  // Remove sensitive headers
  delete response.headers['server'];
  delete response.headers['x-powered-by'];
  
  // Transform body if it's JSON
  if (response.body && typeof response.body === 'object') {
    // Add transformation metadata
    response.body.transform_info = {
      transformed_at: new Date().toISOString(),
      transformer: 'external-handler',
      proxy_version: '1.0.0'
    };
    
    // Add computed fields based on content
    if (response.body.args) {
      // HTTPBin response - add analysis
      response.body.request_analysis = {
        total_args: Object.keys(response.body.args).length,
        has_headers: !!response.body.headers,
        origin_ip: response.body.origin,
        url_analyzed: response.body.url
      };
    }
    
    // Wrap original data
    const originalBody = { ...response.body };
    delete originalBody.transform_info;
    
    response.body = {
      success: true,
      data: originalBody,
      metadata: {
        transformed_at: new Date().toISOString(),
        transformer: 'external-response-handler',
        processing_time: Math.random() * 100, // Simulate processing time
        cache_ttl: 300
      }
    };
  }
  
  console.log('Response transformed successfully');
  
  return response;
}

module.exports = transform;
