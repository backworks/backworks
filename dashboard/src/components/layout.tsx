import { component$, Slot, useSignal, $ } from '@builder.io/qwik';

export const DashboardLayout = component$(() => {
  const isDarkMode = useSignal(false);
  const isSidebarCollapsed = useSignal(false);

  const toggleTheme = $(() => {
    isDarkMode.value = !isDarkMode.value;
    document.documentElement.classList.toggle('dark', isDarkMode.value);
  });

  const toggleSidebar = $(() => {
    isSidebarCollapsed.value = !isSidebarCollapsed.value;
  });

  return (
    <div class={`min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800 ${isDarkMode.value ? 'dark' : ''}`}>
      {/* Sidebar */}
      <div class={`fixed inset-y-0 left-0 z-50 bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl shadow-2xl border-r border-gray-200/50 dark:border-gray-700/50 transition-all duration-300 ${isSidebarCollapsed.value ? 'w-16' : 'w-72'}`}>
        {/* Header */}
        <div class="flex h-16 items-center justify-between px-4 border-b border-gray-200/50 dark:border-gray-700/50">
          {!isSidebarCollapsed.value && (
            <div class="flex items-center space-x-3">
              <div class="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
                <span class="text-white font-bold text-sm">B</span>
              </div>
              <div>
                <h1 class="text-lg font-bold bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
                  Backworks Studio
                </h1>
                <p class="text-xs text-gray-500 dark:text-gray-400">Declarative Backend Platform</p>
              </div>
            </div>
          )}
          
          <button
            onClick$={toggleSidebar}
            class="p-2 rounded-lg text-gray-500 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800 transition-colors"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>
        </div>
        
        {/* Navigation */}
        <nav class="mt-6 px-3">
          <div class="space-y-1">
            {/* Overview */}
            <a href="/" class="group flex items-center rounded-xl px-3 py-2.5 text-sm font-medium bg-gradient-to-r from-blue-50 to-purple-50 text-blue-700 dark:from-blue-900/30 dark:to-purple-900/30 dark:text-blue-300 shadow-sm">
              <svg class="mr-3 h-5 w-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2-2z" />
              </svg>
              {!isSidebarCollapsed.value && <span>Overview</span>}
            </a>
            
            {/* Blueprint Designer */}
            <a href="/designer" class="group flex items-center rounded-xl px-3 py-2.5 text-sm font-medium text-gray-700 hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 hover:text-gray-900 dark:text-gray-300 dark:hover:from-gray-800/50 dark:hover:to-gray-700/50 dark:hover:text-white transition-all duration-200">
              <svg class="mr-3 h-5 w-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zM21 5a2 2 0 00-2-2h-4a2 2 0 00-2 2v12a4 4 0 004 4h4a2 2 0 002-2V5z" />
              </svg>
              {!isSidebarCollapsed.value && <span>Blueprint Designer</span>}
            </a>
            
            {/* Service Schematics */}
            <a href="/schematics" class="group flex items-center rounded-xl px-3 py-2.5 text-sm font-medium text-gray-700 hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 hover:text-gray-900 dark:text-gray-300 dark:hover:from-gray-800/50 dark:hover:to-gray-700/50 dark:hover:text-white transition-all duration-200">
              <svg class="mr-3 h-5 w-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              {!isSidebarCollapsed.value && <span>Service Schematics</span>}
            </a>
            
            {/* Metrics & Analytics */}
            <a href="/metrics" class="group flex items-center rounded-xl px-3 py-2.5 text-sm font-medium text-gray-700 hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 hover:text-gray-900 dark:text-gray-300 dark:hover:from-gray-800/50 dark:hover:to-gray-700/50 dark:hover:text-white transition-all duration-200">
              <svg class="mr-3 h-5 w-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
              </svg>
              {!isSidebarCollapsed.value && <span>Metrics & Analytics</span>}
            </a>
            
            {/* API Explorer */}
            <a href="/explorer" class="group flex items-center rounded-xl px-3 py-2.5 text-sm font-medium text-gray-700 hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 hover:text-gray-900 dark:text-gray-300 dark:hover:from-gray-800/50 dark:hover:to-gray-700/50 dark:hover:text-white transition-all duration-200">
              <svg class="mr-3 h-5 w-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
              {!isSidebarCollapsed.value && <span>API Explorer</span>}
            </a>
            
            {/* Runtime Logs */}
            <a href="/logs" class="group flex items-center rounded-xl px-3 py-2.5 text-sm font-medium text-gray-700 hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 hover:text-gray-900 dark:text-gray-300 dark:hover:from-gray-800/50 dark:hover:to-gray-700/50 dark:hover:text-white transition-all duration-200">
              <svg class="mr-3 h-5 w-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              {!isSidebarCollapsed.value && <span>Runtime Logs</span>}
            </a>
          </div>
        </nav>
        
        {/* Divider */}
        {!isSidebarCollapsed.value && (
          <div class="mt-8 px-3">
            <div class="border-t border-gray-200 dark:border-gray-700"></div>
          </div>
        )}
        
        {/* Footer */}
        <div class="absolute bottom-0 left-0 right-0 p-4 border-t border-gray-200/50 dark:border-gray-700/50">
          {!isSidebarCollapsed.value ? (
            <div class="flex items-center justify-between text-xs">
              <div class="text-gray-500 dark:text-gray-400">
                <div class="font-medium">Backworks v2.0.0</div>
                <div class="text-gray-400 dark:text-gray-500">Platform Status</div>
              </div>
              <div class="flex items-center space-x-2">
                <div class="h-2 w-2 bg-green-400 rounded-full animate-pulse"></div>
                <span class="text-green-600 dark:text-green-400 font-medium">Live</span>
              </div>
            </div>
          ) : (
            <div class="flex justify-center">
              <div class="h-2 w-2 bg-green-400 rounded-full animate-pulse"></div>
            </div>
          )}
        </div>
      </div>
      
      {/* Main content */}
      <div class={`transition-all duration-300 ${isSidebarCollapsed.value ? 'ml-16' : 'ml-72'}`}>
        {/* Top header */}
        <header class="sticky top-0 z-40 flex h-16 items-center justify-between border-b border-gray-200/50 bg-white/80 backdrop-blur-xl px-6 dark:border-gray-700/50 dark:bg-gray-900/80">
          <div class="flex items-center space-x-4">
            <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
              Dashboard
            </h2>
            <div class="hidden sm:flex items-center space-x-2 px-3 py-1 bg-blue-50 dark:bg-blue-900/30 rounded-full">
              <div class="h-2 w-2 bg-blue-500 rounded-full"></div>
              <span class="text-xs font-medium text-blue-700 dark:text-blue-300">Service Active</span>
            </div>
          </div>
          
          <div class="flex items-center space-x-3">
            {/* Quick Actions */}
            <button class="p-2 rounded-lg text-gray-500 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-800 dark:hover:text-white transition-colors">
              <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
              </svg>
            </button>
            
            {/* Theme toggle */}
            <button 
              onClick$={toggleTheme}
              class="p-2 rounded-lg text-gray-500 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-800 dark:hover:text-white transition-colors"
            >
              {isDarkMode.value ? (
                <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
                </svg>
              ) : (
                <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
                </svg>
              )}
            </button>
            
            {/* Settings */}
            <button class="p-2 rounded-lg text-gray-500 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-800 dark:hover:text-white transition-colors">
              <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            </button>
          </div>
        </header>
        
        {/* Page content */}
        <main class="p-6">
          <Slot />
        </main>
      </div>
    </div>
  );
});
