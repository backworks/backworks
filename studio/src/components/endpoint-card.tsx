import { component$ } from '@builder.io/qwik';

interface EndpointCardProps {
  method: string;
  path: string;
  requestCount: number;
  avgResponseTime: number;
  lastAccessed: string;
  errorRate?: number;
}

export const EndpointCard = component$<EndpointCardProps>(({ 
  method, 
  path, 
  requestCount, 
  avgResponseTime, 
  lastAccessed, 
  errorRate = 0 
}) => {
  const getMethodColor = (method: string) => {
    switch (method.toUpperCase()) {
      case 'GET':
        return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400';
      case 'POST':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400';
      case 'PUT':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400';
      case 'DELETE':
        return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400';
    }
  };

  const getResponseTimeColor = (time: number) => {
    if (time < 100) return 'text-green-600 dark:text-green-400';
    if (time < 500) return 'text-yellow-600 dark:text-yellow-400';
    return 'text-red-600 dark:text-red-400';
  };

  return (
    <div class="rounded-lg bg-white p-4 shadow-sm ring-1 ring-gray-900/5 dark:bg-gray-800 dark:ring-white/10">
      <div class="flex items-center justify-between mb-3">
        <div class="flex items-center space-x-3">
          <span class={`inline-flex rounded-full px-2 py-1 text-xs font-medium ${getMethodColor(method)}`}>
            {method}
          </span>
          <span class="font-mono text-sm font-medium text-gray-900 dark:text-white">
            {path}
          </span>
        </div>
        {errorRate > 0 && (
          <div class="flex items-center text-red-500">
            <svg class="h-4 w-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
            </svg>
            <span class="text-xs">{(errorRate * 100).toFixed(1)}%</span>
          </div>
        )}
      </div>
      
      <div class="grid grid-cols-3 gap-4 text-sm">
        <div>
          <p class="text-gray-500 dark:text-gray-400">Requests</p>
          <p class="font-semibold text-gray-900 dark:text-white">{requestCount.toLocaleString()}</p>
        </div>
        <div>
          <p class="text-gray-500 dark:text-gray-400">Avg Time</p>
          <p class={`font-semibold ${getResponseTimeColor(avgResponseTime)}`}>
            {avgResponseTime.toFixed(1)}ms
          </p>
        </div>
        <div>
          <p class="text-gray-500 dark:text-gray-400">Last Access</p>
          <p class="font-semibold text-gray-900 dark:text-white">{lastAccessed}</p>
        </div>
      </div>
    </div>
  );
});
