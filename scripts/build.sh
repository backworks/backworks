#!/bin/bash
# Build script for Backworks

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Building Backworks...${NC}"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust: https://rustup.rs/${NC}"
    exit 1
fi

# Check if Node.js is installed (needed for JavaScript handlers)
if ! command -v node &> /dev/null; then
    echo -e "${RED}⚠️  Node.js not found. JavaScript handlers will not work.${NC}"
    echo -e "${BLUE}   Install Node.js: https://nodejs.org/${NC}"
fi

# Clean previous builds if requested
if [[ "${1:-}" == "clean" ]]; then
    echo -e "${BLUE}🧹 Cleaning previous builds...${NC}"
    cargo clean
fi

# Format code
echo -e "${BLUE}🎨 Formatting code...${NC}"
cargo fmt

# Run clippy for linting
echo -e "${BLUE}📝 Running clippy...${NC}"
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
echo -e "${BLUE}🧪 Running tests...${NC}"
cargo test

# Build release binary
echo -e "${BLUE}🔨 Building release binary...${NC}"
cargo build --release

# Verify the binary works
echo -e "${BLUE}✅ Verifying binary...${NC}"
./target/release/backworks --version

echo -e "${GREEN}🎉 Build completed successfully!${NC}"
echo -e "${BLUE}📦 Binary location: ./target/release/backworks${NC}"

# Show binary size
BINARY_SIZE=$(du -h ./target/release/backworks | cut -f1)
echo -e "${BLUE}📊 Binary size: ${BINARY_SIZE}${NC}"

# Test with hello-world example if available
if [[ -d "examples/hello-world" ]]; then
    echo -e "${BLUE}🧪 Quick test with hello-world example...${NC}"
    cd examples/hello-world
    timeout 5s ../../target/release/backworks start &
    SERVER_PID=$!
    sleep 2
    
    if curl -s http://localhost:3002/hello > /dev/null; then
        echo -e "${GREEN}✅ Hello-world example works!${NC}"
    else
        echo -e "${RED}❌ Hello-world example failed${NC}"
    fi
    
    # Cleanup
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
    cd ../..
fi

echo -e "${GREEN}🚀 Ready to use Backworks!${NC}"
