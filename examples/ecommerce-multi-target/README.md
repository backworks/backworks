# Multi-Target E-commerce Platform with Blueprint Compiler

This example demonstrates the **blueprint compiler approach** - organizing YAML files in a project and compiling target-specific blueprints with security profiles.

## 📁 **Organized Project Structure**

```
ecommerce-multi-target/
├── blueprint.yaml              # Main orchestration file
├── endpoints/                  # API endpoint definitions
│   ├── products.yaml          # Product management endpoints
│   ├── orders.yaml            # Order processing endpoints
│   ├── customers.yaml         # Customer management endpoints
│   └── admin.yaml             # Admin-only endpoints
├── plugins/                   # Plugin configurations
│   ├── security.yaml          # Security plugins (auth, rate limiting)
│   ├── business.yaml          # Business plugins (inventory, payment)
│   ├── analytics.yaml         # Analytics and monitoring
│   └── mobile.yaml            # Mobile-specific plugins
├── database/                  # Database configurations
│   ├── schemas.yaml           # Table definitions
│   ├── migrations.yaml        # Database migrations
│   └── seeds.yaml             # Test data
├── ui/                        # UI component definitions
│   ├── web/                   # Web interface components
│   ├── desktop/               # Desktop app components
│   ├── mobile/                # Mobile app components
│   └── admin/                 # Admin panel components
├── infrastructure/            # Infrastructure configs
│   ├── development.yaml       # Development environment
│   ├── production.yaml        # Production environment
│   └── security.yaml          # Security profiles
└── compiled/                  # Generated target-specific blueprints
    ├── web_api.yaml           # Compiled for web API server
    ├── desktop_app.yaml       # Compiled for desktop application
    ├── mobile_app.yaml        # Compiled for mobile application
    └── admin_panel.yaml       # Compiled for admin panel
```

## 🔧 **Blueprint Compiler Usage**

### **Compilation Commands**
```bash
# Compile for web API server (production security)
backworks compile --config blueprint.yaml --target web_api --security production
# Output: ./compiled/web_api.yaml

# Compile for desktop app (client security)
backworks compile --config blueprint.yaml --target desktop_app --security client
# Output: ./compiled/desktop_app.yaml

# Compile for mobile app (mobile security)
backworks compile --config blueprint.yaml --target mobile_app --security mobile
# Output: ./compiled/mobile_app.yaml

# Compile for admin panel (admin security)
backworks compile --config blueprint.yaml --target admin_panel --security admin
# Output: ./compiled/admin_panel.yaml
```

### **Development Workflow**
```bash
# 1. Develop with full blueprint (development mode)
backworks start --config blueprint.yaml --env development

# 2. Test specific target compilation
backworks compile --config blueprint.yaml --target web_api --security development
backworks start --config ./compiled/web_api.yaml

# 3. Deploy with production security
backworks compile --config blueprint.yaml --target web_api --security production
backworks deploy --config ./compiled/web_api.yaml --env production
```

## 🎯 **Target-Specific Compilation**

### **Web API Server** (./compiled/web_api.yaml)
```yaml
# Compiled blueprint for web API server
name: "E-commerce API Server"
mode: "runtime"

# Only server-relevant endpoints included
endpoints:
  products: { ... }  # From endpoints/products.yaml
  orders: { ... }    # From endpoints/orders.yaml
  customers: { ... } # From endpoints/customers.yaml
  # admin endpoints EXCLUDED for security

# Only server-relevant plugins
plugins:
  auth: { ... }           # From plugins/security.yaml
  rate_limiter: { ... }   # From plugins/security.yaml
  inventory: { ... }      # From plugins/business.yaml
  # mobile plugins EXCLUDED

# Database access included
database: { ... }  # From database/schemas.yaml

# UI components EXCLUDED
# Secrets STRIPPED for security
# Internal endpoints OBFUSCATED
```

### **Desktop App** (./compiled/desktop_app.yaml)
```yaml
# Compiled blueprint for desktop application
name: "E-commerce Desktop App"
mode: "runtime"

# Only client-safe endpoints
endpoints:
  products: { ... }   # Read-only product access
  orders: { ... }     # User's orders only
  # customers endpoint EXCLUDED
  # admin endpoints EXCLUDED

# Client-safe plugins only
plugins:
  local_auth: { ... }     # Local authentication only
  ui_components: { ... }  # Desktop UI plugins
  # server plugins EXCLUDED

# Database config EXCLUDED (API calls only)
api_base_url: "https://api.ecommerce.com"

# Desktop UI components included
ui:
  desktop: { ... }  # From ui/desktop/

# Local config ENCRYPTED
# Secrets REMOVED (API keys only)
```

### **Mobile App** (./compiled/mobile_app.yaml)
```yaml
# Compiled blueprint for mobile application
name: "E-commerce Mobile App"
mode: "runtime"

# Mobile-optimized endpoints
endpoints:
  products: { ... }  # Lightweight product data
  orders: { ... }    # Mobile order flow
  # Heavy endpoints EXCLUDED

# Mobile-specific plugins
plugins:
  api_auth: { ... }         # API key authentication
  push_notifications: { ... } # Mobile notifications
  offline_sync: { ... }    # Offline capabilities
  # Server plugins EXCLUDED

# Mobile UI components only
ui:
  mobile: { ... }  # From ui/mobile/

# API configuration only
api_base_url: "https://api.ecommerce.com"
api_key: "${MOBILE_API_KEY}"  # Encrypted

# Certificate pinning ENABLED
# Runtime protection ENABLED
```

## 🛡️ **Security Benefits**

### **Attack Surface Reduction**
- **Web API**: No UI code, no client secrets, no admin endpoints
- **Desktop App**: No database access, no server secrets, no admin functions
- **Mobile App**: Minimal endpoints, no server plugins, encrypted config

### **Secret Management**
- **Production**: All secrets stripped, environment variables only
- **Client**: Local encryption, no server secrets
- **Mobile**: API keys only, certificate pinning

### **Runtime Protection**
- **Obfuscation**: Internal identifiers renamed in compiled blueprints
- **Dead Code Elimination**: Unused endpoints/plugins removed
- **Security Injection**: Platform-specific security automatically added

## 🚀 **Advantages**

### **1. Security by Design**
- Each target gets only what it needs
- Secrets managed per deployment target
- Security profiles enforced at compile time

### **2. Performance Optimization**
- Smaller compiled blueprints
- Faster startup times
- Reduced memory footprint

### **3. Development Efficiency**
- Organize complex projects with multiple YAML files
- Single source of truth with target-specific compilation
- Environment-specific configurations

### **4. Deployment Flexibility**
- Deploy different targets independently
- Scale different components separately
- Update targets without affecting others

---

**Note:** This demonstrates the blueprint compiler vision. The current Backworks implementation supports organized YAML files, and the compiler functionality is planned for future development.
