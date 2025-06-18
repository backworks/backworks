#!/usr/bin/env node
/**
 * Node.js analytics handler for Backworks
 * Demonstrates real-time analytics and pattern detection
 */

const fs = require('fs');
const path = require('path');

// Simple in-memory analytics store (in production, use Redis or similar)
const analyticsStore = {
    requests: [],
    patterns: new Map(),
    aggregates: {
        totalRequests: 0,
        uniqueUsers: new Set(),
        topEndpoints: new Map(),
        responseTimeStats: { min: Infinity, max: 0, avg: 0, total: 0 }
    }
};

// Mock data for demonstration
function generateMockAnalytics() {
    const endpoints = ['/users', '/users/{id}', '/recommendations', '/analytics'];
    const userAgents = ['Mozilla/5.0', 'Chrome/120.0', 'Safari/17.0', 'PostmanRuntime'];
    
    // Generate some historical data
    for (let i = 0; i < 100; i++) {
        const timestamp = new Date(Date.now() - Math.random() * 86400000 * 7); // Last 7 days
        const endpoint = endpoints[Math.floor(Math.random() * endpoints.length)];
        const responseTime = Math.floor(Math.random() * 500) + 50;
        const userId = Math.floor(Math.random() * 10) + 1;
        
        analyticsStore.requests.push({
            id: `req_${i}`,
            timestamp,
            endpoint,
            method: 'GET',
            responseTime,
            statusCode: 200,
            userId,
            userAgent: userAgents[Math.floor(Math.random() * userAgents.length)],
            ipAddress: `192.168.1.${Math.floor(Math.random() * 255)}`
        });
    }
    
    // Update aggregates
    updateAggregates();
}

function updateAggregates() {
    analyticsStore.aggregates.totalRequests = analyticsStore.requests.length;
    
    // Calculate unique users
    analyticsStore.aggregates.uniqueUsers.clear();
    analyticsStore.requests.forEach(req => {
        if (req.userId) {
            analyticsStore.aggregates.uniqueUsers.add(req.userId);
        }
    });
    
    // Calculate top endpoints
    analyticsStore.aggregates.topEndpoints.clear();
    analyticsStore.requests.forEach(req => {
        const count = analyticsStore.aggregates.topEndpoints.get(req.endpoint) || 0;
        analyticsStore.aggregates.topEndpoints.set(req.endpoint, count + 1);
    });
    
    // Calculate response time stats
    const responseTimes = analyticsStore.requests.map(req => req.responseTime);
    if (responseTimes.length > 0) {
        analyticsStore.aggregates.responseTimeStats = {
            min: Math.min(...responseTimes),
            max: Math.max(...responseTimes),
            avg: responseTimes.reduce((a, b) => a + b, 0) / responseTimes.length,
            total: responseTimes.length
        };
    }
}

function detectPatterns() {
    const patterns = [];
    
    // Pattern 1: Peak usage times
    const hourlyStats = new Map();
    analyticsStore.requests.forEach(req => {
        const hour = new Date(req.timestamp).getHours();
        const stats = hourlyStats.get(hour) || { count: 0, responseTime: 0 };
        stats.count++;
        stats.responseTime += req.responseTime;
        hourlyStats.set(hour, stats);
    });
    
    // Find peak hour
    let peakHour = 0;
    let maxRequests = 0;
    hourlyStats.forEach((stats, hour) => {
        if (stats.count > maxRequests) {
            maxRequests = stats.count;
            peakHour = hour;
        }
    });
    
    patterns.push({
        type: 'peak_usage',
        description: `Peak usage occurs at ${peakHour}:00 with ${maxRequests} requests`,
        confidence: 0.85,
        actionable: true,
        suggestion: 'Consider scaling resources during peak hours'
    });
    
    // Pattern 2: Slow endpoints
    const endpointStats = new Map();
    analyticsStore.requests.forEach(req => {
        const stats = endpointStats.get(req.endpoint) || { count: 0, totalTime: 0 };
        stats.count++;
        stats.totalTime += req.responseTime;
        endpointStats.set(req.endpoint, stats);
    });
    
    endpointStats.forEach((stats, endpoint) => {
        const avgTime = stats.totalTime / stats.count;
        if (avgTime > 200) { // Slow threshold
            patterns.push({
                type: 'slow_endpoint',
                endpoint,
                description: `${endpoint} has average response time of ${avgTime.toFixed(1)}ms`,
                confidence: 0.9,
                actionable: true,
                suggestion: 'Consider optimizing this endpoint or adding caching'
            });
        }
    });
    
    // Pattern 3: User behavior patterns
    const userPatterns = new Map();
    analyticsStore.requests.forEach(req => {
        if (req.userId) {
            const pattern = userPatterns.get(req.userId) || { endpoints: new Set(), lastSeen: null };
            pattern.endpoints.add(req.endpoint);
            pattern.lastSeen = req.timestamp;
            userPatterns.set(req.userId, pattern);
        }
    });
    
    // Find power users
    userPatterns.forEach((pattern, userId) => {
        if (pattern.endpoints.size >= 3) {
            patterns.push({
                type: 'power_user',
                userId,
                description: `User ${userId} uses ${pattern.endpoints.size} different endpoints`,
                confidence: 0.75,
                actionable: true,
                suggestion: 'Consider offering premium features to this engaged user'
            });
        }
    });
    
    return patterns;
}

