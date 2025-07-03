//! Load balancing algorithms for the proxy plugin

use crate::error::{ProxyError, ProxyResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use sha2::{Digest, Sha256};

/// Load balancing algorithm types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    /// Round robin - cycles through targets in order
    RoundRobin,
    
    /// Weighted round robin - considers target weights
    Weighted,
    
    /// IP hash - consistent routing based on client IP
    IpHash,
    
    /// Least connections - route to target with fewest active connections
    LeastConnections,
    
    /// Random - randomly select a target
    Random,
}

impl Default for LoadBalancingAlgorithm {
    fn default() -> Self {
        LoadBalancingAlgorithm::RoundRobin
    }
}

/// Proxy target configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyTarget {
    /// Target identifier
    pub name: String,
    
    /// Target URL
    pub url: String,
    
    /// Target weight (for weighted algorithms)
    pub weight: f64,
    
    /// Whether target is healthy
    pub healthy: bool,
    
    /// Current active connections
    pub active_connections: u32,
    
    /// Request timeout
    pub timeout: Option<std::time::Duration>,
}

impl ProxyTarget {
    pub fn new(name: String, url: String) -> Self {
        Self {
            name,
            url,
            weight: 1.0,
            healthy: true,
            active_connections: 0,
            timeout: None,
        }
    }
}

/// Load balancer implementation
#[derive(Debug, Clone)]
pub struct LoadBalancer {
    algorithm: LoadBalancingAlgorithm,
    targets: Arc<RwLock<Vec<ProxyTarget>>>,
    current_index: Arc<RwLock<usize>>,
}

impl LoadBalancer {
    pub fn new(algorithm: LoadBalancingAlgorithm) -> Self {
        Self {
            algorithm,
            targets: Arc::new(RwLock::new(Vec::new())),
            current_index: Arc::new(RwLock::new(0)),
        }
    }

    /// Add a target to the load balancer
    pub async fn add_target(&self, target: ProxyTarget) -> ProxyResult<()> {
        let mut targets = self.targets.write().await;
        targets.push(target);
        Ok(())
    }

    /// Remove a target from the load balancer
    pub async fn remove_target(&self, name: &str) -> ProxyResult<()> {
        let mut targets = self.targets.write().await;
        targets.retain(|t| t.name != name);
        Ok(())
    }

    /// Update target health status
    pub async fn update_target_health(&self, name: &str, healthy: bool) -> ProxyResult<()> {
        let mut targets = self.targets.write().await;
        if let Some(target) = targets.iter_mut().find(|t| t.name == name) {
            target.healthy = healthy;
        }
        Ok(())
    }

    /// Get the next target based on the load balancing algorithm
    pub async fn get_next_target(&self, client_ip: Option<&str>) -> ProxyResult<ProxyTarget> {
        let targets = self.targets.read().await;
        let healthy_targets: Vec<&ProxyTarget> = targets.iter().filter(|t| t.healthy).collect();
        
        if healthy_targets.is_empty() {
            return Err(ProxyError::TargetUnavailable("No healthy targets available".to_string()));
        }
        
        let selected_target = match self.algorithm {
            LoadBalancingAlgorithm::RoundRobin => {
                self.round_robin_select(&healthy_targets).await?
            }
            
            LoadBalancingAlgorithm::Weighted => {
                self.weighted_select(&healthy_targets).await?
            }
            
            LoadBalancingAlgorithm::IpHash => {
                self.ip_hash_select(&healthy_targets, client_ip).await?
            }
            
            LoadBalancingAlgorithm::LeastConnections => {
                self.least_connections_select(&healthy_targets).await?
            }
            
            LoadBalancingAlgorithm::Random => {
                self.random_select(&healthy_targets).await?
            }
        };
        
        Ok(selected_target.clone())
    }

