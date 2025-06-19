import { component$, useSignal, useTask$, $ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";
import { DashboardLayout } from "~/components/layout";
import { MetricCard } from "~/components/metric-card";
import { RequestLogViewer } from "~/components/request-log-viewer";
import { ApiTester } from "~/components/api-tester";
import { BlueprintDesigner } from "~/components/blueprint-designer";
import { type SystemMetrics, type EndpointMetric } from "~/services/api";

export default component$(() => {
  const systemMetrics = useSignal<SystemMetrics>({
    uptime: '0m 0s',
    total_requests: 0,
    active_connections: 0,
    cpu_usage: 0,
    memory_usage: 0,
    status: 'Starting...'
  });

  const endpointMetrics = useSignal<EndpointMetric[]>([]);
  const lastUpdated = useSignal<string>('');
  const activeTab = useSignal<string>('overview');
  const isLoading = useSignal<boolean>(false);

  // Enhanced service configuration
  const serviceConfig = {
    name: "Backworks Declarative Platform",
    description: "Modern declarative backend platform with visual service schematic designer",
    version: "2.0.0",
    mode: "runtime",
    server: { host: "0.0.0.0", port: 3002 },
    dashboard: { enabled: true, port: 3003 },
    runtime: { 
      language: "javascript", 
      handler: "enhanced", 
      timeout: 10000,
      features: ["hot-reload", "auto-scaling", "monitoring"]
    },
    endpoints: {
      hello: {
        path: "/hello",
        methods: ["GET"],
        description: "Welcome endpoint demonstrating the platform capabilities",
        runtime: {
          language: "javascript",
          handler: `function handler(req, res) {
  return {
    status: 200,
    headers: { "Content-Type": "application/json" },
    body: {
      message: "Welcome to Backworks - Declarative Backend Platform",
      version: "2.0.0",
      features: ["Visual Designer", "Real-time Monitoring", "Auto-scaling"],
      timestamp: new Date().toISOString(),
      platform: "backworks-studio"
    }
  };
}`
        }
      },
      echo: {
        path: "/echo",
        methods: ["POST"],
        description: "Echo service for testing request/response patterns",
        runtime: {
          language: "javascript",
          handler: `function handler(req, res) {
  return {
    status: 200,
    headers: { "Content-Type": "application/json" },
    body: {
      echo: req.body,
      metadata: {
        method: req.method,
        headers: req.headers,
        timestamp: new Date().toISOString(),
        processing_time: Math.random() * 100
      }
    }
  };
}`
        }
      }
    }
  };

  // Simulate real-time updates
  useTask$(({ track, cleanup }) => {
    track(() => activeTab.value);
    
    const updateMetrics = () => {
      systemMetrics.value = {
        uptime: `${Math.floor(Math.random() * 24)}h ${Math.floor(Math.random() * 60)}m`,
        total_requests: Math.floor(Math.random() * 10000) + 1000,
        active_connections: Math.floor(Math.random() * 50) + 5,
        cpu_usage: Math.floor(Math.random() * 30) + 10,
        memory_usage: Math.floor(Math.random() * 40) + 20,
        status: 'Running'
      };

      endpointMetrics.value = [
        {
          path: '/hello',
          method: 'GET',
          request_count: Math.floor(Math.random() * 1000) + 100,
          avg_response_time: Math.floor(Math.random() * 50) + 10,
          last_accessed: new Date(Date.now() - Math.random() * 3600000).toISOString()
        },
        {
          path: '/echo',
          method: 'POST',
          request_count: Math.floor(Math.random() * 500) + 50,
          avg_response_time: Math.floor(Math.random() * 30) + 15,
          last_accessed: new Date(Date.now() - Math.random() * 1800000).toISOString()
        }
      ];

      lastUpdated.value = new Date().toLocaleTimeString();
    };

    updateMetrics();
    const interval = setInterval(updateMetrics, 5000);
    
    cleanup(() => clearInterval(interval));
  });

  const refreshData = $(() => {
    isLoading.value = true;
    setTimeout(() => {
      isLoading.value = false;
    }, 1000);
  });

  return (
    <DashboardLayout>
      {/* Header with Quick Stats */}
      <div class="mb-8">
        <div class="flex items-center justify-between mb-6">
          <div>
            <h1 class="text-3xl font-bold bg-gradient-to-r from-gray-900 to-gray-600 bg-clip-text text-transparent dark:from-white dark:to-gray-300">
              Backworks Studio
            </h1>
            <p class="text-gray-600 dark:text-gray-400 mt-1">
              Declarative Backend Platform - Visual service designer and runtime dashboard
            </p>
          </div>
          
          <div class="flex items-center space-x-4">
            <div class="hidden sm:flex items-center space-x-4 text-sm">
              <div class="flex items-center space-x-2">
                <div class="h-3 w-3 bg-green-400 rounded-full animate-pulse"></div>
                <span class="text-gray-600 dark:text-gray-400">Runtime Active</span>
              </div>
              <div class="text-gray-400 dark:text-gray-500">•</div>
              <span class="text-gray-600 dark:text-gray-400">
                Last updated: {lastUpdated.value || 'Never'}
              </span>
            </div>
            
            <button
              onClick$={refreshData}
              disabled={isLoading.value}
              class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors flex items-center space-x-2"
            >
              <svg class={`h-4 w-4 ${isLoading.value ? 'animate-spin' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              <span>{isLoading.value ? 'Refreshing...' : 'Refresh'}</span>
            </button>
          </div>
        </div>

        {/* Quick Stats Bar */}
        <div class="grid grid-cols-2 sm:grid-cols-4 lg:grid-cols-6 gap-4">
          <div class="bg-white dark:bg-gray-800 rounded-xl p-4 border border-gray-200 dark:border-gray-700">
            <div class="text-2xl font-bold text-blue-600 dark:text-blue-400">
              {systemMetrics.value.total_requests.toLocaleString()}
            </div>
            <div class="text-sm text-gray-500 dark:text-gray-400">Total Requests</div>
          </div>
          
          <div class="bg-white dark:bg-gray-800 rounded-xl p-4 border border-gray-200 dark:border-gray-700">
            <div class="text-2xl font-bold text-green-600 dark:text-green-400">
              {systemMetrics.value.active_connections}
            </div>
            <div class="text-sm text-gray-500 dark:text-gray-400">Active Connections</div>
          </div>
          
          <div class="bg-white dark:bg-gray-800 rounded-xl p-4 border border-gray-200 dark:border-gray-700">
            <div class="text-2xl font-bold text-purple-600 dark:text-purple-400">
              {systemMetrics.value.uptime}
            </div>
            <div class="text-sm text-gray-500 dark:text-gray-400">Uptime</div>
          </div>
          
          <div class="bg-white dark:bg-gray-800 rounded-xl p-4 border border-gray-200 dark:border-gray-700">
            <div class="text-2xl font-bold text-orange-600 dark:text-orange-400">
              {systemMetrics.value.cpu_usage}%
            </div>
            <div class="text-sm text-gray-500 dark:text-gray-400">CPU Usage</div>
          </div>
          
          <div class="bg-white dark:bg-gray-800 rounded-xl p-4 border border-gray-200 dark:border-gray-700">
            <div class="text-2xl font-bold text-red-600 dark:text-red-400">
              {systemMetrics.value.memory_usage}%
            </div>
            <div class="text-sm text-gray-500 dark:text-gray-400">Memory Usage</div>
          </div>
          
          <div class="bg-white dark:bg-gray-800 rounded-xl p-4 border border-gray-200 dark:border-gray-700">
            <div class="text-2xl font-bold text-gray-600 dark:text-gray-400">
              {Object.keys(serviceConfig.endpoints).length}
            </div>
            <div class="text-sm text-gray-500 dark:text-gray-400">Endpoints</div>
          </div>
        </div>
      </div>

      {/* Tab Navigation */}
      <div class="mb-8">
        <div class="border-b border-gray-200 dark:border-gray-700">
          <nav class="-mb-px flex space-x-8">
            {[
              { id: 'overview', label: 'Service Overview', icon: '📊' },
              { id: 'designer', label: 'Blueprint Designer', icon: '🎨' },
              { id: 'endpoints', label: 'API Endpoints', icon: '🔗' },
              { id: 'metrics', label: 'Performance', icon: '📈' },
              { id: 'logs', label: 'Request Logs', icon: '📋' },
              { id: 'tester', label: 'API Tester', icon: '🧪' }
            ].map((tab) => (
              <button
                key={tab.id}
                onClick$={() => { activeTab.value = tab.id; }}
                class={`flex items-center space-x-2 py-4 px-1 border-b-2 font-medium text-sm transition-colors ${
                  activeTab.value === tab.id
                    ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'
                }`}
              >
                <span class="text-lg">{tab.icon}</span>
                <span>{tab.label}</span>
              </button>
            ))}
          </nav>
        </div>
      </div>

      {/* Tab Content */}
      <div class="min-h-96">
        {activeTab.value === 'overview' && (
          <div class="space-y-6">
            {/* Service Information Card */}
            <div class="bg-white dark:bg-gray-800 rounded-xl p-6 border border-gray-200 dark:border-gray-700">
              <div class="flex items-start justify-between mb-4">
                <div>
                  <h3 class="text-xl font-semibold text-gray-900 dark:text-white">
                    {serviceConfig.name}
                  </h3>
                  <p class="text-gray-600 dark:text-gray-400 mt-1">
                    {serviceConfig.description}
                  </p>
                </div>
                <div class="text-right">
                  <div class="text-sm font-medium text-gray-900 dark:text-white">
                    Version {serviceConfig.version}
                  </div>
                  <div class="text-sm text-gray-500 dark:text-gray-400">
                    {serviceConfig.mode} mode
                  </div>
                </div>
              </div>
              
              <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-6">
                <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4">
                  <div class="text-blue-600 dark:text-blue-400 font-medium">Runtime Features</div>
                  <div class="text-sm text-blue-800 dark:text-blue-300 mt-1">
                    {serviceConfig.runtime.features.join(', ')}
                  </div>
                </div>
                <div class="bg-green-50 dark:bg-green-900/20 rounded-lg p-4">
                  <div class="text-green-600 dark:text-green-400 font-medium">Server Configuration</div>
                  <div class="text-sm text-green-800 dark:text-green-300 mt-1">
                    {serviceConfig.server.host}:{serviceConfig.server.port}
                  </div>
                </div>
                <div class="bg-purple-50 dark:bg-purple-900/20 rounded-lg p-4">
                  <div class="text-purple-600 dark:text-purple-400 font-medium">Language Runtime</div>
                  <div class="text-sm text-purple-800 dark:text-purple-300 mt-1">
                    {serviceConfig.runtime.language} ({serviceConfig.runtime.handler})
                  </div>
                </div>
              </div>
            </div>

            {/* System Metrics */}
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              <MetricCard
                title="System Status"
                value={systemMetrics.value.status}
                icon="🟢"
              />
              <MetricCard
                title="Uptime"
                value={systemMetrics.value.uptime}
                icon="⏱️"
              />
              <MetricCard
                title="CPU Usage"
                value={`${systemMetrics.value.cpu_usage}%`}
                icon="💻"
              />
              <MetricCard
                title="Memory Usage"
                value={`${systemMetrics.value.memory_usage}%`}
                icon="🧠"
              />
            </div>

            {/* Endpoints Overview */}
            <div class="bg-white dark:bg-gray-800 rounded-xl p-6 border border-gray-200 dark:border-gray-700">
              <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                Active Endpoints
              </h3>
              <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
                {endpointMetrics.value.map((endpoint) => (
                  <div key={`${endpoint.path}-${endpoint.method}`} class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
                    <div class="flex items-center justify-between mb-2">
                      <div class="flex items-center space-x-2">
                        <span class="px-2 py-1 text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300 rounded">
                          {endpoint.method}
                        </span>
                        <span class="font-medium text-gray-900 dark:text-white">{endpoint.path}</span>
                      </div>
                    </div>
                    <div class="text-sm text-gray-600 dark:text-gray-400">
                      Avg response: {endpoint.avg_response_time}ms
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab.value === 'designer' && (
          <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden">
            <BlueprintDesigner />
          </div>
        )}

        {activeTab.value === 'endpoints' && (
          <div class="space-y-4">
            {Object.entries(serviceConfig.endpoints).map(([key, endpoint]) => (
              <div
                key={key}
                class="bg-white dark:bg-gray-800 rounded-xl p-6 border border-gray-200 dark:border-gray-700"
              >
                <div class="flex items-start justify-between mb-4">
                  <div>
                    <div class="flex items-center space-x-3 mb-2">
                      <span class="px-3 py-1 text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300 rounded-full">
                        {endpoint.methods[0]}
                      </span>
                      <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                        {endpoint.path}
                      </h3>
                    </div>
                    <p class="text-gray-600 dark:text-gray-400">
                      {endpoint.description}
                    </p>
                  </div>
                  <div class="text-sm text-gray-500 dark:text-gray-400">
                    {endpoint.runtime.language}
                  </div>
                </div>
                
                <div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
                  <div class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-2">
                    Handler Code:
                  </div>
                  <pre class="text-sm text-gray-800 dark:text-gray-200 font-mono overflow-x-auto">
                    {endpoint.runtime.handler}
                  </pre>
                </div>
              </div>
            ))}
          </div>
        )}

        {activeTab.value === 'metrics' && (
          <div class="bg-white dark:bg-gray-800 rounded-xl p-6 border border-gray-200 dark:border-gray-700">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              System Performance
            </h3>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <h4 class="text-md font-medium text-gray-900 dark:text-white mb-2">Resource Usage</h4>
                <div class="space-y-2">
                  <div class="flex justify-between">
                    <span class="text-gray-600 dark:text-gray-400">CPU Usage</span>
                    <span class="font-medium">{systemMetrics.value.cpu_usage}%</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-gray-600 dark:text-gray-400">Memory Usage</span>
                    <span class="font-medium">{systemMetrics.value.memory_usage}%</span>
                  </div>
                </div>
              </div>
              <div>
                <h4 class="text-md font-medium text-gray-900 dark:text-white mb-2">Request Stats</h4>
                <div class="space-y-2">
                  <div class="flex justify-between">
                    <span class="text-gray-600 dark:text-gray-400">Total Requests</span>
                    <span class="font-medium">{systemMetrics.value.total_requests.toLocaleString()}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-gray-600 dark:text-gray-400">Active Connections</span>
                    <span class="font-medium">{systemMetrics.value.active_connections}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab.value === 'logs' && (
          <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700">
            <RequestLogViewer />
          </div>
        )}

        {activeTab.value === 'tester' && (
          <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700">
            <ApiTester />
          </div>
        )}
      </div>
    </DashboardLayout>
  );
});

export const head: DocumentHead = {
  title: "Backworks Studio - Declarative Backend Platform",
  meta: [
    {
      name: "description",
      content: "Backworks Studio dashboard - Visual designer and monitoring for your declarative backend platform",
    },
  ],
};
