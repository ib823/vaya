//! vaya-fleet: VAYA Orchestration System
//!
//! Custom orchestration for VAYA with:
//! - Node management and health monitoring
//! - Task scheduling and distribution
//! - Raft consensus for leader election
//! - Service discovery and routing
//!
//! NO KUBERNETES. NO DOCKER. ALL CUSTOM.

mod consensus;
mod error;
mod node;
mod scheduler;
mod service;

pub use consensus::{RaftConfig, RaftNode, RaftState};
pub use error::{FleetError, FleetResult};
pub use node::{Node, NodeId, NodeInfo, NodePool, NodeStatus};
pub use scheduler::{Scheduler, Task, TaskId, TaskResult, TaskStatus};
pub use service::{Service, ServiceConfig, ServiceDiscovery, ServiceRegistry};

/// Fleet version
pub const FLEET_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default configuration values
pub mod defaults {
    /// Default heartbeat interval in milliseconds
    pub const HEARTBEAT_INTERVAL_MS: u64 = 1000;
    /// Default election timeout in milliseconds
    pub const ELECTION_TIMEOUT_MS: u64 = 5000;
    /// Default health check interval
    pub const HEALTH_CHECK_INTERVAL_MS: u64 = 5000;
    /// Maximum nodes in cluster
    pub const MAX_NODES: usize = 100;
}

/// Fleet configuration
#[derive(Debug, Clone)]
pub struct FleetConfig {
    /// Cluster name
    pub cluster_name: String,
    /// Node ID for this instance
    pub node_id: NodeId,
    /// Bind address
    pub bind_addr: String,
    /// Seed nodes for discovery
    pub seed_nodes: Vec<String>,
    /// Heartbeat interval
    pub heartbeat_ms: u64,
    /// Election timeout
    pub election_timeout_ms: u64,
}

impl Default for FleetConfig {
    fn default() -> Self {
        Self {
            cluster_name: "vaya".into(),
            node_id: NodeId::generate(),
            bind_addr: "0.0.0.0:7000".into(),
            seed_nodes: Vec::new(),
            heartbeat_ms: defaults::HEARTBEAT_INTERVAL_MS,
            election_timeout_ms: defaults::ELECTION_TIMEOUT_MS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fleet_version() {
        assert!(!FLEET_VERSION.is_empty());
    }

    #[test]
    fn test_default_config() {
        let config = FleetConfig::default();
        assert_eq!(config.cluster_name, "vaya");
        assert_eq!(config.heartbeat_ms, defaults::HEARTBEAT_INTERVAL_MS);
    }
}
