/**
 * Get a resource by ID
 * 
 * This endpoint is protected by the authentication middleware,
 * so only authenticated users can access it.
 */
module.exports = async function(req, context) {
  // The authentication middleware already verified the token
  // and added the user to the request
  const user = req.user;
  
  if (!user) {
    return {
      status: 401,
      body: { error: "Not authenticated" }
    };
  }
  
  // Extract resource ID from path parameter
  const resourceId = parseInt(req.params.id);
  
  // Validate ID
  if (isNaN(resourceId)) {
    return {
      status: 400,
      body: { error: "Invalid resource ID" }
    };
  }
  
  // In a real application, we would fetch the resource from a database
  // For this example, we'll simulate a lookup based on the ID
  const resources = {
    1: { id: 1, name: "Resource 1", description: "Description for resource 1" },
    2: { id: 2, name: "Resource 2", description: "Description for resource 2" },
    3: { id: 3, name: "Resource 3", description: "Description for resource 3" }
  };
  
  const resource = resources[resourceId];
  
  if (!resource) {
    return {
      status: 404,
      body: { error: `Resource with ID ${resourceId} not found` }
    };
  }
  
  return {
    status: 200,
    body: {
      resource
    }
  };
};