    /// Round robin selection
    async fn round_robin_select<'a>(&self, targets: &'a [&'a ProxyTarget]) -> ProxyResult<&'a ProxyTarget> {
        let mut index = self.current_index.write().await;
        let target = targets[*index % targets.len()];
        *index = (*index + 1) % targets.len();
        Ok(target)
    }

    /// Weighted round robin selection
    async fn weighted_select<'a>(&self, targets: &'a [&'a ProxyTarget]) -> ProxyResult<&'a ProxyTarget> {
        let total_weight: f64 = targets.iter().map(|t| t.weight).sum();
        let mut random_weight = rand::random::<f64>() * total_weight;
        
        for &target in targets {
            random_weight -= target.weight;
            if random_weight <= 0.0 {
                return Ok(target);
            }
        }
        
        // Fallback to first target
        Ok(targets[0])
    }

    /// IP hash based selection
    async fn ip_hash_select<'a>(&self, targets: &'a [&'a ProxyTarget], client_ip: Option<&str>) -> ProxyResult<&'a ProxyTarget> {
        let ip = client_ip.unwrap_or("127.0.0.1");
        let mut hasher = Sha256::new();
        hasher.update(ip.as_bytes());
        let hash = hasher.finalize();
        
        // Convert first 8 bytes to u64
        let hash_value = u64::from_le_bytes([
            hash[0], hash[1], hash[2], hash[3],
            hash[4], hash[5], hash[6], hash[7]
        ]);
        
        let index = (hash_value as usize) % targets.len();
        Ok(targets[index])
    }

    /// Least connections selection
    async fn least_connections_select<'a>(&self, targets: &'a [&'a ProxyTarget]) -> ProxyResult<&'a ProxyTarget> {
        let min_connections = targets.iter().map(|t| t.active_connections).min().unwrap_or(0);
        
        // Find the first target with minimum connections
        for &target in targets {
            if target.active_connections == min_connections {
                return Ok(target);
            }
        }
        
        // Fallback to first target if none found (shouldn't happen)
        Ok(targets[0])
    }

    /// Random selection
    async fn random_select<'a>(&self, targets: &'a [&'a ProxyTarget]) -> ProxyResult<&'a ProxyTarget> {
        let index = rand::random::<usize>() % targets.len();
        Ok(targets[index])
    }

    /// Get all targets
    pub async fn get_targets(&self) -> Vec<ProxyTarget> {
        let targets = self.targets.read().await;
        targets.clone()
    }

    /// Get healthy targets count
    pub async fn healthy_targets_count(&self) -> usize {
        let targets = self.targets.read().await;
        targets.iter().filter(|t| t.healthy).count()
    }

    /// Increment active connections for a target
    pub async fn increment_connections(&self, target_name: &str) -> ProxyResult<()> {
        let mut targets = self.targets.write().await;
        if let Some(target) = targets.iter_mut().find(|t| t.name == target_name) {
            target.active_connections += 1;
        }
        Ok(())
    }

    /// Decrement active connections for a target
    pub async fn decrement_connections(&self, target_name: &str) -> ProxyResult<()> {
        let mut targets = self.targets.write().await;
        if let Some(target) = targets.iter_mut().find(|t| t.name == target_name) {
            target.active_connections = target.active_connections.saturating_sub(1);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_round_robin_load_balancing() {
        let lb = LoadBalancer::new(LoadBalancingAlgorithm::RoundRobin);
        
        // Add targets
        lb.add_target(ProxyTarget::new("target1".to_string(), "http://localhost:8001".to_string())).await.unwrap();
        lb.add_target(ProxyTarget::new("target2".to_string(), "http://localhost:8002".to_string())).await.unwrap();
        
        // Test round robin
        let target1 = lb.get_next_target(None).await.unwrap();
        let target2 = lb.get_next_target(None).await.unwrap();
        let target3 = lb.get_next_target(None).await.unwrap();
        
        assert_eq!(target1.name, "target1");
        assert_eq!(target2.name, "target2");
        assert_eq!(target3.name, "target1"); // Should cycle back
    }

    #[tokio::test]
    async fn test_weighted_load_balancing() {
        let lb = LoadBalancer::new(LoadBalancingAlgorithm::Weighted);
        
        let mut target1 = ProxyTarget::new("target1".to_string(), "http://localhost:8001".to_string());
        target1.weight = 1.0;
        
        let mut target2 = ProxyTarget::new("target2".to_string(), "http://localhost:8002".to_string());
        target2.weight = 3.0;
        
        lb.add_target(target1).await.unwrap();
        lb.add_target(target2).await.unwrap();
        
        // Test weighted selection (target2 should be selected more often)
        let mut target2_count = 0;
        for _ in 0..100 {
            let target = lb.get_next_target(None).await.unwrap();
            if target.name == "target2" {
                target2_count += 1;
            }
        }
        
        // target2 should be selected ~75% of the time (3/4)
        assert!(target2_count > 60); // Allow some variance
    }

    #[tokio::test]
    async fn test_ip_hash_load_balancing() {
        let lb = LoadBalancer::new(LoadBalancingAlgorithm::IpHash);
        
        lb.add_target(ProxyTarget::new("target1".to_string(), "http://localhost:8001".to_string())).await.unwrap();
        lb.add_target(ProxyTarget::new("target2".to_string(), "http://localhost:8002".to_string())).await.unwrap();
        
        // Same IP should always get same target
        let target1 = lb.get_next_target(Some("192.168.1.100")).await.unwrap();
        let target2 = lb.get_next_target(Some("192.168.1.100")).await.unwrap();
        
        assert_eq!(target1.name, target2.name);
    }

    #[tokio::test]
    async fn test_unhealthy_target_exclusion() {
        let lb = LoadBalancer::new(LoadBalancingAlgorithm::RoundRobin);
        
        lb.add_target(ProxyTarget::new("target1".to_string(), "http://localhost:8001".to_string())).await.unwrap();
        lb.add_target(ProxyTarget::new("target2".to_string(), "http://localhost:8002".to_string())).await.unwrap();
        
        // Mark target1 as unhealthy
        lb.update_target_health("target1", false).await.unwrap();
        
        // Should only get target2
        for _ in 0..10 {
            let target = lb.get_next_target(None).await.unwrap();
            assert_eq!(target.name, "target2");
        }
    }
}
