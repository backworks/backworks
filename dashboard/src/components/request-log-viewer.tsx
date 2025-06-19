import { component$, useSignal, useTask$ } from '@builder.io/qwik';

interface RequestLog {
  id: string;
  timestamp: string;
  method: string;
  path: string;
  status: number;
  responseTime: number;
  userAgent?: string;
}

interface RequestLogViewerProps {
  maxEntries?: number;
}

export const RequestLogViewer = component$<RequestLogViewerProps>(({ maxEntries = 50 }) => {
  const logs = useSignal<RequestLog[]>([]);
  const isLive = useSignal(true);

  // Simulate live request data (replace with actual WebSocket or polling)
  useTask$(async () => {
    const addMockRequest = () => {
      const methods = ['GET', 'POST', 'PUT', 'DELETE'];
      const paths = ['/hello', '/echo', '/users', '/api/health', '/metrics'];
      const statuses = [200, 201, 400, 404, 500];
      
      const newLog: RequestLog = {
        id: Date.now().toString(),
        timestamp: new Date().toISOString(),
        method: methods[Math.floor(Math.random() * methods.length)],
        path: paths[Math.floor(Math.random() * paths.length)],
        status: statuses[Math.floor(Math.random() * statuses.length)],
        responseTime: Math.floor(Math.random() * 500) + 10,
        userAgent: 'Mozilla/5.0 (curl/7.68.0)'
      };

      if (isLive.value) {
        logs.value = [newLog, ...logs.value.slice(0, maxEntries - 1)];
      }
    };

    // Add initial logs
    for (let i = 0; i < 5; i++) {
      setTimeout(addMockRequest, i * 1000);
    }

    // Continue adding logs every 3-8 seconds
    const interval = setInterval(() => {
      if (Math.random() > 0.3) { // 70% chance to add a log
        addMockRequest();
      }
    }, 3000 + Math.random() * 5000);

    return () => clearInterval(interval);
  });

  const getStatusColor = (status: number) => {
    if (status >= 200 && status < 300) return 'text-green-600 dark:text-green-400';
    if (status >= 300 && status < 400) return 'text-yellow-600 dark:text-yellow-400';
    if (status >= 400 && status < 500) return 'text-orange-600 dark:text-orange-400';
    return 'text-red-600 dark:text-red-400';
  };

  const getMethodColor = (method: string) => {
    switch (method) {
      case 'GET': return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
      case 'POST': return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
      case 'PUT': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200';
      case 'DELETE': return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
      default: return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200';
    }
  };

  const formatTime = (timestamp: string) => {
    return new Date(timestamp).toLocaleTimeString();
  };

  return (
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white flex items-center">
          <svg class="w-5 h-5 mr-2 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4M7.835 4.697a3.42 3.42 0 001.946-.806 3.42 3.42 0 014.438 0 3.42 3.42 0 001.946.806 3.42 3.42 0 013.138 3.138 3.42 3.42 0 00.806 1.946 3.42 3.42 0 010 4.438 3.42 3.42 0 00-.806 1.946 3.42 3.42 0 01-3.138 3.138 3.42 3.42 0 00-1.946.806 3.42 3.42 0 01-4.438 0 3.42 3.42 0 00-1.946-.806 3.42 3.42 0 01-3.138-3.138 3.42 3.42 0 00-.806-1.946 3.42 3.42 0 010-4.438 3.42 3.42 0 00.806-1.946 3.42 3.42 0 013.138-3.138z" />
          </svg>
          Live Request Log
        </h3>
        <div class="flex items-center space-x-2">
          <div class={`flex items-center ${isLive.value ? 'text-green-600 dark:text-green-400' : 'text-gray-500 dark:text-gray-400'}`}>
            <div class={`w-2 h-2 rounded-full mr-2 ${isLive.value ? 'bg-green-500 animate-pulse' : 'bg-gray-400'}`}></div>
            <span class="text-sm font-medium">
              {isLive.value ? 'Live' : 'Paused'}
            </span>
          </div>
          <button
            type="button"
            onClick$={() => { isLive.value = !isLive.value; }}
            class={`inline-flex items-center px-3 py-1 border border-gray-300 dark:border-gray-600 shadow-sm text-sm leading-4 font-medium rounded-md ${
              isLive.value 
                ? 'text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600' 
                : 'text-green-700 dark:text-green-300 bg-green-50 dark:bg-green-900 hover:bg-green-100 dark:hover:bg-green-800'
            } focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500`}
          >
            {isLive.value ? (
              <>
                <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                Pause
              </>
            ) : (
              <>
                <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1m4 0h1m-6 4h.01M15 14h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                Resume
              </>
            )}
          </button>
          <button
            type="button"
            onClick$={() => { logs.value = []; }}
            class="inline-flex items-center px-3 py-1 border border-gray-300 dark:border-gray-600 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
            Clear
          </button>
        </div>
      </div>

      <div class="max-h-96 overflow-y-auto">
        {logs.value.length === 0 ? (
          <div class="text-center py-8 text-gray-500 dark:text-gray-400">
            <svg class="mx-auto h-12 w-12 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01" />
            </svg>
            <p>No requests yet</p>
            <p class="text-sm mt-1">API requests will appear here in real-time</p>
          </div>
        ) : (
          <div class="divide-y divide-gray-200 dark:divide-gray-700">
            {logs.value.map((log) => (
              <div key={log.id} class="p-4 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
                <div class="flex items-center justify-between">
                  <div class="flex items-center space-x-3">
                    <span class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getMethodColor(log.method)}`}>
                      {log.method}
                    </span>
                    <span class="text-sm font-mono text-gray-900 dark:text-white">
                      {log.path}
                    </span>
                    <span class={`text-sm font-semibold ${getStatusColor(log.status)}`}>
                      {log.status}
                    </span>
                  </div>
                  <div class="flex items-center space-x-2 text-sm text-gray-500 dark:text-gray-400">
                    <span>{log.responseTime}ms</span>
                    <span>{formatTime(log.timestamp)}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
});
