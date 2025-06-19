# Proxy API Example

This example demonstrates Backworks' proxy capabilities, including:

- **Multiple Target Proxy**: Route requests to different backend services
- **Load Balancing**: Distribute requests across multiple targets with weights
- **Health Checks**: Monitor backend health and route around failures
- **Request Transformation**: Add headers, strip prefixes, and transform requests
- **Circuit Breaker**: Protect against cascading failures
- **Request Capture**: Learn API patterns and schemas automatically

## Usage

```bash
# Start the proxy
../../target/debug/backworks start -c blueprint.yaml

# Test GitHub API proxy
curl http://localhost:3010/github/users/octocat

# Test httpbin proxy with load balancing
curl http://localhost:3010/httpbin/get
curl http://localhost:3010/httpbin/json

# View metrics and health in dashboard
open http://localhost:3011
```

## Features Demonstrated

1. **Path-based Routing**: Different endpoints proxy to different services
2. **Load Balancing**: Weighted round-robin between multiple backends
3. **Health Monitoring**: Automatic health checks and failover
4. **Request Capture**: Learn API schemas and patterns
5. **Resilience**: Retry logic and circuit breakers
