# 📋 Examples Collection

This directory contains practical examples demonstrating various Backworks use cases and patterns.

## 🎯 Example Categories

### 🚀 Basic Examples
- **[simple-api](./basic/simple-api/)** - Basic REST API with mock data
- **[crud-operations](./basic/crud-operations/)** - Complete CRUD API
- **[multi-mode](./basic/multi-mode/)** - API using multiple modes

### 🤖 AI-Enhanced Examples  
- **[ai-powered-api](./ai-enhanced/ai-powered-api/)** - AI-enhanced API development
- **[pattern-learning](./ai-enhanced/pattern-learning/)** - Learning from request patterns
- **[intelligent-mocks](./ai-enhanced/intelligent-mocks/)** - AI-generated mock data

### 🔄 Reverse Engineering Examples
- **[api-capture](./reverse-engineering/api-capture/)** - Capturing existing API usage
- **[legacy-migration](./reverse-engineering/legacy-migration/)** - Migrating legacy APIs
- **[proxy-analysis](./reverse-engineering/proxy-analysis/)** - Analyzing through proxy

### 🔌 Integration Examples
- **[external-apis](./integrations/external-apis/)** - Multiple external API integrations
- **[microservices-gateway](./integrations/microservices-gateway/)** - Microservices gateway
- **[database-integration](./integrations/database-integration/)** - Database-driven APIs

### 🏭 Production Examples
- **[ecommerce-api](./production/ecommerce-api/)** - Complete e-commerce API
- **[saas-platform](./production/saas-platform/)** - SaaS platform backend
- **[analytics-dashboard](./production/analytics-dashboard/)** - Analytics API with dashboard

### 🌍 Multi-Runtime Examples
- **[polyglot-api](./multi-runtime/polyglot-api/)** - Multiple programming languages
- **[javascript-handlers](./multi-runtime/javascript-handlers/)** - Node.js handlers
- **[python-ml](./multi-runtime/python-ml/)** - Python ML integration

## 🚀 Quick Start with Examples

### Run a Simple Example
```bash
# Clone the repository
git clone https://github.com/devstroop/backworks
cd backworks/examples

# Run basic API example
cd basic/simple-api
backworks start

# API available at http://localhost:8080
# Dashboard at http://localhost:3000
```

### Try AI-Enhanced Features
```bash
# Run AI-powered example
cd ai-enhanced/ai-powered-api
backworks start

# Watch AI learn and improve your API
```

### Reverse Engineer an API
```bash
# Set up API capture
cd reverse-engineering/api-capture
backworks start

# Point your existing app to http://localhost:8080
# Watch Backworks learn your API patterns
```

## 📖 Example Structure

Each example follows this structure:
```
example-name/
├── project.yaml          # Main configuration
├── README.md              # Example documentation
├── data/                  # Mock data files
├── handlers/              # Custom handlers
│   ├── javascript/
│   ├── python/
│   └── shell/
├── models/                # AI models (if applicable)
├── docs/                  # Additional documentation
└── tests/                 # Example tests
```

## 🎯 Learning Path

### Beginner (Start Here)
1. **[simple-api](./basic/simple-api/)** - Understanding basic concepts
2. **[crud-operations](./basic/crud-operations/)** - CRUD operations
3. **[external-apis](./integrations/external-apis/)** - External integrations

### Intermediate
1. **[ai-powered-api](./ai-enhanced/ai-powered-api/)** - AI enhancement
2. **[microservices-gateway](./integrations/microservices-gateway/)** - Gateway patterns
3. **[polyglot-api](./multi-runtime/polyglot-api/)** - Multi-language handlers

### Advanced
1. **[legacy-migration](./reverse-engineering/legacy-migration/)** - Complex migrations
2. **[ecommerce-api](./production/ecommerce-api/)** - Production patterns
3. **[saas-platform](./production/saas-platform/)** - Enterprise architecture

## 🤝 Contributing Examples

Want to contribute an example? Great! Please:

1. **Follow the structure** - Use the standard example structure
2. **Include documentation** - Add a comprehensive README.md
3. **Add tests** - Include example tests where applicable
4. **Keep it simple** - Focus on demonstrating specific concepts
5. **Add to the index** - Update this README.md

### Example Contribution Template
```yaml
# your-example/project.yaml
name: "your_example_name"
description: "Brief description of what this example demonstrates"

# Your configuration here
endpoints:
  example:
    path: "/example"
    mock:
      data: {"message": "Hello from example"}
```

```markdown
# Your Example Name

## Overview
Brief description of what this example demonstrates.

## Features Demonstrated
- Feature 1
- Feature 2
- Feature 3

## Quick Start
```bash
backworks start
```

## What You'll Learn
- Learning point 1
- Learning point 2

## Next Steps
- Link to related examples
- Link to relevant documentation
```

## 📚 Additional Resources

- **[Configuration Reference](../docs/configuration.md)** - Complete configuration guide
- **[AI Features](../docs/ai-features.md)** - AI capabilities documentation
- **[Dashboard Guide](../docs/dashboard.md)** - Visual dashboard features
- **[Community Examples](https://github.com/backworks-community/examples)** - Community contributions

Start with the [simple-api](./basic/simple-api/) example and work your way through the collection to master Backworks!
