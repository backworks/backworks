// Test script to verify structured response handling
const request_data = {
  method: "GET",
  path_params: {},
  query_params: {},
  headers: {},
  body: null
};

function handler(req, res) {
  return {
    status: 201,
    headers: { "Content-Type": "application/json" },
    body: {
      message: "Testing structured response",
      status_code: 201,
      success: true
    }
  };
}

// Test the handler
const result = handler(request_data, {});
console.log(JSON.stringify(result, null, 2));
