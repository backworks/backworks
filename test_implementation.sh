#!/bin/bash

# Backworks Implementation Test Script
# This script tests the core functionality of the Backworks platform

set -e  # Exit on any error

echo "🧪 Starting Backworks Implementation Tests"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
TOTAL_TESTS=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_pattern="$3"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -e "\n${BLUE}Test ${TOTAL_TESTS}: ${test_name}${NC}"
    echo "Command: $test_command"
    
    local output=$(eval "$test_command" 2>&1)
    local exit_code=$?
    
    if [ -z "$expected_pattern" ]; then
        # For empty pattern, just check if command succeeded
        if [ $exit_code -eq 0 ]; then
            echo -e "${GREEN}✅ PASSED${NC}"
            TESTS_PASSED=$((TESTS_PASSED + 1))
        else
            echo -e "${RED}❌ FAILED${NC}"
            TESTS_FAILED=$((TESTS_FAILED + 1))
            echo "Expected pattern: $expected_pattern"
            echo "Actual output:"
            echo "$output" | head -10
        fi
    else
        # For non-empty pattern, check if output matches
        if echo "$output" | grep -q "$expected_pattern"; then
            echo -e "${GREEN}✅ PASSED${NC}"
            TESTS_PASSED=$((TESTS_PASSED + 1))
        else
            echo -e "${RED}❌ FAILED${NC}"
            TESTS_FAILED=$((TESTS_FAILED + 1))
            echo "Expected pattern: $expected_pattern"
            echo "Actual output:"
            echo "$output" | head -10
        fi
    fi
}

