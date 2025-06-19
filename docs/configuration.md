# 📝 Configuration Reference

Complete reference for the `project.yaml` configuration file.

## 🎯 Basic Structure

```yaml
# Required fields
name: "string"                    # API name
version: "string"                 # API version (optional)
description: "string"             # API description (optional)

# Core configuration
mode: "string"                    # Execution mode
endpoints: {}                     # API endpoints definition
ai: {}                           # AI enhancement settings
dashboard: {}                    # Visual dashboard settings

# Optional integrations
database: {}                     # Database configuration
apis: {}                         # External API definitions
cache: {}                        # Caching configuration
security: {}                     # Security settings
monitoring: {}                   # Monitoring and logging
```

## 🔄 Execution Modes

```yaml
mode: "mock"                     # Default: Mock responses
mode: "capture"                  # Capture and analyze requests
mode: "runtime"                  # Execute custom handlers
mode: "database"                 # Database-driven responses
mode: "proxy"                    # Proxy to other services
mode: "hybrid"                   # Mix of multiple modes
mode: "evolving"                 # Auto-evolve between modes
```

## 🛠️ Endpoints Configuration

### Basic Endpoint
```yaml
endpoints:
  endpoint_name:
    path: "/api/path"                    # Required: URL path
    methods: ["GET", "POST"]             # HTTP methods (default: ["GET"])
    description: "Endpoint description"   # Optional description
```

### Mock Mode Endpoint
```yaml
endpoints:
  users:
    path: "/users"
    methods: ["GET", "POST", "PUT", "DELETE"]
    
    # Static mock data
    mock:
      data:
        - id: 1
          name: "John Doe"
          email: "john@example.com"
        - id: 2
          name: "Jane Smith"
          email: "jane@example.com"
    
    # Dynamic mock responses
    mock_responses:
      GET:
        status: 200
        headers:
          "Content-Type": "application/json"
        body: "${mock.data}"
        
      POST:
        status: 201
        body:
          id: "${random_int(1000, 9999)}"
          name: "${request.body.name}"
          email: "${request.body.email}"
          created_at: "${now()}"
          
      "GET /users/{id}":
        status: 200
        body:
          id: "${path.id}"
          name: "User ${path.id}"
          email: "user${path.id}@example.com"
```

### Runtime Mode Endpoint
```yaml
endpoints:
  complex_logic:
    path: "/process"
    methods: ["POST"]
    
    runtime:
      language: "javascript"           # javascript, python, dotnet, rust, shell
      handler: "./handlers/process.js" # File path
      # OR inline handler:
      handler: |
        export default async (request, context) => {
          const { data } = request.body;
          // Custom processing logic
          return { processed: true, result: data };
        }
      
      # Runtime configuration
      timeout: 30                      # Timeout in seconds
      memory_limit: "512MB"           # Memory limit
      environment:                    # Environment variables
        NODE_ENV: "production"
        
    # Available APIs for this endpoint
    apis: ["stripe", "sendgrid"]
    
    # Parameter validation
    parameters:
      - name: "id"
        type: "integer"
        required: true
        minimum: 1
      - name: "data"
        type: "object"
        required: true
```

### Database Mode Endpoint
```yaml
endpoints:
  users_db:
    path: "/users"
    methods: ["GET", "POST", "PUT", "DELETE"]
    
    database:
      table: "users"                   # Table name
      auto_crud: true                  # Auto-generate CRUD operations
      
      # Custom queries
      queries:
        GET: "SELECT * FROM users WHERE active = true ORDER BY created_at DESC"
        POST: |
          INSERT INTO users (name, email, created_at) 
          VALUES (${name}, ${email}, NOW()) 
          RETURNING *
        "GET /users/{id}": "SELECT * FROM users WHERE id = ${id}"
        
      # Response transformation
      transform:
        list: "data"                   # Wrap list responses
        single: "user"                 # Wrap single responses
        
    # Parameter validation
    validation:
      create:
        name: { type: "string", required: true, max_length: 100 }
        email: { type: "string", format: "email", required: true }
      update:
        name: { type: "string", max_length: 100 }
        email: { type: "string", format: "email" }
```

### Proxy Mode Endpoint
```yaml
endpoints:
  external_service:
    path: "/external/*"
    
    proxy:
      target: "https://api.external.com"   # Target URL
      strip_prefix: "/external"            # Remove prefix before forwarding
      timeout: 30                          # Request timeout
      
      # Request transformation
      transform_request:
        add_headers:
          "X-API-Key": "${EXTERNAL_API_KEY}"
          "User-Agent": "Backworks/1.0"
        remove_headers: ["Authorization"]
        
      # Response transformation  
      transform_response:
        add_headers:
          "X-Proxied-By": "Backworks"
        status_code_mapping:
          404: 204                          # Map 404 to 204
```

## 🤖 AI Configuration

