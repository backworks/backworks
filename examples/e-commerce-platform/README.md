# Plugin-Enhanced E-commerce Platform

This example demonstrates how Backworks' plugin system extends the blueprint-agnostic platform with powerful, reusable functionality.

## 🎯 **Blueprint with Plugin Integration**

The same blueprint works across all platforms (web, desktop, mobile, CLI) with plugins providing cross-cutting concerns.

## 📁 **Project Structure**

```
e-commerce-platform/
├── blueprint.yaml           # Main application blueprint
├── plugins/                 # Custom plugins
│   ├── inventory.js         # Inventory management
│   ├── payment.js           # Payment processing
│   └── shipping.js          # Shipping calculations
├── config/
│   ├── development.yaml     # Development plugin config
│   ├── production.yaml      # Production plugin config
│   └── testing.yaml         # Testing plugin config
└── runtime/                 # Platform-specific runtime
    ├── web/                 # Web server runtime
    ├── desktop/             # Desktop app runtime
    ├── mobile/              # Mobile app runtime
    └── cli/                 # CLI tool runtime
```

## 🔌 **Plugin Categories**

### **Core Plugins (Built-in)**
- **Authentication** - JWT, OAuth, session management
- **Rate Limiting** - Request throttling, DDoS protection
- **Caching** - Memory, Redis, distributed caching
- **Logging** - Structured logging, audit trails
- **Metrics** - Performance monitoring, analytics

### **Business Plugins (Custom)**
- **Inventory Management** - Stock tracking, reorder points
- **Payment Processing** - Stripe, PayPal, credit cards
- **Shipping Calculations** - FedEx, UPS, DHL integrations
- **Tax Calculations** - Regional tax rules, exemptions
- **Fraud Detection** - Risk scoring, blacklists

### **Platform Plugins (Platform-specific)**
- **Web Plugins** - SEO, PWA, web analytics
- **Desktop Plugins** - System notifications, file access
- **Mobile Plugins** - Push notifications, camera, GPS
- **CLI Plugins** - Shell completions, progress bars

## 🚀 **Usage Examples**

### **Web Server Runtime**
```bash
# Start with web-specific plugins
backworks start --config blueprint.yaml --platform web --env production

# Available at:
# API: http://localhost:3000/api/products
# Admin: http://localhost:3001/admin
# Metrics: http://localhost:3002/metrics
```

### **Desktop App Runtime**
```bash
# Start with desktop-specific plugins
backworks start --config blueprint.yaml --platform desktop --env development

# Features:
# - Native GUI with system tray
# - Offline mode with local sync
# - Desktop notifications
# - File import/export
```

### **Mobile App Runtime**
```bash
# Start with mobile-specific plugins
backworks start --config blueprint.yaml --platform mobile --env production

# Features:
# - Touch-optimized UI
# - Push notifications
# - Camera barcode scanning
# - GPS location services
```

### **CLI Tool Runtime**
```bash
# Start with CLI-specific plugins
backworks start --config blueprint.yaml --platform cli --env development

# Commands available:
# ecommerce products list --status active
# ecommerce orders create --customer-id 123
# ecommerce inventory check --sku ABC123
```

## 🎯 **Plugin Benefits**

### **Reusability**
- Same plugins work across ALL platforms
- No platform-specific plugin development
- Shared plugin marketplace and ecosystem

### **Composability**
- Mix and match plugins as needed
- Enable/disable plugins per environment
- Plugin dependencies and ordering

### **Hot-Pluggable**
- Add/remove plugins without restarts
- Dynamic plugin configuration
- Runtime plugin health monitoring

### **Community-Driven**
- Plugin marketplace for sharing
- Plugin SDK for easy development
- Plugin templates and examples

---

**Note:** This demonstrates the plugin architecture vision. Current Backworks has the plugin framework implemented but is building out the plugin ecosystem.
