//! Service discovery and registry

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use time::OffsetDateTime;

use crate::{FleetError, FleetResult, NodeId};

/// Service health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceHealth {
    /// Service is healthy
    Healthy,
    /// Service is degraded
    Degraded,
    /// Service is unhealthy
    Unhealthy,
    /// Health unknown
    Unknown,
}

/// Service endpoint
#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    /// Node ID
    pub node_id: NodeId,
    /// Address
    pub address: String,
    /// Port
    pub port: u16,
    /// Weight for load balancing
    pub weight: u32,
    /// Health status
    pub health: ServiceHealth,
    /// Last health check timestamp
    pub last_check: i64,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl ServiceEndpoint {
    /// Create new endpoint
    pub fn new(node_id: NodeId, address: impl Into<String>, port: u16) -> Self {
        Self {
            node_id,
            address: address.into(),
            port,
            weight: 100,
            health: ServiceHealth::Unknown,
            last_check: 0,
            metadata: HashMap::new(),
        }
    }

    /// Get full address
    pub fn full_address(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }

    /// Set weight
    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Update health
    pub fn update_health(&mut self, health: ServiceHealth) {
        self.health = health;
        self.last_check = OffsetDateTime::now_utc().unix_timestamp();
    }

    /// Check if endpoint is usable
    pub fn is_usable(&self) -> bool {
        matches!(self.health, ServiceHealth::Healthy | ServiceHealth::Unknown)
    }
}

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Health check interval in milliseconds
    pub health_check_interval_ms: u64,
    /// Health check timeout in milliseconds
    pub health_check_timeout_ms: u64,
    /// Unhealthy threshold (consecutive failures)
    pub unhealthy_threshold: u32,
    /// Healthy threshold (consecutive successes)
    pub healthy_threshold: u32,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            health_check_interval_ms: 10000,
            health_check_timeout_ms: 5000,
            unhealthy_threshold: 3,
            healthy_threshold: 2,
        }
    }
}

/// A service in the registry
#[derive(Debug, Clone)]
pub struct Service {
    /// Service name
    pub name: String,
    /// Service version
    pub version: String,
    /// Endpoints
    pub endpoints: Vec<ServiceEndpoint>,
    /// Configuration
    pub config: ServiceConfig,
    /// Tags for discovery
    pub tags: Vec<String>,
    /// Created timestamp
    pub created_at: i64,
}

impl Service {
    /// Create new service
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            endpoints: Vec::new(),
            config: ServiceConfig::default(),
            tags: Vec::new(),
            created_at: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: ServiceConfig) -> Self {
        self.config = config;
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add endpoint
    pub fn add_endpoint(&mut self, endpoint: ServiceEndpoint) {
        self.endpoints.push(endpoint);
    }

    /// Remove endpoint
    pub fn remove_endpoint(&mut self, node_id: &NodeId) {
        self.endpoints.retain(|e| &e.node_id != node_id);
    }

    /// Get healthy endpoints
    pub fn healthy_endpoints(&self) -> Vec<&ServiceEndpoint> {
        self.endpoints.iter().filter(|e| e.is_usable()).collect()
    }

    /// Get next endpoint (weighted round-robin)
    pub fn next_endpoint(&self) -> Option<&ServiceEndpoint> {
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let healthy: Vec<_> = self.healthy_endpoints();
        if healthy.is_empty() {
            return None;
        }

        // Simple weighted selection
        let total_weight: u32 = healthy.iter().map(|e| e.weight).sum();
        if total_weight == 0 {
            return healthy.first().copied();
        }

        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        let selection = (counter as u32) % total_weight;

        let mut cumulative = 0;
        for endpoint in &healthy {
            cumulative += endpoint.weight;
            if selection < cumulative {
                return Some(endpoint);
            }
        }

        healthy.last().copied()
    }

    /// Check if service has healthy endpoints
    pub fn is_available(&self) -> bool {
        self.endpoints.iter().any(|e| e.is_usable())
    }
}

/// Service registry
#[derive(Debug)]
pub struct ServiceRegistry {
    /// Registered services
    services: HashMap<String, Service>,
}

