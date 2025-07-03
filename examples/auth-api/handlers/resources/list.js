/**
 * List all resources
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
  
  // In a real application, we would fetch resources from a database
  // For this example, we'll return some mock data
  const resources = [
    { id: 1, name: "Resource 1", description: "Description for resource 1" },
    { id: 2, name: "Resource 2", description: "Description for resource 2" },
    { id: 3, name: "Resource 3", description: "Description for resource 3" }
  ];
  
  return {
    status: 200,
    body: {
      resources,
      user: {
        username: user.username,
        roles: user.roles
      }
    }
  };
};
