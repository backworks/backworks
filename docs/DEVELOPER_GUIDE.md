# Developer Guide

This guide provides instructions for developers contributing to the Backworks API platform.

## Getting Started

### Prerequisites

- Rust (stable): 1.67+ recommended
- Cargo
- Git

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/devstroop/backworks.git
   cd backworks
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

## Project Structure

- `/src`: Core source code
  - `lib.rs`: Library entry point
  - `main.rs`: Binary entry point
  - `engine.rs`: Core engine implementation
  - `server.rs`: HTTP server implementation
  - `proxy.rs`: Proxy handling
  - `capture.rs`: Request capture functionality
  - `config.rs`: Configuration management
  - `plugin.rs`: Plugin trait definitions
  - `plugins/`: Plugin implementations
  - `resilience.rs`: Resilience patterns (circuit breakers, etc.)
  - `database.rs`: Database interactions
  - `dashboard.rs`: Dashboard functionality
  - `runtime.rs`: Runtime execution
  - `error.rs`: Error types and handling

- `/docs`: Documentation
  - Architecture documentation
  - Configuration guides
  - User guides

- `/examples`: Example configurations and usage patterns
  - Basic examples
  - Advanced use cases
  - Template configurations

- `/tests`: Integration tests

## Development Workflow

### Branching Strategy

- `main`: Stable releases
- `develop`: Active development
- Feature branches: `feature/your-feature-name`
- Bug fix branches: `fix/issue-name-or-number`

### Commit Guidelines

- Use descriptive commit messages
- Start with a verb (Add, Fix, Update, Refactor, etc.)
- Reference issue numbers when applicable: `Fix: Resolve proxy handling issue (Fixes #123)`

### Pull Request Process

1. Create a feature or fix branch from `develop`
2. Implement your changes
3. Ensure tests pass: `cargo test`
4. Ensure no linting errors: `cargo clippy`
5. Update documentation if necessary
6. Submit PR to `develop`

### Code Style

- Follow Rust standard practices
- Use `cargo fmt` to format code
- Follow `clippy` recommendations

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific tests
cargo test proxy_capture_integration

# Run with verbose output
cargo test -- --nocapture
```

### Writing Tests

- Unit tests: Place in the same file as the code being tested
- Integration tests: Add to the `/tests` directory
- Mock external services when necessary

## Documentation

- Update documentation when changing functionality
- Document public APIs with rustdoc comments
- Keep examples up-to-date with the latest features

## Issue Tracking

The project uses a centralized issue tracker in `ISSUES.md`. When working on issues:

1. Update the issue status in `ISSUES.md`
2. Reference the issue ID in commits and PRs
3. Update any relevant documentation

## CI/CD

The project uses GitHub Actions for CI/CD:

- Tests run on every PR
- Clippy linting on every PR
- Documentation generation on merges to `main`

## Getting Help

- Check existing documentation in `/docs`
- Review the issue tracker for similar issues
- Contact the team lead for assistance