impl ServiceRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Register service
    pub fn register(&mut self, service: Service) {
        tracing::info!("Registering service: {} v{}", service.name, service.version);
        self.services.insert(service.name.clone(), service);
    }

    /// Deregister service
    pub fn deregister(&mut self, name: &str) -> Option<Service> {
        tracing::info!("Deregistering service: {}", name);
        self.services.remove(name)
    }

    /// Get service
    pub fn get(&self, name: &str) -> Option<&Service> {
        self.services.get(name)
    }

    /// Get mutable service
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Service> {
        self.services.get_mut(name)
    }

    /// List all services
    pub fn list(&self) -> Vec<&Service> {
        self.services.values().collect()
    }

    /// Find services by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<&Service> {
        self.services
            .values()
            .filter(|s| s.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Add endpoint to service
    pub fn add_endpoint(&mut self, service_name: &str, endpoint: ServiceEndpoint) -> FleetResult<()> {
        let service = self.services.get_mut(service_name).ok_or_else(|| {
            FleetError::ServiceNotFound(service_name.to_string())
        })?;
        service.add_endpoint(endpoint);
        Ok(())
    }

    /// Remove endpoint from service
    pub fn remove_endpoint(&mut self, service_name: &str, node_id: &NodeId) -> FleetResult<()> {
        let service = self.services.get_mut(service_name).ok_or_else(|| {
            FleetError::ServiceNotFound(service_name.to_string())
        })?;
        service.remove_endpoint(node_id);
        Ok(())
    }

    /// Update endpoint health
    pub fn update_health(
        &mut self,
        service_name: &str,
        node_id: &NodeId,
        health: ServiceHealth,
    ) -> FleetResult<()> {
        let service = self.services.get_mut(service_name).ok_or_else(|| {
            FleetError::ServiceNotFound(service_name.to_string())
        })?;

        for endpoint in &mut service.endpoints {
            if &endpoint.node_id == node_id {
                endpoint.update_health(health);
                return Ok(());
            }
        }

        Err(FleetError::NodeNotFound(node_id.as_str().to_string()))
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Service discovery
#[derive(Debug)]
pub struct ServiceDiscovery {
    /// Registry reference
    registry: ServiceRegistry,
    /// Local node ID
    local_node: NodeId,
    /// Discovery configuration
    config: DiscoveryConfig,
}

/// Discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Refresh interval in milliseconds
    pub refresh_interval_ms: u64,
    /// Cache TTL in milliseconds
    pub cache_ttl_ms: u64,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            refresh_interval_ms: 30000,
            cache_ttl_ms: 60000,
        }
    }
}

impl ServiceDiscovery {
    /// Create new service discovery
    pub fn new(local_node: NodeId, config: DiscoveryConfig) -> Self {
        Self {
            registry: ServiceRegistry::new(),
            local_node,
            config,
        }
    }

    /// Get registry
    pub fn registry(&self) -> &ServiceRegistry {
        &self.registry
    }

    /// Get mutable registry
    pub fn registry_mut(&mut self) -> &mut ServiceRegistry {
        &mut self.registry
    }

    /// Discover service endpoint
    pub fn discover(&self, service_name: &str) -> FleetResult<&ServiceEndpoint> {
        let service = self.registry.get(service_name).ok_or_else(|| {
            FleetError::ServiceNotFound(service_name.to_string())
        })?;

        service.next_endpoint().ok_or_else(|| {
            FleetError::ServiceNotFound(format!("{} (no healthy endpoints)", service_name))
        })
    }

    /// Discover all endpoints for service
    pub fn discover_all(&self, service_name: &str) -> FleetResult<Vec<&ServiceEndpoint>> {
        let service = self.registry.get(service_name).ok_or_else(|| {
            FleetError::ServiceNotFound(service_name.to_string())
        })?;

        Ok(service.healthy_endpoints())
    }

    /// Register local service
    pub fn register_local(&mut self, service_name: &str, port: u16) -> FleetResult<()> {
        let endpoint = ServiceEndpoint::new(
            self.local_node.clone(),
            "127.0.0.1",
            port,
        );

        if let Some(service) = self.registry.get_mut(service_name) {
            service.add_endpoint(endpoint);
        } else {
            let mut service = Service::new(service_name, "1.0.0");
            service.add_endpoint(endpoint);
            self.registry.register(service);
        }

        Ok(())
    }

    /// Deregister local service
    pub fn deregister_local(&mut self, service_name: &str) -> FleetResult<()> {
        self.registry.remove_endpoint(service_name, &self.local_node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_endpoint() {
        let endpoint = ServiceEndpoint::new(NodeId::new("node-1"), "localhost", 8080);
        assert_eq!(endpoint.full_address(), "localhost:8080");
        assert!(endpoint.is_usable());
    }

    #[test]
    fn test_service() {
        let mut service = Service::new("api", "1.0.0");
        service.add_endpoint(ServiceEndpoint::new(NodeId::new("node-1"), "localhost", 8080));

        assert!(service.is_available());
        assert_eq!(service.healthy_endpoints().len(), 1);
    }

    #[test]
    fn test_service_registry() {
        let mut registry = ServiceRegistry::new();
        let service = Service::new("api", "1.0.0").with_tag("http");

        registry.register(service);

        assert!(registry.get("api").is_some());
        assert_eq!(registry.find_by_tag("http").len(), 1);
    }

    #[test]
    fn test_service_discovery() {
        let mut discovery = ServiceDiscovery::new(
            NodeId::new("local"),
            DiscoveryConfig::default(),
        );

        discovery.register_local("api", 8080).unwrap();

        let endpoint = discovery.discover("api").unwrap();
        assert_eq!(endpoint.port, 8080);
    }

    #[test]
    fn test_health_update() {
        let mut registry = ServiceRegistry::new();
        let mut service = Service::new("api", "1.0.0");
        let node_id = NodeId::new("node-1");
        service.add_endpoint(ServiceEndpoint::new(node_id.clone(), "localhost", 8080));
        registry.register(service);

        registry.update_health("api", &node_id, ServiceHealth::Healthy).unwrap();

        let service = registry.get("api").unwrap();
        assert_eq!(service.endpoints[0].health, ServiceHealth::Healthy);
    }
}
