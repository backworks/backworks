import { component$, useSignal, useTask$ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";
import { DashboardLayout } from "~/components/layout";
import { MetricCard } from "~/components/metric-card";
import { EndpointCard } from "~/components/endpoint-card";
import { ConfigViewer } from "~/components/config-viewer";
import { RequestLogViewer } from "~/components/request-log-viewer";
import { ApiTester } from "~/components/api-tester";
import { BackworksAPI, type SystemMetrics, type EndpointMetric } from "~/services/api";

export default component$(() => {
  const systemMetrics = useSignal<SystemMetrics>({
    uptime: '0m 0s',
    total_requests: 0,
    active_connections: 0,
    cpu_usage: 0,
    memory_usage: 0,
    status: 'Loading...'
  });

  const endpointMetrics = useSignal<EndpointMetric[]>([]);
  const lastUpdated = useSignal<string>('');
  const activeTab = useSignal<string>('overview');

  // Mock configuration data (replace with actual API call)
  const mockConfig = {
    name: "Hello World API",
    description: "The simplest possible Backworks API",
    mode: "runtime",
    server: { host: "0.0.0.0", port: 3002 },
    dashboard: { enabled: true, port: 3003 },
    runtime: { language: "javascript", handler: "simple", timeout: 5000 },
    endpoints: {
      hello: {
        path: "/hello",
        methods: ["GET"],
        description: "Say hello",
        runtime: {
          language: "javascript",
          handler: "function handler(req, res) { return { status: 200, body: { message: 'Hello, World!', timestamp: new Date().toISOString() } }; }"
        }
      },
      echo: {
        path: "/echo",
        methods: ["POST"],
        description: "Echo back the request data",
        runtime: {
          language: "javascript",
          handler: "function handler(req, res) { return { status: 200, body: { data: req.body, echo: 'You sent some data', received_at: new Date().toISOString() } }; }"
        }
      }
    }
  };

  // Fetch data on component mount and set up polling
  useTask$(async () => {
    const fetchData = async () => {
      try {
        const [sysMetrics, endMetrics] = await Promise.all([
          BackworksAPI.getSystemMetrics(),
          BackworksAPI.getEndpointMetrics()
        ]);
        
        systemMetrics.value = sysMetrics;
        endpointMetrics.value = endMetrics;
        lastUpdated.value = new Date().toLocaleTimeString();
      } catch (error) {
        console.error('Error fetching dashboard data:', error);
      }
    };

    // Initial fetch
    await fetchData();

    // Set up polling every 30 seconds
    const interval = setInterval(fetchData, 30000);
    return () => clearInterval(interval);
  });

  const memoryFormatted = BackworksAPI.formatMemory(systemMetrics.value.memory_usage);
  const isHealthy = systemMetrics.value.status === 'Running';

  const tabs = [
    { id: 'overview', name: 'Overview', icon: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2-2z' },
    { id: 'config', name: 'Configuration', icon: 'M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4' },
    { id: 'logs', name: 'Live Logs', icon: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z' },
    { id: 'test', name: 'API Tester', icon: 'M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.367 2.684 3 3 0 00-5.367-2.684z' }
  ];

  return (
    <DashboardLayout>
      <div class="p-6">
        {/* Header */}
        <div class="mb-8">
          <div class="flex items-center justify-between">
            <div>
              <h1 class="text-3xl font-bold text-gray-900 dark:text-white">
                🚀 Backworks Dashboard
              </h1>
              <p class="mt-2 text-gray-600 dark:text-gray-400">
                Configuration-driven backend platform • YAML → API
              </p>
            </div>
            <div class="flex items-center space-x-4">
              <div class={`flex items-center px-3 py-1 rounded-full text-sm font-medium ${
                isHealthy 
                  ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                  : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
              }`}>
                <div class={`w-2 h-2 rounded-full mr-2 ${
                  isHealthy ? 'bg-green-500 animate-pulse' : 'bg-red-500'
                }`}></div>
                {systemMetrics.value.status}
              </div>
              {lastUpdated.value && (
                <span class="text-sm text-gray-500 dark:text-gray-400">
                  Last updated: {lastUpdated.value}
                </span>
              )}
            </div>
          </div>
        </div>

        {/* Tabs */}
        <div class="mb-6">
          <div class="border-b border-gray-200 dark:border-gray-700">
            <nav class="-mb-px flex space-x-8">
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  type="button"
                  onClick$={() => { activeTab.value = tab.id; }}
                  class={`flex items-center py-2 px-1 border-b-2 font-medium text-sm ${
                    activeTab.value === tab.id
                      ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'
                  }`}
                >
                  <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={tab.icon} />
                  </svg>
                  {tab.name}
                </button>
              ))}
            </nav>
          </div>
        </div>

        {/* Tab Content */}
        {activeTab.value === 'overview' && (
          <div class="space-y-6">
            {/* System Metrics */}
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <MetricCard
                title="System Status"
                value={systemMetrics.value.status}
                icon='<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>'
                changeType={isHealthy ? 'positive' : 'negative'}
              />
              <MetricCard
                title="Total Requests"
                value={systemMetrics.value.total_requests.toLocaleString()}
                icon='<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/></svg>'
                changeType="positive"
              />
              <MetricCard
                title="Uptime"
                value={systemMetrics.value.uptime}
                icon='<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>'
                changeType="positive"
              />
              <MetricCard
                title="Memory Usage"
                value={memoryFormatted}
                icon='<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4"/></svg>'
                changeType="neutral"
              />
            </div>

            {/* Endpoints Grid */}
            <div>
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
                API Endpoints
              </h2>
              {endpointMetrics.value.length > 0 ? (
                <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
                  {endpointMetrics.value.map((endpoint, index) => (
                    <EndpointCard
                      key={index}
                      method={endpoint.method}
                      path={endpoint.path}
                      requestCount={endpoint.request_count}
                      avgResponseTime={endpoint.avg_response_time}
                      lastAccessed={endpoint.last_accessed}
                    />
                  ))}
                </div>
              ) : (
                <div class="text-center py-12 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
                  <svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                  <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">No API activity yet</h3>
                  <p class="text-gray-500 dark:text-gray-400 mb-4">Make some requests to see endpoint metrics</p>
                  <button
                    type="button"
                    onClick$={() => { activeTab.value = 'test'; }}
                    class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700"
                  >
                    Test API Endpoints
                  </button>
                </div>
              )}
            </div>
          </div>
        )}

        {activeTab.value === 'config' && (
          <div class="space-y-6">
            <ConfigViewer config={mockConfig} title="Loaded Configuration" />
          </div>
        )}

        {activeTab.value === 'logs' && (
          <div class="space-y-6">
            <RequestLogViewer />
          </div>
        )}

        {activeTab.value === 'test' && (
          <div class="space-y-6">
            <ApiTester />
          </div>
        )}
      </div>
    </DashboardLayout>
  );
});

export const head: DocumentHead = {
  title: "Backworks Dashboard",
  meta: [
    {
      name: "description",
      content: "Real-time monitoring dashboard for Backworks API platform",
    },
  ],
};