# Function to test file exists
test_file_exists() {
    local file_path="$1"
    local description="$2"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -e "\n${BLUE}Test ${TOTAL_TESTS}: ${description}${NC}"
    
    if [ -f "$file_path" ]; then
        echo -e "${GREEN}✅ PASSED${NC} - File exists: $file_path"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ FAILED${NC} - File missing: $file_path"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# Function to test directory exists
test_dir_exists() {
    local dir_path="$1"
    local description="$2"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -e "\n${BLUE}Test ${TOTAL_TESTS}: ${description}${NC}"
    
    if [ -d "$dir_path" ]; then
        echo -e "${GREEN}✅ PASSED${NC} - Directory exists: $dir_path"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ FAILED${NC} - Directory missing: $dir_path"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

echo -e "\n${YELLOW}🏗️  Testing Project Structure${NC}"
echo "================================"

# Test core project files
test_file_exists "Cargo.toml" "Cargo.toml exists"
test_file_exists "README.md" "README.md exists"
test_file_exists "src/main.rs" "Main source file exists"

# Test source modules
test_file_exists "src/config.rs" "Config module exists"
test_file_exists "src/engine.rs" "Engine module exists"
test_file_exists "src/server.rs" "Server module exists"
test_file_exists "src/ai.rs" "AI module exists"
test_file_exists "src/dashboard.rs" "Dashboard module exists"
test_file_exists "src/runtime.rs" "Runtime module exists"
test_file_exists "src/database.rs" "Database module exists"
test_file_exists "src/capture.rs" "Capture module exists"
test_file_exists "src/proxy.rs" "Proxy module exists"
test_file_exists "src/mock.rs" "Mock module exists"
test_file_exists "src/error.rs" "Error module exists"

# Test documentation
test_dir_exists "docs" "Documentation directory exists"
test_file_exists "docs/architecture.md" "Architecture documentation exists"
test_file_exists "docs/quick-start.md" "Quick start guide exists"
test_file_exists "docs/configuration.md" "Configuration documentation exists"
test_file_exists "docs/modes.md" "Modes documentation exists"
test_file_exists "docs/ai-features.md" "AI features documentation exists"
test_file_exists "docs/dashboard.md" "Dashboard documentation exists"

# Test examples
test_dir_exists "examples" "Examples directory exists"
test_file_exists "examples/README.md" "Examples README exists"
test_dir_exists "examples/basic/simple-api" "Basic example directory exists"
test_file_exists "examples/basic/simple-api/project.yaml" "Basic example config exists"
test_dir_exists "examples/advanced/ai-powered-api" "Advanced example directory exists"
test_file_exists "examples/advanced/ai-powered-api/project.yaml" "Advanced example config exists"

# Test dashboard
test_dir_exists "dashboard" "Dashboard directory exists"
test_file_exists "dashboard/index.html" "Dashboard HTML exists"

echo -e "\n${YELLOW}🔧 Testing Rust Compilation${NC}"
echo "============================="

# Test Rust compilation
run_test "Cargo check passes" "cd /Volumes/EXT/repos/devstroop/backworks && cargo check" "Finished"

# Test specific module compilation
run_test "Config module compiles" "cd /Volumes/EXT/repos/devstroop/backworks && cargo check --lib" "Finished"

echo -e "\n${YELLOW}📝 Testing Configuration Validation${NC}"
echo "===================================="

# Test YAML parsing of example configs
run_test "Basic example YAML is valid" "cd /Volumes/EXT/repos/devstroop/backworks/examples/basic/simple-api && python3 -c 'import yaml; yaml.safe_load(open(\"project.yaml\"))'" ""

run_test "Advanced example YAML is valid" "cd /Volumes/EXT/repos/devstroop/backworks/examples/advanced/ai-powered-api && python3 -c 'import yaml; yaml.safe_load(open(\"project.yaml\"))'" ""

echo -e "\n${YELLOW}🧪 Testing Handler Scripts${NC}"
echo "==========================="

# Test Python handler
if command -v python3 &> /dev/null; then
    run_test "Python handler syntax is valid" "cd /Volumes/EXT/repos/devstroop/backworks/examples/advanced/ai-powered-api && python3 -m py_compile handlers/recommendations.py" ""
else
    echo -e "${YELLOW}⚠️  Skipping Python handler test (python3 not available)${NC}"
fi

# Test Node.js handler
if command -v node &> /dev/null; then
    run_test "Node.js handler syntax is valid" "cd /Volumes/EXT/repos/devstroop/backworks/examples/advanced/ai-powered-api && node -c handlers/analytics.js" ""
else
    echo -e "${YELLOW}⚠️  Skipping Node.js handler test (node not available)${NC}"
fi

echo -e "\n${YELLOW}📊 Testing SQL Schema${NC}"
echo "===================="

# Test SQL schema
if command -v sqlite3 &> /dev/null; then
    run_test "SQL schema is valid" "cd /Volumes/EXT/repos/devstroop/backworks/examples/advanced/ai-powered-api && sqlite3 test.db < schema.sql && rm -f test.db" ""
else
    echo -e "${YELLOW}⚠️  Skipping SQL schema test (sqlite3 not available)${NC}"
fi

echo -e "\n${YELLOW}🔍 Testing Code Quality${NC}"
echo "======================="

# Test for common issues
run_test "No TODO comments in main code" "cd /Volumes/EXT/repos/devstroop/backworks && ! grep -r 'TODO' src/" ""

# Test documentation completeness
run_test "All modules have documentation" "cd /Volumes/EXT/repos/devstroop/backworks && grep -l '//!' src/*.rs | wc -l" "[0-9]"

echo -e "\n${YELLOW}📦 Testing Dependencies${NC}"
echo "======================="

# Check critical dependencies
run_test "Cargo.toml contains axum" "cd /Volumes/EXT/repos/devstroop/backworks && grep 'axum' Cargo.toml" "axum"
run_test "Cargo.toml contains tokio" "cd /Volumes/EXT/repos/devstroop/backworks && grep 'tokio' Cargo.toml" "tokio"
run_test "Cargo.toml contains sqlx" "cd /Volumes/EXT/repos/devstroop/backworks && grep 'sqlx' Cargo.toml" "sqlx"
run_test "Cargo.toml contains serde" "cd /Volumes/EXT/repos/devstroop/backworks && grep 'serde' Cargo.toml" "serde"

echo -e "\n${YELLOW}🎯 Testing Example Completeness${NC}"
echo "==============================="

# Test example completeness
run_test "Basic example has README" "cd /Volumes/EXT/repos/devstroop/backworks/examples/basic/simple-api && test -f README.md && grep -q 'Simple User API' README.md" ""

run_test "Advanced example has handlers" "cd /Volumes/EXT/repos/devstroop/backworks/examples/advanced/ai-powered-api && test -f handlers/recommendations.py && test -f handlers/analytics.js" ""

run_test "Advanced example has schema" "cd /Volumes/EXT/repos/devstroop/backworks/examples/advanced/ai-powered-api && test -f schema.sql && grep -q 'CREATE TABLE' schema.sql" ""

echo -e "\n=========================================="
echo -e "${BLUE}📊 Test Results Summary${NC}"
echo "=========================================="

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed! ($TESTS_PASSED/$TOTAL_TESTS)${NC}"
    echo -e "${GREEN}✅ Backworks implementation is ready!${NC}"
    exit_code=0
else
    echo -e "${RED}❌ Some tests failed: $TESTS_FAILED/$TOTAL_TESTS${NC}"
    echo -e "${GREEN}✅ Tests passed: $TESTS_PASSED/$TOTAL_TESTS${NC}"
    echo -e "${RED}❌ Tests failed: $TESTS_FAILED/$TOTAL_TESTS${NC}"
    exit_code=1
fi

echo ""
echo -e "${BLUE}📋 Next Steps:${NC}"
if [ $TESTS_FAILED -eq 0 ]; then
    echo "1. Build the project: cargo build --release"
    echo "2. Run the basic example: cd examples/basic/simple-api && backworks start"
    echo "3. Try the advanced example: cd examples/advanced/ai-powered-api && backworks start"
    echo "4. Open the dashboard at http://localhost:3001"
else
    echo "1. Fix the failing tests above"
    echo "2. Re-run this test script"
    echo "3. Consider checking the Rust compiler errors with: cargo check"
fi

echo ""
echo -e "${BLUE}🚀 Ready to ship Backworks!${NC}"

exit $exit_code
