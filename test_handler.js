// Parse request data
const request = JSON.parse(process.argv[2] || '{}');

// Handler function
function handler(req, res) {
  return {
    status: 200,
    headers: { "Content-Type": "application/json" },
    body: {
      message: "Hello, World!",
      timestamp: new Date().toISOString()
    }
  };
}

// Execute handler and output result
try {
    const result = handler(request);
    console.log(JSON.stringify(result));
} catch (error) {
    console.error('Handler error:', error.message);
    process.exit(1);
}
