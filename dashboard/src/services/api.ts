export interface SystemMetrics {
  uptime: string;
  total_requests: number;
  active_connections: number;
  cpu_usage: number;
  memory_usage: number;
  status: string;
}

export interface EndpointMetric {
  method: string;
  path: string;
  request_count: number;
  avg_response_time: number;
  last_accessed: string;
}

export interface PerformanceMetric {
  timestamp: string;
  response_time: number;
  requests_per_second: number;
  error_rate: number;
}

const API_BASE = 'http://localhost:3003/api';

export class BackworksAPI {
  static async getSystemMetrics(): Promise<SystemMetrics> {
    try {
      const response = await fetch(`${API_BASE}/system`);
      if (!response.ok) {
        throw new Error('Failed to fetch system metrics');
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching system metrics:', error);
      // Return mock data if API is unavailable
      return {
        uptime: '0m 0s',
        total_requests: 0,
        active_connections: 0,
        cpu_usage: 0,
        memory_usage: 0,
        status: 'Offline'
      };
    }
  }

  static async getEndpointMetrics(): Promise<EndpointMetric[]> {
    try {
      const response = await fetch(`${API_BASE}/metrics`);
      if (!response.ok) {
        throw new Error('Failed to fetch endpoint metrics');
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching endpoint metrics:', error);
      return [];
    }
  }

  static async getPerformanceData(): Promise<PerformanceMetric[]> {
    try {
      const response = await fetch(`${API_BASE}/performance`);
      if (!response.ok) {
        throw new Error('Failed to fetch performance data');
      }
      const data = await response.json();
      return data.metrics || [];
    } catch (error) {
      console.error('Error fetching performance data:', error);
      return [];
    }
  }

  static formatMemory(bytes: number): string {
    const sizes = ['B', 'KB', 'MB', 'GB'];
    if (bytes === 0) return '0 B';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
  }

  static formatUptime(uptime: string): string {
    return uptime;
  }
}
