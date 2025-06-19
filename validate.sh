#!/bin/bash

# 🧪 Backworks Validation Script
# Tests current state and identifies issues

echo "🧪 Testing Backworks Current State"
echo "================================="

# Check if build works
echo "📦 Building Backworks..."
if cargo build --release --quiet; then
    echo "✅ Build successful"
else
    echo "❌ Build failed"
    exit 1
fi

# Check if examples exist and are valid YAML
echo ""
echo "📝 Validating Examples..."

for example in hello-world blog-api task-manager; do
    config_file="examples/$example/api.yaml"
    if [ -f "$config_file" ]; then
        echo "✅ Found: $config_file"
        # Basic YAML validation (check if it's parseable)
        if python3 -c "import yaml; yaml.safe_load(open('$config_file'))" 2>/dev/null; then
            echo "✅ Valid YAML: $config_file"
        else
            echo "❌ Invalid YAML: $config_file"
        fi
    else
        echo "❌ Missing: $config_file"
    fi
done

echo ""
echo "🎯 Next Steps:"
echo "1. Test runtime execution with examples"
echo "2. Validate dashboard functionality" 
echo "3. Update documentation based on what works"
echo "4. Create automated tests"

echo ""
echo "📋 Run this to test an example:"
echo "./target/release/backworks start --config examples/hello-world/api.yaml"
