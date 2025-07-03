/**
 * Validate a token
 * 
 * This endpoint is protected by the authentication middleware,
 * so if we reach this handler, the token is already valid.
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
  
  return {
    status: 200,
    body: {
      message: "Token is valid",
      user
    }
  };
};
