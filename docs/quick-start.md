# ⚡ Quick Start Guide

Get your first Backworks API running in under 5 minutes!

## 🚀 Installation

### Option 1: Using Cargo (Recommended)
```bash
cargo install backworks
```

### Option 2: From Source
```bash
git clone https://github.com/devstroop/backworks
cd backworks
cargo build --release
./target/release/backworks --version
```

### Option 3: Docker
```bash
docker pull backworks/backworks:latest
```

## 🎯 Your First API in 30 Seconds

### 1. Create Configuration
Create a `project.yaml` file:

```yaml
name: "my_first_api"
description: "A simple user management API"

endpoints:
  users:
    path: "/users"
    methods: ["GET", "POST"]
    mock:
      data:
        - id: 1
          name: "John Doe"
          email: "john@example.com"
        - id: 2
          name: "Jane Smith"
          email: "jane@example.com"
          
  user_detail:
    path: "/users/{id}"
    methods: ["GET", "PUT", "DELETE"]
    mock:
      data:
        id: "${path.id}"
        name: "User ${path.id}"
        email: "user${path.id}@example.com"
```

### 2. Start Your API
```bash
backworks start
```

### 3. Test Your API
```bash
# Get all users
curl http://localhost:8080/users

# Get specific user
curl http://localhost:8080/users/1

# Create new user
curl -X POST http://localhost:8080/users \
  -H "Content-Type: application/json" \
  -d '{"name": "New User", "email": "new@example.com"}'
```

### 4. View Dashboard
Open your browser to `http://localhost:3000` to see the visual dashboard!

## 🔄 Evolution Modes

### Mock Mode (Default)
Perfect for prototyping and frontend development:

```yaml
name: "prototype_api"
mode: "mock"

endpoints:
  products:
    path: "/products"
    mock:
      data: "./data/products.json"
```

### Capture Mode
Learn from existing API usage:

```yaml
name: "learning_api"
mode: "capture"

listeners:
  http:
    port: 8080
    capture_all: true
    
capture_settings:
  analyze_patterns: true
  generate_schemas: true
  export_format: "openapi"
```

### Runtime Mode
Add custom business logic:

```yaml
name: "business_api"
mode: "runtime"

endpoints:
  complex_calculation:
    path: "/calculate"
    runtime:
      language: "javascript"
      handler: |
        export default async (request, context) => {
          const { numbers } = request.body;
          const sum = numbers.reduce((a, b) => a + b, 0);
          return { result: sum, count: numbers.length };
        }
```

## 🤖 Adding AI Intelligence

Enable AI features to enhance your API:

```yaml
name: "intelligent_api"
ai:
  enabled: true
  features:
    - "pattern_recognition"
    - "schema_prediction"
    - "performance_insights"
    - "anomaly_detection"

endpoints:
  smart_endpoint:
    path: "/smart"
    ai_enhanced: true
    mock:
      ai_generated: true
      based_on_patterns: true
```

## 🔌 External API Integration

Connect to third-party services:

```yaml
name: "integration_api"

# Define external APIs
apis:
  stripe:
    base_url: "https://api.stripe.com/v1"
    authentication:
      type: "bearer"
      token_env: "STRIPE_SECRET_KEY"
      
  sendgrid:
    base_url: "https://api.sendgrid.com/v3"
    authentication:
      type: "bearer"
      token_env: "SENDGRID_API_KEY"

endpoints:
  process_payment:
    path: "/payments"
    runtime:
      language: "javascript"
      handler: |
        export default async (request, context) => {
          const { amount, email } = request.body;
          
          // Create Stripe payment
          const payment = await context.apis.stripe.post('/charges', {
            amount: amount * 100,
            currency: 'usd',
            source: request.body.token
          });
          
          // Send confirmation email
          await context.apis.sendgrid.post('/mail/send', {
            personalizations: [{ to: [{ email }] }],
            from: { email: 'noreply@company.com' },
            subject: 'Payment Confirmation',
            content: [{ 
              type: 'text/html', 
              value: `Payment of $${amount} processed successfully!` 
            }]
          });
          
          return { success: true, payment_id: payment.id };
        }
```

