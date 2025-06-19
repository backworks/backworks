#!/bin/bash

# Backworks Quick Start Script
# This script helps you get started with Backworks quickly

set -e

echo "🚀 Backworks Quick Start"
echo "========================"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check if Node.js is installed
if ! command -v npm &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js from https://nodejs.org/"
    exit 1
fi

echo "✅ Prerequisites found"

# Build Backworks
echo ""
echo "🔨 Building Backworks..."
cargo build --release

# Build Studio
echo ""
echo "🎨 Building Studio interface..."
cd studio
npm install --silent
npm run build --silent
cd ..

echo ""
echo "🎉 Backworks is ready!"
echo ""
echo "📚 Try an example:"
echo "   cd examples/hello-world"
echo "   ../../target/release/backworks start -c blueprint.yaml"
echo ""
echo "📱 Then open your browser to:"
echo "   🌐 API: http://localhost:3002"
echo "   🎨 Studio: http://localhost:3003"
echo ""
