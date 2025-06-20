# Contributing to Backworks

Thank you for your interest in contributing to Backworks! This document provides guidelines and information about contributing to the project.

## Table of Contents
- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Code Style](#code-style)
- [Documentation](#documentation)
- [Community](#community)

## Code of Conduct

This project follows a Code of Conduct to ensure a welcoming environment for everyone. Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Getting Started

### Prerequisites
- **Rust**: Install via [rustup.rs](https://rustup.rs/)
- **Node.js**: Version 16+ for JavaScript handler support
- **Git**: For version control
- **Docker**: Optional, for containerized development

### Development Setup

1. **Fork and Clone**
   ```bash
   git clone https://github.com/your-username/backworks.git
   cd backworks
   ```

2. **Install Dependencies**
   ```bash
   # Rust dependencies
   cargo build
   
   # Install development tools
   rustup component add rustfmt clippy
   cargo install cargo-watch cargo-audit
   ```

3. **Verify Setup**
   ```bash
   cargo test
   cargo run -- --version
   ```

4. **Run Hello World Example**
   ```bash
   cd examples/hello-world
   ../../target/debug/backworks start
   ```

## Making Changes

### Branch Strategy
- `main`: Stable release branch
- `develop`: Development integration branch
- Feature branches: `feature/your-feature-name`
- Bug fixes: `fix/issue-description`
- Documentation: `docs/topic-name`

### Workflow
1. Create a feature branch from `develop`
2. Make your changes
3. Add tests for new functionality
4. Ensure all tests pass
5. Update documentation
6. Submit a pull request

### Types of Contributions

#### Bug Fixes
- Check existing issues before creating new ones
- Include reproduction steps
- Add regression tests
- Update documentation if needed

#### New Features
- Discuss major features in issues first
- Follow existing patterns and conventions
- Include comprehensive tests
- Update documentation and examples
- Consider backward compatibility

#### Documentation
- Fix typos and improve clarity
- Add missing documentation
- Update examples
- Improve code comments

## Testing

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test

# Run integration tests
cargo test --test integration_tests
```

### Writing Tests
- Unit tests: Place in the same file as the code being tested
- Integration tests: Place in `tests/` directory
- Example tests: Ensure examples work as documented

### Test Guidelines
- Test both happy path and error cases
- Use descriptive test names
- Keep tests focused and independent
- Mock external dependencies

## Code Style

### Rust Code Style
We use standard Rust formatting and linting tools:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check for security issues
cargo audit
```

### Guidelines
- Follow Rust naming conventions
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Keep functions focused and small
- Handle errors appropriately (don't panic in library code)

### Example Code Style
```rust
/// Executes a JavaScript handler with the given request data.
/// 
/// # Arguments
/// * `handler_code` - The JavaScript code or file path
/// * `request_data` - JSON string containing request data
/// 
/// # Returns
/// * `Result<String, BackworksError>` - The handler output or error
/// 
/// # Example
/// ```rust
/// let result = execute_javascript_handler("function handler() { return 'hello'; }", "{}").await?;
/// ```
pub async fn execute_javascript_handler(
    handler_code: &str, 
    request_data: &str
) -> BackworksResult<String> {
    // Implementation
}
```

## Documentation

### Types of Documentation
- **API Documentation**: Rust doc comments (`///`)
- **User Documentation**: Markdown files in `docs/`
- **Examples**: Working code examples in `examples/`
- **README**: Project overview and quick start

### Writing Documentation
- Use clear, concise language
- Include working examples
- Keep documentation up to date with code changes
- Use proper markdown formatting

### Building Documentation
```bash
# Build API docs
cargo doc --open

# Test documentation examples
cargo test --doc
```

## Submitting Changes

### Pull Request Process
1. **Create Pull Request**
   - Use the PR template
   - Provide clear description
   - Link related issues
   - Include testing information

2. **Code Review**
   - Address reviewer feedback
   - Keep PR focused and atomic
   - Update based on suggestions

3. **Merge Requirements**
   - All tests must pass
   - Code review approval required
   - Documentation must be updated
   - No merge conflicts

### Pull Request Checklist
- [ ] Tests added/updated and passing
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (for significant changes)
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Security audit clean (`cargo audit`)

## Project Structure

```
backworks/
├── src/           # Core Rust source code
├── examples/      # Example projects
├── docs/          # Documentation
├── tests/         # Integration tests
├── scripts/       # Build and utility scripts
├── .github/       # GitHub Actions and templates
└── target/        # Build artifacts
```

### Key Components
- **Core Engine** (`src/engine.rs`): Main application engine
- **Configuration** (`src/config.rs`): Configuration parsing and validation
- **Runtime** (`src/runtime.rs`): Handler execution
- **Proxy** (`src/proxy.rs`): Proxy functionality
- **Server** (`src/server.rs`): HTTP server implementation

## Development Tips

### Local Development
```bash
# Use cargo-watch for hot reload
cargo watch -x 'run -- start'

# Debug with detailed logging
RUST_LOG=debug cargo run -- start

# Run specific example
cd examples/hello-world && cargo run --bin backworks -- start
```

### Docker Development
```bash
# Build development image
docker-compose -f docker-compose.yml --profile dev up

# Run tests in container
docker-compose run backworks cargo test
```

### Debugging
- Use `tracing` for logging, not `println!`
- Add appropriate log levels (debug, info, warn, error)
- Use `RUST_LOG=debug` for detailed output
- Use debugger or add temporary debug prints for complex issues

## Release Process

The project uses semantic versioning (SemVer):
- **Major** (1.0.0): Breaking changes
- **Minor** (0.1.0): New features, backward compatible
- **Patch** (0.0.1): Bug fixes, backward compatible

Releases are automated through GitHub Actions when tags are pushed.

## Community

### Getting Help
- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and general discussion
- **Documentation**: Check docs/ directory first

### Communication Guidelines
- Be respectful and constructive
- Search existing issues before creating new ones
- Provide detailed information for bug reports
- Use clear, descriptive titles

### Recognition
Contributors are recognized in:
- Git commit history
- Release notes
- Project documentation

## Thank You!

Your contributions help make Backworks better for everyone. Whether you're fixing bugs, adding features, improving documentation, or helping other users, your efforts are appreciated!

For questions about contributing, feel free to open an issue or start a discussion.