function generateInsights() {
    const insights = [];
    const now = new Date();
    const oneHourAgo = new Date(now.getTime() - 60 * 60 * 1000);
    
    // Recent activity
    const recentRequests = analyticsStore.requests.filter(req => 
        new Date(req.timestamp) > oneHourAgo
    );
    
    insights.push({
        type: 'activity_summary',
        title: 'Recent Activity',
        value: recentRequests.length,
        description: `${recentRequests.length} requests in the last hour`,
        trend: recentRequests.length > 10 ? 'up' : 'stable'
    });
    
    // Error rate
    const errorRequests = analyticsStore.requests.filter(req => req.statusCode >= 400);
    const errorRate = (errorRequests.length / analyticsStore.requests.length) * 100;
    
    insights.push({
        type: 'error_rate',
        title: 'Error Rate',
        value: `${errorRate.toFixed(2)}%`,
        description: `${errorRequests.length} errors out of ${analyticsStore.requests.length} total requests`,
        trend: errorRate > 5 ? 'down' : 'stable'
    });
    
    // Performance insight
    const avgResponseTime = analyticsStore.aggregates.responseTimeStats.avg;
    insights.push({
        type: 'performance',
        title: 'Average Response Time',
        value: `${avgResponseTime.toFixed(1)}ms`,
        description: 'Overall API performance',
        trend: avgResponseTime < 200 ? 'up' : avgResponseTime > 500 ? 'down' : 'stable'
    });
    
    return insights;
}

function handleAnalyticsRequest(request) {
    const { path, query } = request;
    
    if (path.includes('/analytics/users')) {
        return {
            status: 200,
            headers: {
                'Content-Type': 'application/json',
                'X-Powered-By': 'Backworks-Analytics-NodeJS'
            },
            body: {
                summary: {
                    totalRequests: analyticsStore.aggregates.totalRequests,
                    uniqueUsers: analyticsStore.aggregates.uniqueUsers.size,
                    avgResponseTime: analyticsStore.aggregates.responseTimeStats.avg.toFixed(1),
                    timeRange: '7 days'
                },
                topEndpoints: Array.from(analyticsStore.aggregates.topEndpoints.entries())
                    .sort((a, b) => b[1] - a[1])
                    .slice(0, 5)
                    .map(([endpoint, count]) => ({ endpoint, requests: count })),
                responseTimeStats: analyticsStore.aggregates.responseTimeStats,
                insights: generateInsights(),
                generatedAt: new Date().toISOString()
            }
        };
    }
    
    if (path.includes('/analytics/patterns')) {
        return {
            status: 200,
            headers: {
                'Content-Type': 'application/json',
                'X-Powered-By': 'Backworks-Analytics-NodeJS'
            },
            body: {
                patterns: detectPatterns(),
                metadata: {
                    analysisPeriod: '7 days',
                    totalRequests: analyticsStore.aggregates.totalRequests,
                    patternDetectionAlgorithm: 'statistical_analysis_v1',
                    confidence: 0.8
                },
                recommendations: [
                    {
                        priority: 'high',
                        category: 'performance',
                        action: 'Enable caching for frequently accessed endpoints',
                        expectedImpact: '30-50% response time improvement'
                    },
                    {
                        priority: 'medium',
                        category: 'scaling',
                        action: 'Consider implementing rate limiting for peak hours',
                        expectedImpact: 'Better resource utilization'
                    },
                    {
                        priority: 'low',
                        category: 'monitoring',
                        action: 'Set up alerts for endpoints with >500ms response time',
                        expectedImpact: 'Proactive issue detection'
                    }
                ],
                generatedAt: new Date().toISOString()
            }
        };
    }
    
    // Default response
    return {
        status: 404,
        headers: { 'Content-Type': 'application/json' },
        body: {
            error: 'Analytics endpoint not found',
            availableEndpoints: [
                'GET /analytics/users',
                'GET /analytics/patterns'
            ]
        }
    };
}

function main() {
    try {
        // Initialize mock data
        generateMockAnalytics();
        
        // Read request from stdin
        let inputData = '';
        
        if (process.stdin.isTTY) {
            // Running directly, use mock request
            inputData = JSON.stringify({
                method: 'GET',
                path: '/analytics/users',
                query: {},
                headers: {}
            });
        } else {
            // Read from stdin
            const chunks = [];
            process.stdin.on('data', chunk => chunks.push(chunk));
            process.stdin.on('end', () => {
                inputData = Buffer.concat(chunks).toString();
                processRequest(inputData);
            });
            return;
        }
        
        processRequest(inputData);
        
    } catch (error) {
        const errorResponse = {
            status: 500,
            headers: { 'Content-Type': 'application/json' },
            body: {
                error: 'Internal server error',
                message: error.message,
                handler: 'nodejs_analytics'
            }
        };
        console.log(JSON.stringify(errorResponse));
    }
}

function processRequest(inputData) {
    try {
        const request = inputData.trim() ? JSON.parse(inputData) : {};
        const response = handleAnalyticsRequest(request);
        console.log(JSON.stringify(response));
    } catch (error) {
        const errorResponse = {
            status: 500,
            headers: { 'Content-Type': 'application/json' },
            body: {
                error: 'Failed to process request',
                message: error.message,
                handler: 'nodejs_analytics'
            }
        };
        console.log(JSON.stringify(errorResponse));
    }
}

// Run if called directly
if (require.main === module) {
    main();
}

module.exports = {
    handleAnalyticsRequest,
    detectPatterns,
    generateInsights
};
