# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New project-based architecture with package.json support
- Array-based blueprint endpoint format
- External JavaScript handler support alongside inline handlers
- Comprehensive CI/CD workflows with GitHub Actions
- Docker containerization support
- Multi-platform release builds (Linux, macOS, Windows, ARM64)
- Security scanning and dependency auditing
- Documentation automation and deployment
- Hot reload development environment

### Changed
- Refactored all examples to use modern project structure
- Improved handler execution with proper file path resolution
- Updated blueprint format to support both legacy and new formats
- Enhanced error handling and logging

### Fixed
- JavaScript handler file loading and execution
- Mode detection for proxy vs runtime configurations
- Path resolution for external handler files

### Security
- Added automated security auditing with cargo-audit
- Implemented CodeQL analysis for vulnerability detection
- Added container security scanning with Trivy
- Secret scanning with TruffleHog

## [1.0.0] - TBD

### Added
- Initial stable release
- Core API runtime functionality
- Proxy capabilities with transformations
- JavaScript and Python handler support
- YAML-based configuration
- Dashboard interface
- Database integration
- Plugin system
- Multiple deployment modes

### Features
- **Runtime Mode**: Execute custom handlers for API endpoints
- **Proxy Mode**: Forward requests with transformations
- **Multi-language Support**: JavaScript, Python handlers
- **Transformations**: Request/response modification
- **Load Balancing**: Multiple upstream targets
- **Rate Limiting**: Built-in rate limiting
- **Monitoring**: Metrics and health checks
- **Hot Reload**: Development-friendly configuration reloading

### Examples
- Hello World API (inline and external handlers)
- Blog API (complex data structures)
- Advanced Proxy (transformation examples)
- Task Manager (CRUD operations)
- Proxy API (simple forwarding)

### Documentation
- Comprehensive README
- API documentation
- Configuration guide
- Development guide
- Architecture documentation

### Infrastructure
- Cross-platform builds
- Docker containerization
- CI/CD pipelines
- Automated testing
- Security scanning
- Documentation deployment
