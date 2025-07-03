/**
 * Update a resource by ID
 * 
 * This endpoint is protected by the authentication and admin middleware,
 * so only authenticated users with the admin role can access it.
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
  
  // Additional role check (even though middleware should have already handled this)
  if (!user.roles.includes('admin')) {
    return {
      status: 403,
      body: { error: "Insufficient permissions. Admin role required." }
    };
  }
  
  // Extract resource ID from path parameter and data from body
  const resourceId = parseInt(req.params.id);
  const { name, description } = req.body;
  
  // Validate ID
  if (isNaN(resourceId)) {
    return {
      status: 400,
      body: { error: "Invalid resource ID" }
    };
  }
  
  // Basic validation
  if (!name) {
    return {
      status: 400,
      body: { error: "Resource name is required" }
    };
  }
  
  // In a real application, we would fetch and update the resource in a database
  // For this example, we'll simulate a lookup and update based on the ID
  const resources = {
    1: { id: 1, name: "Resource 1", description: "Description for resource 1" },
    2: { id: 2, name: "Resource 2", description: "Description for resource 2" },
    3: { id: 3, name: "Resource 3", description: "Description for resource 3" }
  };
  
  if (!resources[resourceId]) {
    return {
      status: 404,
      body: { error: `Resource with ID ${resourceId} not found` }
    };
  }
  
  // Update the resource
  const updatedResource = {
    id: resourceId,
    name,
    description: description || resources[resourceId].description,
    updatedBy: user.username,
    updatedAt: new Date().toISOString()
  };
  
  return {
    status: 200,
    body: {
      message: "Resource updated successfully",
      resource: updatedResource
    }
  };
};
