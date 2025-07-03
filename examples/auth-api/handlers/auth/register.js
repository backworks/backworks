/**
 * Register a new user
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
    // Extract user data from request body
    const { username, password, email, full_name } = req.body;
    
    // Basic validation
    if (!username || !password) {
      return {
        status: 400,
        body: { error: "Username and password are required" }
      };
    }
    
    // Process the registration request
    const result = await plugin.process_auth({
      action: "register",
      username,
      password,
      email,
      full_name
    });

    if (!result.success) {
      context.log.error(`Registration error: ${result.error}`);
      return {
        status: 400,
        body: { error: result.error || "Registration failed" }
      };
    }

    // Return the result (includes user and token)
    return {
      status: 201,
      body: {
        message: "User registered successfully",
        user: result.user,
        token: result.token
      }
    };
  } catch (error) {
    context.log.error(`Registration error: ${error.message}`);
    return {
      status: 500,
      body: { error: error.message }
    };
  }
};
