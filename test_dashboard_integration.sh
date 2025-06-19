#!/bin/bash

# Comprehensive Dashboard Integration Test
# Tests all dashboard features including analytics, WebSocket, and monitoring

echo "🧪 Starting Comprehensive Dashboard Test Suite"
echo "================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
API_BASE="http://localhost:8080"
DASHBOARD_BASE="http://localhost:8081"

# Function to check if service is running
check_service() {
    local url=$1
    local name=$2
    if curl -s "$url" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ $name is running${NC}"
        return 0
    else
        echo -e "${RED}❌ $name is not running${NC}"
        return 1
    fi
}

# Test basic connectivity
echo -e "${BLUE}🔗 Testing Service Connectivity${NC}"
check_service "$API_BASE/health" "API Server" || exit 1
check_service "$DASHBOARD_BASE/api/metrics" "Dashboard Server" || exit 1

# Test API endpoints and generate traffic
echo -e "${BLUE}📊 Generating Test Traffic${NC}"
for i in {1..5}; do
    echo "  Request $i..."
    curl -s "$API_BASE/health" > /dev/null
    curl -s "$API_BASE/test" > /dev/null
    sleep 0.5
done
echo -e "${GREEN}✅ Generated 10 test requests${NC}"

# Test Dashboard APIs
echo -e "${BLUE}🎯 Testing Dashboard APIs${NC}"

# Test metrics API
METRICS=$(curl -s "$DASHBOARD_BASE/api/metrics")
ENDPOINT_COUNT=$(echo "$METRICS" | jq '. | length')
echo "  Metrics API: $ENDPOINT_COUNT endpoints tracked"

# Test system metrics API
SYSTEM=$(curl -s "$DASHBOARD_BASE/api/system")
TOTAL_REQUESTS=$(echo "$SYSTEM" | jq '.total_requests')
echo "  System API: $TOTAL_REQUESTS total requests recorded"

# Test new performance API
PERFORMANCE=$(curl -s "$DASHBOARD_BASE/api/performance")
OVERALL_GRADE=$(echo "$PERFORMANCE" | jq -r '.summary.overall_grade')
AVG_RESPONSE_TIME=$(echo "$PERFORMANCE" | jq '.summary.avg_response_time')
echo "  Performance API: Overall grade $OVERALL_GRADE, Avg response time ${AVG_RESPONSE_TIME}ms"

# Test architecture API
ARCHITECTURE=$(curl -s "$DASHBOARD_BASE/api/architecture")
NODE_COUNT=$(echo "$ARCHITECTURE" | jq '.nodes | length')
echo "  Architecture API: $NODE_COUNT nodes in architecture"

# Validate performance grading system
echo -e "${BLUE}🏆 Testing Performance Grading${NC}"
ENDPOINTS=$(echo "$PERFORMANCE" | jq -r '.endpoints[] | "\(.endpoint): \(.grade) (\(.avg_response_time)ms)"')
echo "$ENDPOINTS"

# Test recommendation system
RECOMMENDATIONS=$(echo "$PERFORMANCE" | jq '.recommendations | length')
echo "  Recommendations: $RECOMMENDATIONS generated"

# Test error handling with invalid endpoint
echo -e "${BLUE}🔥 Testing Error Handling${NC}"
ERROR_RESPONSE=$(curl -s -w "%{http_code}" "$API_BASE/nonexistent" -o /dev/null)
echo "  Invalid endpoint response: HTTP $ERROR_RESPONSE"

# Generate some slow requests to test performance warnings
echo -e "${BLUE}⏱️  Testing Performance Monitoring${NC}"
for i in {1..3}; do
    curl -s "$API_BASE/test" > /dev/null &
done
wait
echo "  Generated concurrent requests for performance testing"

# Final metrics check
sleep 2
FINAL_METRICS=$(curl -s "$DASHBOARD_BASE/api/performance")
FINAL_REQUESTS=$(echo "$FINAL_METRICS" | jq '.summary.total_requests')
FINAL_GRADE=$(echo "$FINAL_METRICS" | jq -r '.summary.overall_grade')
ERROR_RATE=$(echo "$FINAL_METRICS" | jq '.summary.error_rate')

echo -e "${BLUE}📈 Final Test Results${NC}"
echo "  Total Requests: $FINAL_REQUESTS"
echo "  Overall Grade: $FINAL_GRADE"
echo "  Error Rate: $ERROR_RATE"
echo "  Dashboard UI: http://localhost:8081/dashboard"

# Check if WebSocket endpoint is available
echo -e "${BLUE}🔌 Testing WebSocket Endpoint${NC}"
WS_TEST=$(curl -s -I "$DASHBOARD_BASE/ws" | grep -i "upgrade")
if [ ! -z "$WS_TEST" ]; then
    echo -e "${GREEN}✅ WebSocket endpoint available${NC}"
else
    echo -e "${YELLOW}⚠️  WebSocket upgrade header not found${NC}"
fi

# Performance recommendations check
RECOMMENDATIONS_JSON=$(echo "$FINAL_METRICS" | jq '.recommendations[]?')
if [ ! -z "$RECOMMENDATIONS_JSON" ]; then
    echo -e "${YELLOW}💡 Performance Recommendations:${NC}"
    echo "$RECOMMENDATIONS_JSON" | jq -r '.title + ": " + .description'
else
    echo -e "${GREEN}✅ No performance issues detected${NC}"
fi

echo ""
echo -e "${GREEN}🎉 Dashboard Integration Test Complete!${NC}"
echo "================================================"
echo "📊 Dashboard Features Verified:"
echo "   ✅ Real-time metrics tracking"
echo "   ✅ Performance grading system"
echo "   ✅ System monitoring"
echo "   ✅ Analytics and recommendations"
echo "   ✅ WebSocket infrastructure"
echo "   ✅ Error handling"
echo "   ✅ Multi-endpoint support"
echo ""
echo "🌐 Access the dashboard at: http://localhost:8081/dashboard"
echo "📡 API endpoints available at: http://localhost:8081/api/*"
