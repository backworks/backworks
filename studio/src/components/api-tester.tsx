import { component$, useSignal, $ } from '@builder.io/qwik';

interface ApiTestResult {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
  responseTime: number;
  timestamp: string;
}

export const ApiTester = component$(() => {
  const method = useSignal('GET');
  const url = useSignal('/hello');
  const headers = useSignal('{}');
  const body = useSignal('');

  const result = useSignal<ApiTestResult | null>(null);
  const isLoading = useSignal(false);
  const error = useSignal<string>('');

  const makeRequest = $(async () => {
    isLoading.value = true;
    error.value = '';
    result.value = null;

    try {
      const startTime = Date.now();
      
      // Parse headers
      let parsedHeaders: Record<string, string> = {};
      try {
        parsedHeaders = JSON.parse(headers.value);
      } catch {
        parsedHeaders = {};
      }

      // Build request options
      const requestOptions: RequestInit = {
        method: method.value,
        headers: {
          'Content-Type': 'application/json',
          ...parsedHeaders
        }
      };

      if (method.value !== 'GET' && body.value) {
        requestOptions.body = body.value;
      }

      let response: Response;
      let responseText: string;
      
      // Try to make request to actual backend first (port 3000 - the default Backworks port)
      const apiUrl = `http://localhost:3000${url.value}`;
      
      try {
        response = await fetch(apiUrl, requestOptions);
        responseText = await response.text();
      } catch (networkError) {
        // If backend is not available, provide mock responses for demonstration
        console.log('Backend not available, using mock response', networkError);
        
        const mockResponse = {
          status: 200,
          statusText: 'OK (Mock)',
          headers: {
            'content-type': 'application/json',
            'x-mock-response': 'true',
            'x-powered-by': 'Backworks Studio Demo'
          },
          body: JSON.stringify({
            message: `${method.value} request to ${url.value}`,
            timestamp: new Date().toISOString(),
            data: method.value === 'POST' || method.value === 'PUT' ? 
              (body.value ? JSON.parse(body.value || '{}') : null) : null,
            mock: true,
            note: "This is a mock response. Start your Backworks service to see real API responses."
          }, null, 2)
        };
        
        // Simulate network delay
        await new Promise(resolve => setTimeout(resolve, 100 + Math.random() * 200));
        
        const endTime = Date.now();
        const responseTime = endTime - startTime;

        result.value = {
          status: mockResponse.status,
          statusText: mockResponse.statusText,
          headers: mockResponse.headers,
          body: mockResponse.body,
          responseTime,
          timestamp: new Date().toISOString()
        };
        
        return;
      }
      
      const endTime = Date.now();
      const responseTime = endTime - startTime;
      
      // Get response headers
      const responseHeaders: Record<string, string> = {};
      response.headers.forEach((value, key) => {
        responseHeaders[key] = value;
      });

      result.value = {
        status: response.status,
        statusText: response.statusText,
        headers: responseHeaders,
        body: responseText,
        responseTime,
        timestamp: new Date().toISOString()
      };

    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Unknown error occurred';
    } finally {
      isLoading.value = false;
    }
  });

  const getStatusColor = (status: number) => {
    if (status >= 200 && status < 300) return 'text-green-600 dark:text-green-400 bg-green-50 dark:bg-green-900';
    if (status >= 300 && status < 400) return 'text-yellow-600 dark:text-yellow-400 bg-yellow-50 dark:bg-yellow-900';
    if (status >= 400 && status < 500) return 'text-orange-600 dark:text-orange-400 bg-orange-50 dark:bg-orange-900';
    return 'text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900';
  };

  const formatJson = (str: string) => {
    try {
      return JSON.stringify(JSON.parse(str), null, 2);
    } catch {
      return str;
    }
  };

  const loadPreset = $((preset: { method: string; url: string; body?: string }) => {
    method.value = preset.method;
    url.value = preset.url;
    body.value = preset.body || '';
  });

  return (
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
      <div class="p-4 border-b border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-white flex items-center">
            <svg class="w-5 h-5 mr-2 text-purple-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.367 2.684 3 3 0 00-5.367-2.684z" />
            </svg>
            API Tester
          </h3>
          <div class="flex items-center space-x-2">
            <div class="px-2 py-1 bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300 rounded-full text-xs font-medium">
              Demo Mode
            </div>
          </div>
        </div>
        
        {/* Quick Test Presets */}
        <div class="mt-4">
          <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">Quick Test Presets:</p>
          <div class="flex flex-wrap gap-2">
            <button
              onClick$={() => loadPreset({ method: 'GET', url: '/api/hello' })}
              class="px-3 py-1 text-xs bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300 rounded-full hover:bg-green-200 dark:hover:bg-green-900/50 transition-colors"
            >
              GET /api/hello
            </button>
            <button
              onClick$={() => loadPreset({ 
                method: 'POST', 
                url: '/api/users', 
                body: JSON.stringify({ name: "John Doe", email: "john@example.com" }, null, 2)
              })}
              class="px-3 py-1 text-xs bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300 rounded-full hover:bg-blue-200 dark:hover:bg-blue-900/50 transition-colors"
            >
              POST /api/users
            </button>
            <button
              onClick$={() => loadPreset({ method: 'GET', url: '/api/status' })}
              class="px-3 py-1 text-xs bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300 rounded-full hover:bg-purple-200 dark:hover:bg-purple-900/50 transition-colors"
            >
              GET /api/status
            </button>
          </div>
        </div>
      </div>

      <div class="p-4 space-y-4">
        {/* Request Form */}
        <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div class="md:col-span-1">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Method
            </label>
            <select
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              value={method.value}
              onInput$={(_, el) => { method.value = el.value; }}
            >
              <option value="GET">GET</option>
              <option value="POST">POST</option>
              <option value="PUT">PUT</option>
              <option value="DELETE">DELETE</option>
              <option value="PATCH">PATCH</option>
            </select>
          </div>
          
          <div class="md:col-span-3">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              URL Path
            </label>
            <div class="flex">
              <span class="inline-flex items-center px-3 rounded-l-md border border-r-0 border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-600 text-gray-500 dark:text-gray-400 text-sm">
                localhost:3002
              </span>
              <input
                type="text"
                class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-r-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="/hello"
                value={url.value}
                onInput$={(_, el) => { url.value = el.value; }}
              />
            </div>
          </div>
        </div>

        {/* Headers */}
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Headers (JSON)
          </label>
          <textarea
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono text-sm"
            rows={3}
            placeholder='{"Authorization": "Bearer token", "Custom-Header": "value"}'
            value={headers.value}
            onInput$={(_, el) => { headers.value = el.value; }}
          />
        </div>

        {/* Body (only for non-GET requests) */}
        {method.value !== 'GET' && (
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Request Body (JSON)
            </label>
            <textarea
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono text-sm"
              rows={4}
              placeholder='{"key": "value"}'
              value={body.value}
              onInput$={(_, el) => { body.value = el.value; }}
            />
          </div>
        )}

        {/* Send Button */}
        <div class="flex justify-end">          
          <button
            type="button"
            onClick$={makeRequest}
            disabled={isLoading.value}
            class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isLoading.value ? (
              <>
                <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Sending...
              </>
            ) : (
              <>
                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
                </svg>
                Send Request
              </>
            )}
          </button>
        </div>

        {/* Error Display */}
        {error.value && (
          <div class="rounded-md bg-red-50 dark:bg-red-900 p-4">
            <div class="flex">
              <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
              </svg>
              <div class="ml-3">
                <h3 class="text-sm font-medium text-red-800 dark:text-red-200">
                  Request Failed
                </h3>
                <div class="mt-2 text-sm text-red-700 dark:text-red-300">
                  {error.value}
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Result Display */}
        {result.value && (
          <div class="border border-gray-200 dark:border-gray-600 rounded-lg">
            {/* Mock Response Notice */}
            {result.value.headers['x-mock-response'] && (
              <div class="bg-yellow-50 dark:bg-yellow-900/20 px-4 py-3 border-b border-yellow-200 dark:border-yellow-700">
                <div class="flex items-center">
                  <svg class="w-4 h-4 text-yellow-600 dark:text-yellow-400 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  <span class="text-sm text-yellow-800 dark:text-yellow-300">
                    This is a mock response for demonstration. Start your Backworks service to test real endpoints.
                  </span>
                </div>
              </div>
            )}
            
            <div class="bg-gray-50 dark:bg-gray-700 px-4 py-3 border-b border-gray-200 dark:border-gray-600">
              <div class="flex items-center justify-between">
                <div class="flex items-center space-x-3">
                  <span class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(result.value.status)}`}>
                    {result.value.status} {result.value.statusText}
                  </span>
                  <span class="text-sm text-gray-600 dark:text-gray-400">
                    {result.value.responseTime}ms
                  </span>
                </div>
                <span class="text-sm text-gray-500 dark:text-gray-400">
                  {new Date(result.value.timestamp).toLocaleTimeString()}
                </span>
              </div>
            </div>
            
            <div class="p-4">
              <div class="space-y-4">
                {/* Response Headers */}
                <div>
                  <h4 class="text-sm font-medium text-gray-900 dark:text-white mb-2">Response Headers</h4>
                  <pre class="text-xs bg-gray-50 dark:bg-gray-900 p-3 rounded border overflow-x-auto text-gray-800 dark:text-gray-200">
                    {JSON.stringify(result.value.headers, null, 2)}
                  </pre>
                </div>

                {/* Response Body */}
                <div>
                  <h4 class="text-sm font-medium text-gray-900 dark:text-white mb-2">Response Body</h4>
                  <pre class="text-sm bg-gray-50 dark:bg-gray-900 p-3 rounded border overflow-x-auto text-gray-800 dark:text-gray-200 max-h-64 overflow-y-auto">
                    {formatJson(result.value.body)}
                  </pre>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
});
