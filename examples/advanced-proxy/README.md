# Advanced Proxy Example

This example demonstrates the comprehensive transformation capabilities of Backworks proxy mode.

## Features Demonstrated

### Load Balancing & Health Checks
- Multiple backend targets with different weights
- Health check configuration with custom endpoints
- Circuit breaker and retry logic

### Request Transformations
- **Headers**: Add/remove/modify headers
- **Path Rewriting**: Strip/add prefixes, pattern-based replacements
- **Query Parameters**: Add/remove/rename parameters with defaults
- **Body Transformations**: JSON field manipulation and string replacements

### Response Transformations
- **Status Code Mapping**: Convert status codes for frontend compatibility
- **Header Management**: Add CORS headers, remove sensitive server headers
- **Body Standardization**: Consistent JSON response format

### Capture & Monitoring
- Detailed request/response logging
- Performance metrics collection
- Health check monitoring

## Usage

1. Start the Backworks server:
```bash
cd /path/to/backworks
cargo run -- examples/advanced-proxy/blueprint.yaml
```

2. Test the endpoints:

### Test JSON transformation
```bash
curl http://localhost:3000/test/json
```

### Test path rewriting and query transformations
```bash
curl "http://localhost:3000/posts/1?page=1&size=5&debug=true"
```

### Test POST with body transformation
```bash
curl -X POST http://localhost:3000/posts \
  -H "Content-Type: application/json" \
  -d '{"user_id": 123, "title": "Test Post", "password": "secret123"}'
```

### Test status code mapping
```bash
curl http://localhost:3000/status/404
curl http://localhost:3000/status/500
```

## Expected Transformations

### Request Transformations Applied:
- **Headers**: `X-API-Version: 2.0`, `X-Client: Backworks` added
- **Security**: `Cookie` and sensitive headers removed
- **Paths**: `/api/v1` prefix stripped, `/api/v2` prefix added
- **Query**: `page`→`offset`, `size`→`limit`, `debug` removed
- **Body**: `user_id`→`userId`, `password` field removed, `timestamp` added

### Response Transformations Applied:
- **Status**: 404→200, 500→503 (configurable mapping)
- **Headers**: CORS headers added, server headers removed
- **Body**: Wrapped in standard format with metadata

## Configuration Highlights

The `blueprint.yaml` shows:
- Multi-target proxy configuration
- Comprehensive transformation rules
- Health check and load balancing setup
- Security header management
- Response standardization

This example serves as a template for building production-ready API gateways with Backworks.
