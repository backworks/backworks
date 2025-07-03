/**
 * Login with username and password
 */
module.exports = async function(req, context) {
  const plugin = context.plugins.auth;

  if (!plugin) {
    return {
      status: 500,
      body: { error: "Auth plugin not available" }
    };
  }

  try {
    // Extract credentials from request body
    const { username, password } = req.body;
    
    // Basic validation
    if (!username || !password) {
      return {
        status: 400,
        body: { error: "Username and password are required" }
      };
    }
    
    // Process the login request
    const result = await plugin.process_auth({
      action: "login",
      username,
      password
    });

    if (!result.success) {
      // Don't provide specific error details to prevent username enumeration
      return {
        status: 401,
        body: { error: "Invalid username or password" }
      };
    }

    // Return the result (includes user and token)
    return {
      status: 200,
      body: {
        message: "Login successful",
        user: result.user,
        token: result.token
      }
    };
  } catch (error) {
    context.log.error(`Login error: ${error.message}`);
    return {
      status: 500,
      body: { error: "An error occurred during login" }
    };
  }
};
