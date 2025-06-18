# Architecture Simplification - Capture Mechanism

## Problem Identified

The original capture mechanism had a fundamental architectural flaw:
- **Standalone `Capture` mode** only captured requests but didn't respond to them
- This would break any real API traffic since clients wouldn't get responses
- Unnecessary complexity with separate capture handlers
- Confusing execution modes (`Hybrid`, `Capture`, etc.)

## Solution Implemented

### 1. Simplified Execution Modes

**Before:**
```rust
pub enum ExecutionMode {
    Mock,
    Capture,     // ❌ Removed - fundamentally flawed
    Runtime,
    Database,
    Proxy,
    Hybrid,      // ❌ Removed - unnecessary complexity
    Evolving,    // ❌ Removed - unused
}
```

**After:**
```rust
pub enum ExecutionMode {
    Mock,        // ✅ Simple mock responses
    Runtime,     // ✅ Execute code handlers
    Database,    // ✅ Database-driven responses
    Proxy,       // ✅ Proxy with optional capture
    Plugin,      // ✅ Plugin-based execution
}
```

### 2. Integrated Capture into Proxy

**Before:** Separate capture mode that doesn't respond
```yaml
mode: capture  # ❌ Captures but doesn't respond!
endpoints:
  - path: "/*"
    mode: capture
```

**After:** Proxy with integrated capture
```yaml
mode: proxy
endpoints:
  main_api:
    path: "/*"
    mode: proxy
    proxy:
      target: "http://localhost:3000"  # ✅ Real responses
      capture:                         # ✅ Capture while proxying
        enabled: true
        storage_path: "./captures"
```

### 3. Removed Unnecessary Components

- ❌ **Standalone CaptureHandler** - moved to proxy integration
- ❌ **Hybrid mode** - unnecessary complexity
- ❌ **capture_handler field** from AppState
- ❌ **Standalone capture execution** logic

### 4. Logical Architecture Flow

**New Flow:**
1. **Proxy Mode** → Forward to real API + optionally capture
2. **Mock Mode** → Return predefined responses  
3. **Plugin Mode** → Execute plugin logic
4. **Database Mode** → Query database for responses
5. **Runtime Mode** → Execute code handlers

## Benefits

### ✅ **Simplified & Logical**
- Capture now works **with** real APIs, not instead of them
- Clear separation of concerns
- Reduced cognitive overhead

### ✅ **Practical & Usable**
- Proxy + capture = real-world traffic analysis
- Generated configs are based on actual API behavior
- No broken request/response cycles

### ✅ **Reduced Complexity**
- Fewer execution modes to understand
- Less code to maintain
- Clearer error handling

## Updated Examples

### Basic Proxy with Capture
```yaml
name: simple-proxy-capture
mode: proxy

endpoints:
  main_api:
    path: "/*"
    mode: proxy
    proxy:
      target: "http://localhost:3000"
      capture:
        enabled: true
        storage_path: "./captures"
        include_patterns: ["/api/*"]
        exclude_patterns: ["/health"]
```

### Plugin-based Capture
```yaml
name: plugin-capture
mode: plugin

plugins:
  capture_plugin:
    type: "capture"
    config:
      storage_path: "./captures"

endpoints:
  api_analysis:
    path: "/api/*"
    mode: plugin
    plugin: "capture_plugin"
```

## Migration Path

For users with existing capture configurations:

**Old:**
```yaml
mode: capture
capture:
  enabled: true
```

**New:**
```yaml
mode: proxy
endpoints:
  main:
    mode: proxy
    proxy:
      target: "YOUR_REAL_API_URL"
      capture:
        enabled: true
```

## Conclusion

The architecture is now much simpler and more practical:
- **5 clear execution modes** instead of 7 confusing ones
- **Capture integrates with proxy** for real-world usage
- **Removed unnecessary complexity** while maintaining functionality
- **Better separation of concerns** between different operational modes

This provides a solid foundation for the plugin-first, resilient architecture we're building.
