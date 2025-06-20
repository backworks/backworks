/**
 * Echo Handler - External JavaScript Handler Example
 * 
 * This demonstrates how to use external JavaScript files for handlers
 * instead of inline code in the YAML blueprint.
 */

function handler(req, res) {
  // Log the incoming request for demonstration
  console.log(`Echo endpoint called: ${req.method} ${req.path}`);
  
  // Echo back the request with additional metadata
  return {
    status: 200,
    headers: { 
      "Content-Type": "application/json",
      "X-Handler-Type": "external-js"
    },
    body: {
      echo: req.body,
      metadata: {
        method: req.method,
        path: req.path,
        headers: req.headers,
        timestamp: new Date().toISOString(),
        handler_source: "external-file"
      }
    }
  };
}

module.exports = handler;