```yaml
ai:
  enabled: true                          # Enable AI features
  
  # AI capabilities
  features:
    - "pattern_recognition"              # Recognize API usage patterns
    - "schema_prediction"                # Predict missing schema fields
    - "performance_insights"             # Performance optimization suggestions
    - "anomaly_detection"                # Detect unusual patterns
    - "mock_improvement"                 # Improve mock data over time
    - "documentation_generation"         # Auto-generate documentation
  
  # Model configuration
  models:
    pattern_classifier:
      type: "onnx"                       # onnx or candle
      path: "./models/patterns.onnx"
      confidence_threshold: 0.8
      
    schema_predictor:
      type: "candle"
      path: "./models/schema.safetensors"
      
  # Learning configuration
  learning:
    enabled: true
    retention_days: 30                   # How long to keep learning data
    export_insights: true                # Export insights to files
    
  # Custom AI features per endpoint
  endpoint_ai:
    users:
      generate_realistic_data: true      # AI-generated mock data
      analyze_usage_patterns: true       # Track usage patterns
      predict_missing_fields: true       # Suggest missing fields
```

## 🔌 External APIs Configuration

```yaml
apis:
  # Bearer token authentication
  stripe:
    base_url: "https://api.stripe.com/v1"
    authentication:
      type: "bearer"
      token_env: "STRIPE_SECRET_KEY"
    headers:
      "Content-Type": "application/json"
    timeout: 30
    rate_limit:
      requests_per_minute: 100
      
  # OAuth2 authentication
  salesforce:
    base_url: "https://your-instance.salesforce.com/services/data/v55.0"
    authentication:
      type: "oauth2"
      client_id_env: "SALESFORCE_CLIENT_ID"
      client_secret_env: "SALESFORCE_CLIENT_SECRET"
      token_url: "https://login.salesforce.com/services/oauth2/token"
      scope: "api"
      
  # API Key authentication
  weather:
    base_url: "https://api.openweathermap.org/data/2.5"
    authentication:
      type: "api_key"
      key_env: "WEATHER_API_KEY"
      location: "query"                  # query or header
      parameter: "appid"                 # Parameter name
      
  # Basic authentication
  legacy_system:
    base_url: "https://legacy.company.com/api"
    authentication:
      type: "basic"
      username_env: "LEGACY_USERNAME"
      password_env: "LEGACY_PASSWORD"
      
  # Custom authentication
  custom_api:
    base_url: "https://api.custom.com"
    authentication:
      type: "custom"
      headers:
        "X-API-Key": "${CUSTOM_API_KEY}"
        "X-Client-ID": "${CUSTOM_CLIENT_ID}"
```

## 🗄️ Database Configuration

```yaml
database:
  # PostgreSQL
  type: "postgresql"
  connection_string: "postgresql://user:pass@localhost:5432/dbname"
  connection_string_env: "DATABASE_URL"  # Prefer environment variable
  
  # Connection pool settings
  pool:
    min_connections: 5
    max_connections: 20
    connection_timeout: 30
    
  # MySQL
  type: "mysql"
  connection_string: "mysql://user:pass@localhost:3306/dbname"
  
  # SQLite
  type: "sqlite"
  file_path: "./data/app.db"
  
  # MongoDB
  type: "mongodb"
  connection_string: "mongodb://localhost:27017/dbname"
  
  # Multiple databases
  databases:
    primary:
      type: "postgresql"
      connection_string_env: "PRIMARY_DB_URL"
    analytics:
      type: "clickhouse"
      connection_string_env: "ANALYTICS_DB_URL"
    cache:
      type: "redis"
      connection_string_env: "REDIS_URL"
```

## 🎨 Dashboard Configuration

```yaml
dashboard:
  enabled: true                          # Enable visual dashboard
  port: 3000                            # Dashboard port
  
  # Dashboard features
  features:
    - "flows"                           # Flow diagrams
    - "metrics"                         # Performance metrics
    - "ai_insights"                     # AI-powered insights
    - "architecture"                    # Architecture overview
    - "logs"                           # Request logs
    
  # Real-time updates
  real_time:
    enabled: true
    update_frequency: 1000              # Milliseconds
    
  # Visualization settings
  visualization:
    layout: "hierarchical"              # hierarchical, force-directed, circular
    show_data_flow: true
    animate_requests: true
    color_scheme: "dark"                # dark, light, auto
    
  # Access control
  security:
    enabled: true
    api_key_env: "DASHBOARD_API_KEY"
    allowed_ips: ["127.0.0.1", "10.0.0.0/8"]
```

## 🔒 Security Configuration

