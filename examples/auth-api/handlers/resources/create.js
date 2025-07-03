/**
 * Create a new resource
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
  
  // Extract resource data from request body
  const { name, description } = req.body;
  
  // Basic validation
  if (!name) {
    return {
      status: 400,
      body: { error: "Resource name is required" }
    };
  }
  
  // In a real application, we would create the resource in a database
  // For this example, we'll just return the created resource with a mock ID
  const newResource = {
    id: Math.floor(Math.random() * 1000), // Generate a random ID
    name,
    description,
    createdBy: user.username,
    createdAt: new Date().toISOString()
  };
  
  return {
    status: 201,
    body: {
      message: "Resource created successfully",
      resource: newResource
    }
  };
};
