import { component$, useSignal, $, useComputed$ } from "@builder.io/qwik";

export interface EndpointNode {
  id: string;
  path: string;
  methods: string[];
  description: string;
  handler: string;
  x: number;
  y: number;
}

export interface BlueprintConfig {
  name: string;
  description: string;
  server: {
    host: string;
    port: number;
  };
  dashboard: {
    enabled: boolean;
    port: number;
  };
  mode: string;
}

export const BlueprintDesigner = component$(() => {
  const blueprintConfig = useSignal<BlueprintConfig>({
    name: "My Service",
    description: "A new service created with the blueprint designer",
    server: { host: "0.0.0.0", port: 3000 },
    dashboard: { enabled: true, port: 3001 },
    mode: "runtime"
  });

  const endpoints = useSignal<EndpointNode[]>([]);
  const selectedEndpoint = useSignal<EndpointNode | null>(null);
  const showYamlPreview = useSignal<boolean>(false);
  const copySuccess = useSignal<boolean>(false);

  const addEndpoint = $((method: string) => {
    const newEndpoint: EndpointNode = {
      id: `endpoint_${Date.now()}`,
      path: `/${method.toLowerCase()}`,
      methods: [method],
      description: `${method} endpoint`,
      handler: `function handler(req, res) {
  return {
    status: 200,
    headers: { "Content-Type": "application/json" },
    body: {
      message: "${method} endpoint response",
      timestamp: new Date().toISOString(),
      data: req.body || null
    }
  };
}`,
      x: Math.random() * 400 + 50,
      y: Math.random() * 300 + 50
    };
    
    endpoints.value = [...endpoints.value, newEndpoint];
  });

  const removeEndpoint = $((id: string) => {
    endpoints.value = endpoints.value.filter(ep => ep.id !== id);
    if (selectedEndpoint.value?.id === id) {
      selectedEndpoint.value = null;
    }
  });

  const updateEndpoint = $((updatedEndpoint: EndpointNode) => {
    endpoints.value = endpoints.value.map(ep => 
      ep.id === updatedEndpoint.id ? updatedEndpoint : ep
    );
    selectedEndpoint.value = updatedEndpoint;
  });  const yamlOutput = useComputed$(() => {
    const yamlEndpoints: Record<string, any> = {};

    endpoints.value.forEach(ep => {
      yamlEndpoints[ep.id] = {
        path: ep.path,
        methods: ep.methods,
        description: ep.description,
        runtime: {
          language: "javascript",
          handler: ep.handler
        }
      };
    });

    return `name: "${blueprintConfig.value.name}"
description: "${blueprintConfig.value.description}"
version: "1.0.0"

server:
  host: "${blueprintConfig.value.server.host}"
  port: ${blueprintConfig.value.server.port}

dashboard:
  enabled: ${blueprintConfig.value.dashboard.enabled}
  port: ${blueprintConfig.value.dashboard.port}

mode: "${blueprintConfig.value.mode}"

runtime:
  language: "javascript"
  handler: "enhanced"
  timeout: 10000

endpoints:
${Object.entries(yamlEndpoints).map(([key, value]) => `  ${key}:
    path: "${value.path}"
    methods: [${value.methods.map((m: string) => `"${m}"`).join(', ')}]
    description: "${value.description}"
    runtime:
      language: "${value.runtime.language}"
      handler: |
        ${value.runtime.handler.split('\n').map((line: string) => `        ${line}`).join('\n')}`).join('\n\n')}`;
  });

  const copyToClipboard = $(() => {
    const yaml = yamlOutput.value;
    navigator.clipboard.writeText(yaml).then(() => {
      copySuccess.value = true;
      setTimeout(() => {
        copySuccess.value = false;
      }, 2000);
    });
  });

  const loadTemplate = $((templateName: string) => {
    if (templateName === 'rest-api') {
      endpoints.value = [
        {
          id: 'get_items',
          path: '/api/items',
          methods: ['GET'],
          description: 'Get all items',
          handler: `function handler(req, res) {
  return {
    status: 200,
    headers: { "Content-Type": "application/json" },
    body: {
      items: [
        { id: 1, name: "Item 1", status: "active" },
        { id: 2, name: "Item 2", status: "inactive" }
      ],
      total: 2,
      timestamp: new Date().toISOString()
    }
  };
}`,
          x: 100,
          y: 100
        },
        {
          id: 'create_item',
          path: '/api/items',
          methods: ['POST'],
          description: 'Create a new item',
          handler: `function handler(req, res) {
  const newItem = {
    id: Date.now(),
    name: req.body.name,
    status: "active",
    created: new Date().toISOString()
  };
  
  return {
    status: 201,
    headers: { "Content-Type": "application/json" },
    body: {
      item: newItem,
      message: "Item created successfully"
    }
  };
}`,
          x: 300,
          y: 100
        }
      ];
    }
  });

  return (
    <div class="h-full flex bg-gray-50 dark:bg-gray-900">
      {/* Left Sidebar - Tools & Config */}
      <div class="w-80 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col shadow-lg">
        {/* Header */}
        <div class="p-6 border-b border-gray-200 dark:border-gray-700 bg-gradient-to-r from-blue-50 to-purple-50 dark:from-blue-900/20 dark:to-purple-900/20">
          <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-2 flex items-center">
            🎨 Blueprint Designer
          </h3>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            Visually design your service schematic
          </p>
        </div>
        
        {/* Service Configuration */}
        <div class="p-4 border-b border-gray-200 dark:border-gray-700">
          <h4 class="text-md font-semibold text-gray-900 dark:text-white mb-4">
            Service Configuration
          </h4>
          
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Service Name
              </label>
              <input
                type="text"
                value={blueprintConfig.value.name}
                onInput$={(e) => {
                  blueprintConfig.value = {
                    ...blueprintConfig.value,
                    name: (e.target as HTMLInputElement).value
                  };
                }}
                class="block w-full rounded-lg border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white text-sm"
                placeholder="My Awesome Service"
              />
            </div>
            
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Description
              </label>
              <textarea
                value={blueprintConfig.value.description}
                onInput$={(e) => {
                  blueprintConfig.value = {
                    ...blueprintConfig.value,
                    description: (e.target as HTMLTextAreaElement).value
                  };
                }}
                rows={2}
                class="block w-full rounded-lg border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white text-sm"
                placeholder="Describe your service..."
              />
            </div>
            
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Port
                </label>
                <input
                  type="number"
                  value={blueprintConfig.value.server.port}
                  onInput$={(e) => {
                    blueprintConfig.value = {
                      ...blueprintConfig.value,
                      server: {
                        ...blueprintConfig.value.server,
                        port: parseInt((e.target as HTMLInputElement).value) || 3000
                      }
                    };
                  }}
                  class="block w-full rounded-lg border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white text-sm"
                />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Mode
                </label>
                <select
                  value={blueprintConfig.value.mode}
                  onInput$={(e) => {
                    blueprintConfig.value = {
                      ...blueprintConfig.value,
                      mode: (e.target as HTMLSelectElement).value
                    };
                  }}
                  class="block w-full rounded-lg border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white text-sm"
                >
                  <option value="runtime">Runtime</option>
                  <option value="proxy">Proxy</option>
                  <option value="capture">Capture</option>
                </select>
              </div>
            </div>
          </div>
        </div>

        {/* Templates */}
        <div class="p-4 border-b border-gray-200 dark:border-gray-700">
          <h4 class="text-md font-semibold text-gray-900 dark:text-white mb-3">
            Quick Start Templates
          </h4>
          <div class="space-y-2">
            <button
              onClick$={() => loadTemplate('rest-api')}
              class="w-full text-left px-3 py-2 text-sm bg-gradient-to-r from-green-50 to-emerald-50 hover:from-green-100 hover:to-emerald-100 text-green-800 dark:from-green-900/20 dark:to-emerald-900/20 dark:hover:from-green-900/30 dark:hover:to-emerald-900/30 dark:text-green-300 rounded-lg border border-green-200 dark:border-green-700 transition-colors"
            >
              📋 REST API Template
              <div class="text-xs text-green-600 dark:text-green-400 mt-1">
                GET, POST endpoints with items
              </div>
            </button>
          </div>
        </div>

        {/* Endpoint Palette */}
        <div class="p-4 border-b border-gray-200 dark:border-gray-700">
          <h4 class="text-md font-semibold text-gray-900 dark:text-white mb-3">
            Add Endpoints
          </h4>
          <div class="grid grid-cols-2 gap-2">
            {[
              { method: 'GET', color: 'green', desc: 'Retrieve' },
              { method: 'POST', color: 'blue', desc: 'Create' },
              { method: 'PUT', color: 'yellow', desc: 'Update' },
              { method: 'DELETE', color: 'red', desc: 'Remove' }
            ].map((item) => (
              <button
                key={item.method}
                onClick$={() => addEndpoint(item.method)}
                class={`px-3 py-3 text-sm font-medium rounded-lg border-2 border-dashed transition-all hover:scale-105 ${
                  item.color === 'green' ? 'border-green-300 text-green-700 hover:bg-green-50 dark:border-green-600 dark:text-green-400 dark:hover:bg-green-900/20' :
                  item.color === 'blue' ? 'border-blue-300 text-blue-700 hover:bg-blue-50 dark:border-blue-600 dark:text-blue-400 dark:hover:bg-blue-900/20' :
                  item.color === 'yellow' ? 'border-yellow-300 text-yellow-700 hover:bg-yellow-50 dark:border-yellow-600 dark:text-yellow-400 dark:hover:bg-yellow-900/20' :
                  'border-red-300 text-red-700 hover:bg-red-50 dark:border-red-600 dark:text-red-400 dark:hover:bg-red-900/20'
                }`}
              >
                <div class="font-bold">+ {item.method}</div>
                <div class="text-xs opacity-75">{item.desc}</div>
              </button>
            ))}
          </div>
        </div>

        {/* Endpoints List */}
        <div class="flex-1 p-4 overflow-y-auto">
          <h4 class="text-md font-semibold text-gray-900 dark:text-white mb-3">
            Endpoints ({endpoints.value.length})
          </h4>
          
          {endpoints.value.length === 0 ? (
            <div class="text-center py-8">
              <div class="text-4xl mb-3">🎯</div>
              <p class="text-sm text-gray-500 dark:text-gray-400 italic">
                No endpoints yet. Add some using the buttons above or try a template!
              </p>
            </div>
          ) : (
            <div class="space-y-3">
              {endpoints.value.map((endpoint) => (
                <div
                  key={endpoint.id}
                  onClick$={() => { selectedEndpoint.value = endpoint; }}
                  class={`p-4 rounded-lg border cursor-pointer transition-all hover:shadow-md ${
                    selectedEndpoint.value?.id === endpoint.id
                      ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 shadow-md'
                      : 'border-gray-200 hover:border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
                  }`}
                >
                  <div class="flex items-center justify-between">
                    <div class="flex-1">
                      <div class="flex items-center space-x-2 mb-1">
                        <span class={`px-2 py-1 text-xs font-bold rounded ${
                          endpoint.methods[0] === 'GET' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300' :
                          endpoint.methods[0] === 'POST' ? 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300' :
                          endpoint.methods[0] === 'PUT' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300' :
                          'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300'
                        }`}>
                          {endpoint.methods[0]}
                        </span>
                        <span class="text-sm font-medium text-gray-900 dark:text-white">
                          {endpoint.path}
                        </span>
                      </div>
                      <p class="text-xs text-gray-500 dark:text-gray-400">
                        {endpoint.description}
                      </p>
                    </div>
                    <button
                      onClick$={(e) => {
                        e.stopPropagation();
                        removeEndpoint(endpoint.id);
                      }}
                      class="text-red-500 hover:text-red-700 hover:bg-red-50 dark:hover:bg-red-900/30 p-1 rounded transition-colors"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Actions */}
        <div class="p-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50">
          <div class="space-y-3">
            <button
              onClick$={() => { showYamlPreview.value = !showYamlPreview.value; }}
              class="w-full px-4 py-3 bg-gradient-to-r from-blue-600 to-blue-700 text-white rounded-lg hover:from-blue-700 hover:to-blue-800 transition-colors font-medium flex items-center justify-center space-x-2"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
              </svg>
              <span>{showYamlPreview.value ? 'Hide' : 'Show'} YAML Preview</span>
            </button>
            
            <button
              onClick$={copyToClipboard}
              class={`w-full px-4 py-3 rounded-lg transition-colors font-medium flex items-center justify-center space-x-2 ${
                copySuccess.value 
                  ? 'bg-green-600 text-white' 
                  : 'bg-gradient-to-r from-green-600 to-emerald-600 text-white hover:from-green-700 hover:to-emerald-700'
              }`}
            >
              {copySuccess.value ? (
                <>
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                  </svg>
                  <span>Copied!</span>
                </>
              ) : (
                <>
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                  <span>Copy YAML to Clipboard</span>
                </>
              )}
            </button>
          </div>
        </div>
      </div>

      {/* Main Content Area */}
      <div class="flex-1 flex flex-col bg-white dark:bg-gray-800">
        {/* Canvas or Editor */}
        {showYamlPreview.value ? (
          /* YAML Preview */
          <div class="flex-1 p-6">
            <div class="h-full bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800 rounded-xl p-6 overflow-auto border border-gray-200 dark:border-gray-700">
              <div class="flex items-center justify-between mb-6">
                <h3 class="text-xl font-bold text-gray-900 dark:text-white flex items-center">
                  <span class="text-2xl mr-2">📄</span>
                  Generated blueprint.yaml
                </h3>
                <div class="flex items-center space-x-2">
                  <div class="px-3 py-1 bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300 rounded-full text-sm font-medium">
                    Ready to Deploy
                  </div>
                </div>
              </div>
              
              <div class="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
                <div class="bg-gray-800 text-gray-200 px-4 py-2 text-sm font-mono border-b border-gray-700">
                  blueprint.yaml
                </div>
                <pre class="text-sm text-gray-800 dark:text-gray-200 font-mono p-4 overflow-x-auto whitespace-pre-wrap">
                  {yamlOutput.value}
                </pre>
              </div>
              
              <div class="mt-6 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-700">
                <h4 class="text-md font-semibold text-blue-900 dark:text-blue-300 mb-2">
                  🚀 Next Steps
                </h4>
                <ul class="text-sm text-blue-800 dark:text-blue-300 space-y-1">
                  <li>• Copy the YAML above to a new file called <code class="bg-blue-100 dark:bg-blue-900 px-1 rounded">blueprint.yaml</code></li>
                  <li>• Run <code class="bg-blue-100 dark:bg-blue-900 px-1 rounded">backworks run blueprint.yaml</code> to start your service</li>
                  <li>• Test your endpoints using the API Tester tab</li>
                </ul>
              </div>
            </div>
          </div>
        ) : selectedEndpoint.value ? (
          /* Endpoint Editor */
          <div class="flex-1 p-6">
            <div class="h-full bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 overflow-auto">
              <div class="flex items-center justify-between mb-6">
                <h3 class="text-xl font-bold text-gray-900 dark:text-white flex items-center">
                  <span class="text-2xl mr-2">⚙️</span>
                  Edit Endpoint: <span class="text-blue-600 dark:text-blue-400 ml-2">{selectedEndpoint.value.path}</span>
                </h3>
                <div class="flex items-center space-x-2">
                  <span class={`px-3 py-1 text-xs font-bold rounded-full ${
                    selectedEndpoint.value.methods[0] === 'GET' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300' :
                    selectedEndpoint.value.methods[0] === 'POST' ? 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300' :
                    selectedEndpoint.value.methods[0] === 'PUT' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300' :
                    'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300'
                  }`}>
                    {selectedEndpoint.value.methods[0]}
                  </span>
                </div>
              </div>
              
              <div class="space-y-6">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div>
                    <label class="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                      🌐 Endpoint Path
                    </label>
                    <input
                      type="text"
                      value={selectedEndpoint.value.path}
                      onInput$={(e) => {
                        const updated = {
                          ...selectedEndpoint.value!,
                          path: (e.target as HTMLInputElement).value
                        };
                        updateEndpoint(updated);
                      }}
                      class="block w-full rounded-lg border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                      placeholder="/api/endpoint"
                    />
                  </div>
                  
                  <div>
                    <label class="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                      📝 Description
                    </label>
                    <input
                      type="text"
                      value={selectedEndpoint.value.description}
                      onInput$={(e) => {
                        const updated = {
                          ...selectedEndpoint.value!,
                          description: (e.target as HTMLInputElement).value
                        };
                        updateEndpoint(updated);
                      }}
                      class="block w-full rounded-lg border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                      placeholder="What does this endpoint do?"
                    />
                  </div>
                </div>
                
                <div>
                  <label class="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                    💻 Handler Code
                  </label>
                  <div class="bg-gray-50 dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
                    <div class="bg-gray-800 text-gray-200 px-4 py-2 text-sm font-mono border-b border-gray-700 flex items-center justify-between">
                      <span>JavaScript Handler Function</span>
                      <span class="text-xs text-gray-400">Auto-save enabled</span>
                    </div>
                    <textarea
                      value={selectedEndpoint.value.handler}
                      onInput$={(e) => {
                        const updated = {
                          ...selectedEndpoint.value!,
                          handler: (e.target as HTMLTextAreaElement).value
                        };
                        updateEndpoint(updated);
                      }}
                      rows={20}
                      class="block w-full border-0 bg-white dark:bg-gray-900 text-gray-800 dark:text-gray-200 font-mono text-sm p-4 focus:ring-0 resize-none"
                      placeholder="function handler(req, res) { ... }"
                    />
                  </div>
                </div>
                
                <div class="bg-gradient-to-r from-yellow-50 to-amber-50 dark:from-yellow-900/20 dark:to-amber-900/20 p-4 rounded-lg border border-yellow-200 dark:border-yellow-700">
                  <h4 class="text-md font-semibold text-yellow-900 dark:text-yellow-300 mb-2">
                    💡 Handler Tips
                  </h4>
                  <ul class="text-sm text-yellow-800 dark:text-yellow-300 space-y-1">
                    <li>• Use <code class="bg-yellow-100 dark:bg-yellow-900 px-1 rounded">req.body</code> to access POST data</li>
                    <li>• Use <code class="bg-yellow-100 dark:bg-yellow-900 px-1 rounded">req.params</code> for URL parameters</li>
                    <li>• Return an object with <code class="bg-yellow-100 dark:bg-yellow-900 px-1 rounded">status</code>, <code class="bg-yellow-100 dark:bg-yellow-900 px-1 rounded">headers</code>, and <code class="bg-yellow-100 dark:bg-yellow-900 px-1 rounded">body</code></li>
                    <li>• Add error handling with try/catch blocks</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>
        ) : (
          /* Welcome/Instructions */
          <div class="flex-1 flex items-center justify-center p-6">
            <div class="text-center max-w-2xl">
              <div class="text-8xl mb-6">🎨</div>
              <h3 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                Blueprint Designer
              </h3>
              <p class="text-lg text-gray-600 dark:text-gray-400 mb-8">
                Design your declarative backend service with our visual service schematic builder
              </p>
              
              <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
                <div class="bg-gradient-to-br from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 p-6 rounded-xl border border-blue-200 dark:border-blue-700">
                  <div class="text-3xl mb-3">🚀</div>
                  <h4 class="font-semibold text-blue-900 dark:text-blue-300 mb-2">Quick Start</h4>
                  <p class="text-sm text-blue-800 dark:text-blue-300">
                    Use our REST API template to get started immediately with common endpoints
                  </p>
                </div>
                
                <div class="bg-gradient-to-br from-green-50 to-emerald-50 dark:from-green-900/20 dark:to-emerald-900/20 p-6 rounded-xl border border-green-200 dark:border-green-700">
                  <div class="text-3xl mb-3">🔧</div>
                  <h4 class="font-semibold text-green-900 dark:text-green-300 mb-2">Custom Build</h4>
                  <p class="text-sm text-green-800 dark:text-green-300">
                    Add individual endpoints using the HTTP method buttons and customize each one
                  </p>
                </div>
              </div>
              
              <div class="space-y-3 text-sm text-gray-500 dark:text-gray-400">
                <div class="flex items-center justify-center space-x-2">
                  <span>1️⃣</span>
                  <span>Configure your service details</span>
                </div>
                <div class="flex items-center justify-center space-x-2">
                  <span>2️⃣</span>
                  <span>Add endpoints using templates or custom methods</span>
                </div>
                <div class="flex items-center justify-center space-x-2">
                  <span>3️⃣</span>
                  <span>Click endpoints to edit their handler code</span>
                </div>
                <div class="flex items-center justify-center space-x-2">
                  <span>4️⃣</span>
                  <span>Generate and copy your blueprint.yaml file</span>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
});
