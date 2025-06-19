import { component$, useSignal, useTask$ } from '@builder.io/qwik';

export interface TableColumn {
  key: string;
  label: string;
  type?: 'text' | 'number' | 'date' | 'email' | 'status' | 'actions';
  sortable?: boolean;
  width?: string;
  render?: (value: any, row: any) => string;
}

export interface TableAction {
  label: string;
  type: 'edit' | 'delete' | 'view' | 'custom';
  endpoint?: string;
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE';
  icon?: string;
  variant?: 'primary' | 'secondary' | 'danger';
  confirmation?: string;
}

export interface WorkflowTableConfig {
  title: string;
  description?: string;
  endpoint: string;
  columns: TableColumn[];
  actions?: TableAction[];
  pagination?: {
    enabled: boolean;
    pageSize: number;
  };
  search?: {
    enabled: boolean;
    placeholder?: string;
    fields?: string[];
  };
  refreshInterval?: number; // seconds
}

interface WorkflowTableProps {
  config: WorkflowTableConfig;
  onRowAction?: (action: TableAction, row: any) => void;
  onError?: (error: any) => void;
}

export const WorkflowTable = component$<WorkflowTableProps>(({ 
  config, 
  onRowAction, 
  onError 
}) => {
  const data = useSignal<any[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string>('');
  const searchTerm = useSignal('');
  const currentPage = useSignal(1);
  const sortColumn = useSignal<string>('');
  const sortDirection = useSignal<'asc' | 'desc'>('asc');

  const fetchData = async () => {
    try {
      loading.value = true;
      error.value = '';
      
      const response = await fetch(config.endpoint);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
      const result = await response.json();
      data.value = Array.isArray(result) ? result : result.data || [];
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch data';
      error.value = errorMessage;
      if (onError) {
        onError(err);
      }
    } finally {
      loading.value = false;
    }
  };

  // Initial data fetch and periodic refresh
  useTask$(async () => {
    await fetchData();
    
    if (config.refreshInterval && config.refreshInterval > 0) {
      const interval = setInterval(fetchData, config.refreshInterval * 1000);
      return () => clearInterval(interval);
    }
  });

  const filteredData = () => {
    let filtered = data.value;
    
    // Apply search filter
    if (config.search?.enabled && searchTerm.value) {
      const searchFields = config.search.fields || config.columns.map(col => col.key);
      filtered = filtered.filter(row =>
        searchFields.some(field =>
          String(row[field] || '').toLowerCase().includes(searchTerm.value.toLowerCase())
        )
      );
    }
    
    // Apply sorting
    if (sortColumn.value) {
      filtered = [...filtered].sort((a, b) => {
        const aVal = a[sortColumn.value];
        const bVal = b[sortColumn.value];
        
        if (aVal === bVal) return 0;
        
        const comparison = aVal < bVal ? -1 : 1;
        return sortDirection.value === 'asc' ? comparison : -comparison;
      });
    }
    
    return filtered;
  };

  const paginatedData = () => {
    const filtered = filteredData();
    
    if (!config.pagination?.enabled) {
      return filtered;
    }
    
    const pageSize = config.pagination.pageSize || 10;
    const startIndex = (currentPage.value - 1) * pageSize;
    return filtered.slice(startIndex, startIndex + pageSize);
  };

  const totalPages = () => {
    if (!config.pagination?.enabled) return 1;
    const pageSize = config.pagination.pageSize || 10;
    return Math.ceil(filteredData().length / pageSize);
  };

  const handleSort = (column: TableColumn) => {
    if (!column.sortable) return;
    
    if (sortColumn.value === column.key) {
      sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc';
    } else {
      sortColumn.value = column.key;
      sortDirection.value = 'asc';
    }
  };

  const handleAction = async (action: TableAction, row: any) => {
    if (action.confirmation) {
      if (!confirm(action.confirmation)) return;
    }

    if (onRowAction) {
      onRowAction(action, row);
      return;
    }

    // Default action handling
    if (action.endpoint) {
      try {
        const response = await fetch(action.endpoint.replace(':id', row.id), {
          method: action.method || 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: action.method !== 'GET' ? JSON.stringify(row) : undefined,
        });

        if (!response.ok) {
          throw new Error(`Action failed: ${response.statusText}`);
        }

        // Refresh data after successful action
        await fetchData();
      } catch (err) {
        console.error('Action error:', err);
        if (onError) {
          onError(err);
        }
      }
    }
  };

  const formatCellValue = (column: TableColumn, value: any, row: any) => {
    if (column.render) {
      return column.render(value, row);
    }

    switch (column.type) {
      case 'date':
        return value ? new Date(value).toLocaleDateString() : '';
      case 'email':
        return value ? `mailto:${value}` : '';
      case 'status':
        return value ? String(value).charAt(0).toUpperCase() + String(value).slice(1) : '';
      default:
        return String(value || '');
    }
  };

  const getStatusColor = (value: string) => {
    switch (value.toLowerCase()) {
      case 'active':
      case 'published':
      case 'completed':
        return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400';
      case 'inactive':
      case 'draft':
      case 'pending':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400';
      case 'deleted':
      case 'failed':
      case 'cancelled':
        return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400';
    }
  };

  return (
    <div class="rounded-lg bg-white shadow-sm ring-1 ring-gray-900/5 dark:bg-gray-800 dark:ring-white/10">
      {/* Header */}
      <div class="border-b border-gray-200 p-6 dark:border-gray-700">
        <div class="flex items-center justify-between">
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
              {config.title}
            </h3>
            {config.description && (
              <p class="mt-1 text-sm text-gray-600 dark:text-gray-400">
                {config.description}
              </p>
            )}
          </div>
          
          <div class="flex items-center space-x-3">
            {config.search?.enabled && (
              <div class="relative">
                <input
                  type="text"
                  placeholder={config.search.placeholder || 'Search...'}
                  class="block w-64 rounded-md border-0 py-1.5 pl-10 pr-3 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-blue-600 dark:bg-gray-700 dark:text-white dark:ring-gray-600 sm:text-sm"
                  value={searchTerm.value}
                  onInput$={(e) => {
                    searchTerm.value = (e.target as HTMLInputElement).value;
                    currentPage.value = 1; // Reset to first page on search
                  }}
                />
                <div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
                  <svg class="h-5 w-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                  </svg>
                </div>
              </div>
            )}
            
            <button
              onClick$={fetchData}
              disabled={loading.value}
              class="rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50 disabled:opacity-50 dark:bg-gray-700 dark:text-white dark:ring-gray-600 dark:hover:bg-gray-600"
            >
              <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            </button>
          </div>
        </div>
      </div>

      {/* Error State */}
      {error.value && (
        <div class="p-6">
          <div class="rounded-md bg-red-50 p-4 dark:bg-red-900/20">
            <div class="flex">
              <svg class="h-5 w-5 text-red-400" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
              </svg>
              <div class="ml-3">
                <h3 class="text-sm font-medium text-red-800 dark:text-red-200">Error loading data</h3>
                <p class="mt-1 text-sm text-red-700 dark:text-red-300">{error.value}</p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Loading State */}
      {loading.value && !error.value && (
        <div class="p-6">
          <div class="animate-pulse space-y-4">
            {[...Array(5)].map((_, i) => (
              <div key={i} class="h-4 bg-gray-200 rounded dark:bg-gray-700"></div>
            ))}
          </div>
        </div>
      )}

      {/* Table */}
      {!loading.value && !error.value && (
        <>
          <div class="overflow-x-auto">
            <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead class="bg-gray-50 dark:bg-gray-900">
                <tr>
                  {config.columns.map((column) => (
                    <th
                      key={column.key}
                      class={`px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider dark:text-gray-400 ${
                        column.sortable ? 'cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800' : ''
                      }`}
                      style={column.width ? { width: column.width } : {}}
                      onClick$={() => handleSort(column)}
                    >
                      <div class="flex items-center space-x-1">
                        <span>{column.label}</span>
                        {column.sortable && (
                          <svg
                            class={`h-4 w-4 ${
                              sortColumn.value === column.key
                                ? 'text-gray-900 dark:text-white'
                                : 'text-gray-400'
                            } ${
                              sortColumn.value === column.key && sortDirection.value === 'desc'
                                ? 'rotate-180'
                                : ''
                            }`}
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                          >
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 10l5 5 5-5H7z" />
                          </svg>
                        )}
                      </div>
                    </th>
                  ))}
                  {config.actions && config.actions.length > 0 && (
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider dark:text-gray-400">
                      Actions
                    </th>
                  )}
                </tr>
              </thead>
              <tbody class="bg-white divide-y divide-gray-200 dark:bg-gray-800 dark:divide-gray-700">
                {paginatedData().map((row, rowIndex) => (
                  <tr key={rowIndex} class="hover:bg-gray-50 dark:hover:bg-gray-700">
                    {config.columns.map((column) => (
                      <td key={column.key} class="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                        {column.type === 'status' ? (
                          <span class={`inline-flex rounded-full px-2 py-1 text-xs font-medium ${getStatusColor(row[column.key])}`}>
                            {formatCellValue(column, row[column.key], row)}
                          </span>
                        ) : column.type === 'email' ? (
                          <a href={`mailto:${row[column.key]}`} class="text-blue-600 hover:text-blue-500 dark:text-blue-400">
                            {formatCellValue(column, row[column.key], row)}
                          </a>
                        ) : (
                          formatCellValue(column, row[column.key], row)
                        )}
                      </td>
                    ))}
                    {config.actions && config.actions.length > 0 && (
                      <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        <div class="flex space-x-2">
                          {config.actions.map((action, actionIndex) => (
                            <button
                              key={actionIndex}
                              onClick$={() => handleAction(action, row)}
                              class={`inline-flex items-center px-2 py-1 border border-transparent text-xs font-medium rounded ${
                                action.variant === 'danger'
                                  ? 'text-red-700 bg-red-100 hover:bg-red-200 dark:bg-red-900/20 dark:text-red-400'
                                  : action.variant === 'primary'
                                  ? 'text-blue-700 bg-blue-100 hover:bg-blue-200 dark:bg-blue-900/20 dark:text-blue-400'
                                  : 'text-gray-700 bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:text-gray-300'
                              }`}
                            >
                              {action.label}
                            </button>
                          ))}
                        </div>
                      </td>
                    )}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Pagination */}
          {config.pagination?.enabled && totalPages() > 1 && (
            <div class="bg-white px-4 py-3 flex items-center justify-between border-t border-gray-200 dark:bg-gray-800 dark:border-gray-700">
              <div class="flex-1 flex justify-between sm:hidden">
                <button
                  onClick$={() => currentPage.value = Math.max(1, currentPage.value - 1)}
                  disabled={currentPage.value === 1}
                  class="relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 dark:bg-gray-700 dark:text-gray-300 dark:border-gray-600"
                >
                  Previous
                </button>
                <button
                  onClick$={() => currentPage.value = Math.min(totalPages(), currentPage.value + 1)}
                  disabled={currentPage.value === totalPages()}
                  class="ml-3 relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 dark:bg-gray-700 dark:text-gray-300 dark:border-gray-600"
                >
                  Next
                </button>
              </div>
              <div class="hidden sm:flex-1 sm:flex sm:items-center sm:justify-between">
                <div>
                  <p class="text-sm text-gray-700 dark:text-gray-300">
                    Showing {((currentPage.value - 1) * (config.pagination.pageSize || 10)) + 1} to{' '}
                    {Math.min(currentPage.value * (config.pagination.pageSize || 10), filteredData().length)} of{' '}
                    {filteredData().length} results
                  </p>
                </div>
                <div>
                  <nav class="relative z-0 inline-flex rounded-md shadow-sm -space-x-px">
                    <button
                      onClick$={() => currentPage.value = Math.max(1, currentPage.value - 1)}
                      disabled={currentPage.value === 1}
                      class="relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50 dark:bg-gray-700 dark:border-gray-600 dark:text-gray-300"
                    >
                      <svg class="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd" />
                      </svg>
                    </button>
                    
                    {/* Page numbers */}
                    {[...Array(Math.min(5, totalPages()))].map((_, i) => {
                      const page = i + 1;
                      return (
                        <button
                          key={page}
                          onClick$={() => currentPage.value = page}
                          class={`relative inline-flex items-center px-4 py-2 border text-sm font-medium ${
                            currentPage.value === page
                              ? 'z-10 bg-blue-50 border-blue-500 text-blue-600 dark:bg-blue-900/50 dark:border-blue-400 dark:text-blue-300'
                              : 'bg-white border-gray-300 text-gray-500 hover:bg-gray-50 dark:bg-gray-700 dark:border-gray-600 dark:text-gray-300'
                          }`}
                        >
                          {page}
                        </button>
                      );
                    })}
                    
                    <button
                      onClick$={() => currentPage.value = Math.min(totalPages(), currentPage.value + 1)}
                      disabled={currentPage.value === totalPages()}
                      class="relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50 dark:bg-gray-700 dark:border-gray-600 dark:text-gray-300"
                    >
                      <svg class="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
                      </svg>
                    </button>
                  </nav>
                </div>
              </div>
            </div>
          )}
        </>
      )}

      {/* Empty State */}
      {!loading.value && !error.value && data.value.length === 0 && (
        <div class="p-12 text-center">
          <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <h3 class="mt-4 text-sm font-medium text-gray-900 dark:text-white">No data available</h3>
          <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
            No records found for this table.
          </p>
        </div>
      )}
    </div>
  );
});
