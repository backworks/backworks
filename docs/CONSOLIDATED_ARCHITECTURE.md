# Backworks Architecture

## Overview

Backworks is an API platform that follows a plugin-first, modular core philosophy. The architecture is designed to evolve from simple mock APIs to production-ready services through configuration changes rather than code changes.

## Key Architectural Principles

1. **Plugin-First Architecture**: Core functionality is minimal, with advanced features implemented as plugins
2. **Configuration-Driven**: All behavior controlled through YAML configuration
3. **Mode Evolution**: Seamlessly transition between modes (proxy, capture, etc.)
4. **Developer Experience**: Focus on ease of use and clear development patterns

## Current Architecture (Mock-Free)

As of the most recent update, Backworks has moved to a fully functional, mock-free architecture:

- Default execution mode is now `proxy` (previously `mock`)
- All placeholder/mock code has been removed
- The system uses real HTTP proxying for all functionality
- Tests have been updated to use real proxy handlers

## Core Components

### Engine (`engine.rs`)
The central orchestration component that:
- Initializes the system based on configuration
- Manages plugins
- Coordinates request handling

### Server (`server.rs`)
HTTP server implementation that:
- Handles incoming HTTP requests
- Routes to appropriate handlers
- Implements proxy functionality

### Proxy (`proxy.rs`)
Real HTTP proxy functionality that:
- Forwards requests to backend services
- Captures request/response data
- Handles error cases

### Capture (`capture.rs`)
Request/response capture system that:
- Records API interactions
- Filters requests based on patterns
- Stores capture data

### Plugin System (`plugin.rs`, `resilience.rs`)
Plugin architecture that:
- Provides extension points for additional functionality
- Implements resilience patterns (circuit breakers, etc.)
- Manages plugin lifecycle

### Configuration (`config.rs`)
Configuration management that:
- Parses YAML configuration
- Validates settings
- Provides defaults

## Execution Flow

1. Configuration is parsed (`config.rs`)
2. Engine is initialized with configuration (`engine.rs`)
3. Plugins are registered and initialized (`plugin.rs`)
4. Server starts and listens for requests (`server.rs`)
5. Incoming requests are routed to handlers
6. Proxy handler forwards requests to backend services (`proxy.rs`)
7. Responses are returned to clients
8. (Optional) Requests/responses are captured (`capture.rs`)

## Plugin Architecture

The plugin system is a key part of the Backworks architecture:

- Plugins implement the `BackworksPlugin` trait
- The plugin manager handles registration and lifecycle
- Circuit breakers prevent cascade failures
- Resource limits manage plugin execution
- Health monitoring ensures system stability

## Future Direction

1. **Enhanced Plugin System**: Fully implement resilient plugin execution
2. **Advanced Proxy Features**: Circuit breakers, resilience patterns
3. **AI Enhancements**: Integrate real ML/AI functionality
4. **Monitoring & Analytics**: Comprehensive observability solutions

## Related Documents

- [Mock-Free Architecture](MOCK_FREE_ARCHITECTURE.md): Detailed information on mock-free design
- [Final Architecture](../FINAL_ARCHITECTURE.md): Target architecture documentation
- [Refactoring Plan](REFACTORING_PLAN.md): Plan for the mock-free refactoring
- [Implementation Roadmap](../IMPLEMENTATION_ROADMAP.md): Phases of implementation
