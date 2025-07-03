# Advanced Proxy Example

This example demonstrates advanced proxy capabilities with the Backworks proxy plugin, including:

- **Multiple Backend Targets**: Load balancing across multiple backend services
- **Weighted Round Robin**: Distributes requests based on target weights
- **Circuit Breaker**: Prevents cascading failures when backends become unhealthy
- **Health Checks**: Continuous monitoring of backend service health
- **Request/Response Transformation**: Modify headers and payloads
- **Metrics Collection**: Detailed metrics for monitoring and observability

## Architecture

```
┌─────────────────┐    ┌─────────────────┐
│   Client        │    │   Backworks     │
│   Requests      │───▶│   Proxy         │
│                 │    │   Plugin        │
└─────────────────┘    └─────────────────┘
                                │
                   ┌────────────┼────────────┐
                   │            │            │
                   ▼            ▼            ▼
            ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
            │ Backend-1   │ │ Backend-2   │ │ Backend-3   │
            │ Weight: 1.0 │ │ Weight: 2.0 │ │ Weight: 1.0 │
            │ Port: 3001  │ │ Port: 3002  │ │ Port: 3003  │
            └─────────────┘ └─────────────┘ └─────────────┘
```

## Configuration

The proxy plugin is configured with:

### Load Balancing
- **Algorithm**: Weighted Round Robin
- **Weights**: Backend-2 receives 2x more requests than Backend-1 and Backend-3

### Circuit Breaker
- **Failure Threshold**: 5 consecutive failures trigger circuit breaker
- **Recovery Timeout**: 60 seconds before attempting recovery
- **Half-Open Calls**: 3 test calls during recovery phase

### Health Checks
- **Interval**: Every 30 seconds
- **Timeout**: 5 seconds per check
- **Healthy Threshold**: 2 consecutive successes mark target as healthy
- **Unhealthy Threshold**: 3 consecutive failures mark target as unhealthy

### Request/Response Transformation
- **Request Headers**: Added `X-Proxy-Version`, `X-Load-Balancer`, `X-Request-Source`
- **Response Headers**: Added `X-Proxied-By`

## Endpoints

### User Service
- `GET /api/users` - List all users
- `GET /api/users/{id}` - Get user by ID
- `POST /api/users` - Create new user

### Order Service
- `GET /api/orders` - List all orders
- `GET /api/orders/{id}` - Get order by ID
- `POST /api/orders` - Create new order

### Product Service
- `GET /api/products` - List all products
- `GET /api/products/{id}` - Get product by ID

### System Endpoints
- `GET /health` - Health check endpoint
- `GET /metrics` - Proxy metrics and status

## Running the Example

### Prerequisites

1. **Start Backend Services** (simulate with simple HTTP servers):

```bash
# Terminal 1 - Backend 1
python3 -m http.server 3001 --directory /tmp/backend1

# Terminal 2 - Backend 2  
python3 -m http.server 3002 --directory /tmp/backend2

# Terminal 3 - Backend 3
python3 -m http.server 3003 --directory /tmp/backend3
```

2. **Create Mock Health Endpoints**:

```bash
# Create health endpoints for each backend
mkdir -p /tmp/backend1 /tmp/backend2 /tmp/backend3

echo '{"status": "healthy", "service": "backend-1"}' > /tmp/backend1/health
echo '{"status": "healthy", "service": "backend-2"}' > /tmp/backend2/health
echo '{"status": "healthy", "service": "backend-3"}' > /tmp/backend3/health
```

### Start the Proxy

```bash
# Start Backworks with the proxy example
backworks start

# Or with hot reload for development
backworks start --watch
```

### Testing

#### Test Load Balancing
```bash
# Make multiple requests to see load balancing in action
for i in {1..10}; do
  curl -H "X-Request-ID: $i" http://localhost:8080/api/users
  echo ""
done
```

#### Test Health Checks
```bash
# Check proxy health
curl http://localhost:8080/health

# Check detailed metrics
curl http://localhost:8080/metrics
```

#### Test Circuit Breaker
```bash
# Stop one backend to trigger circuit breaker
# Then make requests to see failover behavior
curl -v http://localhost:8080/api/users
```

### Monitoring

Access the Backworks dashboard at http://localhost:8081 to monitor:
- Request distribution across backends
- Circuit breaker states
- Health check status
- Response times and error rates
- Target-specific metrics

## Key Features Demonstrated

1. **High Availability**: Automatic failover when backends become unhealthy
2. **Performance**: Weighted load balancing optimizes resource utilization
3. **Resilience**: Circuit breaker prevents cascading failures
4. **Observability**: Comprehensive metrics and monitoring
5. **Flexibility**: Request/response transformation for integration

## Customization

You can modify the `blueprints/main.yaml` file to:
- Add more backend targets
- Adjust load balancing weights
- Configure different circuit breaker thresholds
- Customize health check parameters
- Add request/response transformations
- Enable additional metrics

## Production Considerations

For production use, consider:
- Using service discovery for dynamic backend registration
- Implementing TLS termination
- Adding authentication/authorization
- Configuring persistent health check storage
- Setting up monitoring and alerting
- Implementing rate limiting
- Adding request caching
