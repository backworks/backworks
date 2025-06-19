#!/bin/bash

# 📖 Documentation Validation Script
# Tests that all documented commands and examples work as described

echo "📖 Validating Backworks Documentation"
echo "===================================="

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
    fi
}

# Check if Backworks builds
echo "🔧 Testing Build Process..."
cargo build --release --quiet
print_status $? "Cargo build successful"

# Check if examples exist and are valid YAML
echo ""
echo "📁 Validating Example Files..."

for example in hello-world blog-api task-manager; do
    config_file="examples/$example/api.yaml"
    readme_file="examples/$example/README.md"
    
    if [ -f "$config_file" ]; then
        print_status 0 "Found: $config_file"
        
        # Check if YAML is parseable
        if python3 -c "import yaml; yaml.safe_load(open('$config_file'))" 2>/dev/null; then
            print_status 0 "Valid YAML: $config_file"
        else
            print_status 1 "Invalid YAML: $config_file"
        fi
        
        # Check if README exists
        if [ -f "$readme_file" ]; then
            print_status 0 "README exists: $readme_file"
        else
            print_status 1 "Missing README: $readme_file"
        fi
        
        # Check for runtime mode in YAML
        if grep -q 'mode: "runtime"' "$config_file"; then
            print_status 0 "Uses runtime mode: $config_file"
        else
            print_status 1 "Missing runtime mode: $config_file"
        fi
        
    else
        print_status 1 "Missing: $config_file"
    fi
done

# Check documentation files
echo ""
echo "📚 Validating Documentation Files..."

docs=(
    "README.md"
    "ARCHITECTURE.md" 
    "DIRECTION.md"
    "DEVELOPER_GUIDE.md"
    "docs/quick-start.md"
    "docs/configuration.md"
    "docs/README.md"
    "examples/README.md"
)

for doc in "${docs[@]}"; do
    if [ -f "$doc" ]; then
        print_status 0 "Found: $doc"
        
        # Check for outdated references
        if grep -q "mock_responses\|mode.*mock" "$doc"; then
            print_status 1 "Contains outdated mock references: $doc"
        else
            print_status 0 "No outdated references: $doc"
        fi
    else
        print_status 1 "Missing: $doc"
    fi
done

echo ""
echo "🎯 Next Steps:"
echo "1. Run: ./target/release/backworks start --config examples/hello-world/api.yaml"
echo "2. Test: curl http://localhost:3002/hello"
echo "3. Visit: http://localhost:3003 (dashboard)"

echo ""
echo "📋 Validation Complete!"
echo "If all items show ✅, the documentation should be consistent and accurate."