```yaml
security:
  # CORS settings
  cors:
    enabled: true
    origins: ["https://app.example.com", "http://localhost:3000"]
    methods: ["GET", "POST", "PUT", "DELETE"]
    headers: ["Content-Type", "Authorization"]
    credentials: true
    
  # Rate limiting
  rate_limiting:
    enabled: true
    requests_per_minute: 100
    burst_size: 20
    key_generator: "ip"                  # ip, user, api_key
    
  # Authentication
  authentication:
    type: "jwt"                          # jwt, api_key, oauth2, custom
    secret_env: "JWT_SECRET"
    algorithm: "HS256"
    expiration: 3600                     # seconds
    
    # Custom validation
    validation:
      handler: "./auth/validate.js"
      
  # Request validation
  validation:
    max_body_size: "10MB"
    require_content_type: true
    validate_json: true
    
  # Security headers
  headers:
    "X-Content-Type-Options": "nosniff"
    "X-Frame-Options": "DENY"
    "X-XSS-Protection": "1; mode=block"
    "Strict-Transport-Security": "max-age=31536000"
```

## 📊 Monitoring Configuration

```yaml
monitoring:
  # Metrics collection
  metrics:
    enabled: true
    export_format: "prometheus"          # prometheus, statsd, json
    export_endpoint: "/metrics"
    
    # Custom metrics
    custom:
      - name: "api_requests_total"
        type: "counter"
        description: "Total API requests"
        labels: ["endpoint", "method", "status"]
        
  # Logging
  logging:
    level: "info"                        # debug, info, warn, error
    format: "json"                       # json, text
    output: "stdout"                     # stdout, file, syslog
    
    # File logging
    file:
      path: "./logs/backworks.log"
      max_size: "100MB"
      max_files: 10
      
  # Health checks
  health:
    enabled: true
    endpoint: "/health"
    checks:
      - name: "database"
        type: "database"
        timeout: 5
      - name: "external_api"
        type: "http"
        url: "https://api.external.com/health"
        
  # Alerting
  alerts:
    enabled: true
    channels:
      slack:
        webhook_url_env: "SLACK_WEBHOOK_URL"
        channel: "#alerts"
      email:
        smtp_host: "smtp.gmail.com"
        smtp_port: 587
        username_env: "SMTP_USERNAME"
        password_env: "SMTP_PASSWORD"
        
    rules:
      - name: "high_error_rate"
        condition: "error_rate > 0.05"
        duration: "5m"
        channels: ["slack", "email"]
```

## 🔄 Capture Mode Configuration

```yaml
mode: "capture"

# Listener configuration
listeners:
  http:
    port: 8080
    capture_all: true
    log_requests: true
    
  websocket:
    port: 8081
    capture_messages: true
    
  proxy:
    port: 8082
    target: "https://existing-api.com"
    capture_and_forward: true

# Capture settings
capture:
  # What to capture
  requests:
    headers: true
    body: true
    query_parameters: true
    method: true
    timestamp: true
    
  responses:
    headers: true
    body: true
    status_code: true
    response_time: true
    
  # Analysis settings
  analysis:
    auto_generate_schema: true
    detect_patterns: true
    group_similar_requests: true
    confidence_threshold: 0.8
    
  # Export settings
  export:
    format: "openapi"                    # openapi, postman, curl, backworks
    output_file: "./captured-api.yaml"
    include_examples: true
```

## 🌍 Multi-Runtime Configuration

```yaml
# Runtime environments
runtimes:
  javascript:
    command: "node"
    version: ">=18.0.0"
    working_dir: "./handlers/js"
    install_deps: "npm install"
    
  python:
    command: "python3"
    version: ">=3.8"
    working_dir: "./handlers/python"
    install_deps: "pip install -r requirements.txt"
    virtual_env: true
    
  dotnet:
    command: "dotnet run"
    project_file: "./handlers/dotnet/Handler.csproj"
    
  rust:
    command: "cargo run --release"
    working_dir: "./handlers/rust"
    
  shell:
    command: "bash"
    working_dir: "./handlers/shell"

# Global runtime settings
runtime_settings:
  timeout: 30                            # Default timeout
  memory_limit: "512MB"                  # Default memory limit
  environment:                           # Global environment variables
    BACKWORKS_VERSION: "1.0.0"
    NODE_ENV: "production"
```

## ⚙️ Advanced Configuration

### Environment Variables
```yaml
# Use environment variables throughout configuration
database:
  connection_string: "${DATABASE_URL}"
  
apis:
  stripe:
    auth: "bearer:${STRIPE_SECRET_KEY}"
    
# Default values
database:
  connection_string: "${DATABASE_URL:-sqlite://./default.db}"
```

### Configuration Inheritance
```yaml
# base.yaml
extends: "./base.yaml"

# Override specific values
endpoints:
  users:
    path: "/v2/users"  # Override path from base.yaml
```

### Conditional Configuration
```yaml
# Different configs for different environments
environments:
  development:
    mode: "mock"
    ai: { enabled: false }
    
  staging:
    mode: "hybrid"
    ai: { enabled: true }
    
  production:
    mode: "database"
    ai: { enabled: true }
    monitoring: { enabled: true }
```

This configuration reference covers all aspects of Backworks configuration. For specific examples and use cases, see the [examples](../examples/) directory.
