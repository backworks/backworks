# 🚀 Backworks

**The Configuration-Driven API Platform That Works Backwards**

Backworks revolutionizes API development by working backwards from your needs. Start with a simple YAML configuration and seamlessly evolve from mock APIs to production-ready services with AI enhancement, visual monitoring, and multi-runtime support.

## ✨ What Makes Backworks Special?

- **🎯 Reverse API Generation** - Capture existing API usage and auto-generate configurations
- **🔄 Seamless Evolution** - Mock → Capture → Runtime → Production without breaking changes
- **🤖 AI-Powered Intelligence** - Enhanced with Candle & ONNX for smart pattern recognition
- **🎨 Visual Architecture** - Real-time flow diagrams and performance monitoring
- **🌐 Multi-Runtime Support** - JavaScript, Python, .NET, Rust, Shell scripts, and more
- **🔌 External API Integration** - First-class support for 3rd party services
- **📊 Intelligent Monitoring** - Auto-generated insights and optimization suggestions

## 🚀 Quick Start

```bash
# Install Backworks
cargo install backworks

# Create your first API
echo 'name: "my_api"
endpoints:
  users:
    path: "/users"
    mock:
      data: [{"id": 1, "name": "John Doe"}]' > backworks.yaml

# Start the API
backworks start

# API running at http://localhost:8080
# Dashboard at http://localhost:3000
```

## 🎯 Use Cases

### 1. **Rapid Prototyping**
Start with mock data and have your API running in seconds:

```yaml
name: "product_catalog"
endpoints:
  products:
    path: "/products"
    mock:
      data: "./data/products.json"
```

### 2. **Reverse Engineering**
Capture existing API usage patterns:

```yaml
name: "legacy_api_capture"
mode: "capture"
listeners:
  http: { port: 8080, capture_all: true }
```

### 3. **AI-Enhanced Development**
Let AI improve your APIs based on real usage:

```yaml
name: "intelligent_api"
ai:
  enabled: true
  features: ["schema_prediction", "mock_improvement", "anomaly_detection"]
```

### 4. **Multi-Service Integration**
Connect multiple external APIs seamlessly:

```yaml
name: "integration_hub"
apis:
  stripe: { auth: "bearer:${STRIPE_KEY}" }
  sendgrid: { auth: "bearer:${SENDGRID_KEY}" }
  salesforce: { auth: "oauth2:${SF_TOKEN}" }
```

## 📖 Documentation

- [🏗️ Architecture Overview](./docs/architecture.md)
- [⚡ Quick Start Guide](./docs/quick-start.md)
- [📝 Configuration Reference](./docs/configuration.md)
- [🔄 Evolution Modes](./docs/modes.md)
- [🤖 AI Features](./docs/ai-features.md)
- [🎨 Visual Dashboard](./docs/dashboard.md)
- [🔌 External APIs](./docs/external-apis.md)
- [🌍 Multi-Runtime Support](./docs/runtimes.md)
- [📊 Monitoring & Analytics](./docs/monitoring.md)
- [🔧 Advanced Usage](./docs/advanced.md)

## 🛠️ Installation

### From Source
```bash
git clone https://github.com/devstroop/backworks
cd backworks
cargo build --release
```

### Using Cargo
```bash
cargo install backworks
```

### Docker
```bash
docker run -p 8080:8080 -p 3000:3000 backworks/backworks
```

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   YAML Config   │───▶│  Backworks Core  │───▶│  Visual Dashboard│
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                    ┌─────────┼─────────┐
                    ▼         ▼         ▼
              ┌──────────┐ ┌─────────┐ ┌──────────┐
              │   Mock   │ │ Runtime │ │ Capture  │
              │   Mode   │ │  Mode   │ │   Mode   │
              └──────────┘ └─────────┘ └──────────┘
                    │         │         │
                    └─────────┼─────────┘
                              ▼
                    ┌──────────────────┐
                    │  AI Enhancement  │
                    │ (Candle + ONNX)  │
                    └──────────────────┘
```

## 🌟 Examples

### E-commerce API
```yaml
name: "ecommerce_api"
mode: "production"

endpoints:
  products:
    path: "/products"
    database: { table: "products" }
    
  orders:
    path: "/orders"
    runtime:
      language: "javascript"
      handler: "./handlers/orders.js"
    apis: ["stripe", "sendgrid"]
    
apis:
  stripe:
    base_url: "https://api.stripe.com/v1"
    auth: "bearer:${STRIPE_SECRET_KEY}"
    
ai:
  enabled: true
  features: ["performance_insights", "security_analysis"]
  
dashboard:
  enabled: true
  features: ["flows", "metrics", "ai_insights"]
```

### Microservices Gateway
```yaml
name: "microservices_gateway"
mode: "proxy"

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
      
ai:
  enabled: true
  features: ["traffic_analysis", "bottleneck_detection"]
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](./CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## 🙏 Acknowledgments

- [Candle](https://github.com/huggingface/candle) - Rust ML framework
- [ONNX Runtime](https://onnxruntime.ai/) - Cross-platform ML inference
- [Tokio](https://tokio.rs/) - Async runtime for Rust

---

**Backworks: Making complex API architectures simple to build, evolve, and maintain.**