## 📊 Database Integration

Connect to databases:

```yaml
name: "database_api"

# Database configuration
database:
  type: "postgresql"
  connection_string_env: "DATABASE_URL"

endpoints:
  users_db:
    path: "/users"
    methods: ["GET", "POST", "PUT", "DELETE"]
    database:
      table: "users"
      # Automatic CRUD operations
      auto_crud: true
      
  custom_query:
    path: "/analytics"
    database:
      query: |
        SELECT 
          DATE(created_at) as date,
          COUNT(*) as user_count
        FROM users 
        WHERE created_at >= NOW() - INTERVAL '30 days'
        GROUP BY DATE(created_at)
        ORDER BY date
```

## 🎨 Visual Dashboard Features

The dashboard automatically provides:

### Real-Time Flow Diagrams
- Request flow visualization
- Performance bottleneck identification
- Error propagation tracking

### AI Insights
- Usage pattern analysis
- Performance optimization suggestions
- Anomaly detection alerts

### Architecture Overview
- System topology visualization
- Dependency mapping
- Health status monitoring

## 📈 Progressive Enhancement

### Phase 1: Rapid Prototyping
```yaml
# Start with mocks
name: "ecommerce_prototype"
endpoints:
  products: { path: "/products", mock: { data: "./products.json" } }
  orders: { path: "/orders", mock: { data: [] } }
```

### Phase 2: Add Intelligence
```yaml
# Enable AI
ai: { enabled: true }
```

### Phase 3: Custom Logic
```yaml
# Add handlers
endpoints:
  orders:
    path: "/orders"
    runtime:
      language: "python"
      handler: "./handlers/orders.py"
```

### Phase 4: Production Ready
```yaml
# Connect to real systems
database: { connection_string_env: "DATABASE_URL" }
apis:
  stripe: { auth: "bearer:${STRIPE_KEY}" }
  inventory: { base_url: "https://inventory.company.com" }
```

## 🔧 Common Patterns

### REST API with CRUD
```yaml
endpoints:
  users:
    path: "/users"
    methods: ["GET", "POST"]
    database: { table: "users", auto_crud: true }
    
  user_detail:
    path: "/users/{id}"
    methods: ["GET", "PUT", "DELETE"]
    database: { table: "users", auto_crud: true }
```

### Microservices Gateway
```yaml
endpoints:
  user_service:
    path: "/users/*"
    proxy:
      target: "http://user-service:8081"
      strip_prefix: "/users"
      
  order_service:
    path: "/orders/*"
    proxy:
      target: "http://order-service:8082"
```

### API Aggregation
```yaml
endpoints:
  dashboard_data:
    path: "/dashboard"
    runtime:
      language: "javascript"
      handler: |
        export default async (request, context) => {
          const [users, orders, analytics] = await Promise.all([
            context.apis.userService.get('/users/count'),
            context.apis.orderService.get('/orders/recent'),
            context.apis.analytics.get('/metrics/daily')
          ]);
          
          return { users, orders, analytics };
        }
```

## 🚀 Next Steps

1. **Explore Examples** - Check the [examples](../examples/) directory
2. **Read Configuration Reference** - Learn all [configuration options](./configuration.md)
3. **Try AI Features** - Explore [AI capabilities](./ai-features.md)
4. **Set Up Monitoring** - Configure [dashboard and monitoring](./monitoring.md)
5. **Advanced Usage** - Dive into [advanced patterns](./advanced.md)

## 🤝 Getting Help

- 📚 **Documentation**: Browse the full [documentation](./README.md)
- 💬 **Community**: Join our [Discord server](https://discord.gg/backworks)
- 🐛 **Issues**: Report bugs on [GitHub](https://github.com/devstroop/backworks/issues)
- 💡 **Features**: Request features on [GitHub Discussions](https://github.com/devstroop/backworks/discussions)

Start building amazing APIs with Backworks today! 🚀
