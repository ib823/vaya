//! Node management

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use time::OffsetDateTime;

use crate::{FleetError, FleetResult};

/// Unique node identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

impl NodeId {
    /// Create from string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate new unique ID
    pub fn generate() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let timestamp = OffsetDateTime::now_utc().unix_timestamp_nanos() as u64;
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        Self(format!("node-{:x}-{:04x}", timestamp, counter))
    }

    /// Get as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Node status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    /// Node is starting up
    Starting,
    /// Node is healthy and ready
    Ready,
    /// Node is busy
    Busy,
    /// Node is draining (not accepting new work)
    Draining,
    /// Node is unhealthy
    Unhealthy,
    /// Node is offline
    Offline,
}

impl NodeStatus {
    /// Check if node can accept work
    pub fn can_accept_work(&self) -> bool {
        matches!(self, NodeStatus::Ready)
    }

    /// Check if node is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(
            self,
            NodeStatus::Ready | NodeStatus::Busy | NodeStatus::Draining
        )
    }
}

/// Node information
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// Node ID
    pub id: NodeId,
    /// Node address
    pub address: String,
    /// Node port
    pub port: u16,
    /// Node status
    pub status: NodeStatus,
    /// CPU cores
    pub cpu_cores: u32,
    /// Memory in MB
    pub memory_mb: u64,
    /// Current load (0-100)
    pub load_percent: u8,
    /// Last heartbeat timestamp
    pub last_heartbeat: i64,
    /// Node version
    pub version: String,
    /// Custom labels
    pub labels: HashMap<String, String>,
}

impl NodeInfo {
    /// Create new node info
    pub fn new(id: NodeId, address: impl Into<String>, port: u16) -> Self {
        Self {
            id,
            address: address.into(),
            port,
            status: NodeStatus::Starting,
            cpu_cores: num_cpus(),
            memory_mb: available_memory_mb(),
            load_percent: 0,
            last_heartbeat: OffsetDateTime::now_utc().unix_timestamp(),
            version: env!("CARGO_PKG_VERSION").into(),
            labels: HashMap::new(),
        }
    }

    /// Get full address
    pub fn full_address(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }

    /// Update heartbeat
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = OffsetDateTime::now_utc().unix_timestamp();
    }

    /// Check if heartbeat is stale
    pub fn is_stale(&self, timeout_secs: i64) -> bool {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        now - self.last_heartbeat > timeout_secs
    }

    /// Add label
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }
}

/// A single node in the cluster
#[derive(Debug)]
pub struct Node {
    /// Node information
    pub info: NodeInfo,
    /// Assigned tasks
    pub tasks: Vec<String>,
}

impl Node {
    /// Create new node
    pub fn new(info: NodeInfo) -> Self {
        Self {
            info,
            tasks: Vec::new(),
        }
    }

    /// Check if node can accept more tasks
    pub fn can_accept_task(&self) -> bool {
        self.info.status.can_accept_work() && self.info.load_percent < 90
    }

    /// Assign task
    pub fn assign_task(&mut self, task_id: String) {
        self.tasks.push(task_id);
        self.update_load();
    }

    /// Remove task
    pub fn remove_task(&mut self, task_id: &str) {
        self.tasks.retain(|t| t != task_id);
        self.update_load();
    }

    fn update_load(&mut self) {
        // Simple load calculation based on task count
        self.info.load_percent = ((self.tasks.len() * 10) as u8).min(100);
    }
}

/// Pool of nodes
#[derive(Debug)]
pub struct NodePool {
    /// All nodes
    nodes: HashMap<NodeId, Node>,
    /// Maximum nodes
    max_nodes: usize,
}

impl NodePool {
    /// Create new pool
    pub fn new(max_nodes: usize) -> Self {
        Self {
            nodes: HashMap::new(),
            max_nodes,
        }
    }

    /// Add node
    pub fn add(&mut self, node: Node) -> FleetResult<()> {
        if self.nodes.len() >= self.max_nodes {
            return Err(FleetError::ClusterFull {
                max_nodes: self.max_nodes,
            });
        }
        self.nodes.insert(node.info.id.clone(), node);
        Ok(())
    }

    /// Remove node
    pub fn remove(&mut self, id: &NodeId) -> Option<Node> {
        self.nodes.remove(id)
    }

    /// Get node
    pub fn get(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Get mutable node
    pub fn get_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    /// Get all ready nodes
    pub fn ready_nodes(&self) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|n| n.info.status == NodeStatus::Ready)
            .collect()
    }

    /// Get node with least load
    pub fn least_loaded(&self) -> Option<&Node> {
        self.nodes
            .values()
            .filter(|n| n.can_accept_task())
            .min_by_key(|n| n.info.load_percent)
    }

    /// Count total nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Update stale nodes
    pub fn mark_stale_nodes(&mut self, timeout_secs: i64) {
        for node in self.nodes.values_mut() {
            if node.info.is_stale(timeout_secs) {
                node.info.status = NodeStatus::Unhealthy;
            }
        }
    }
}

/// Get CPU count
fn num_cpus() -> u32 {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/cpuinfo")
            .map(|s| s.lines().filter(|l| l.starts_with("processor")).count() as u32)
            .unwrap_or(4)
    }
    #[cfg(not(target_os = "linux"))]
    {
        4
    }
}

/// Get available memory in MB
fn available_memory_mb() -> u64 {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("MemTotal:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .map(|kb| kb / 1024)
            .unwrap_or(4096)
    }
    #[cfg(not(target_os = "linux"))]
    {
        4096
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_generate() {
        let id1 = NodeId::generate();
        let id2 = NodeId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_node_status() {
        assert!(NodeStatus::Ready.can_accept_work());
        assert!(!NodeStatus::Draining.can_accept_work());
        assert!(NodeStatus::Busy.is_healthy());
    }

    #[test]
    fn test_node_info() {
        let id = NodeId::new("test");
        let info = NodeInfo::new(id, "localhost", 8080);
        assert_eq!(info.full_address(), "localhost:8080");
    }

    #[test]
    fn test_node_pool() {
        let mut pool = NodePool::new(10);
        let id = NodeId::new("node-1");
        let info = NodeInfo::new(id.clone(), "localhost", 8080);
        let node = Node::new(info);

        pool.add(node).unwrap();
        assert_eq!(pool.len(), 1);
        assert!(pool.get(&id).is_some());
    }

    #[test]
    fn test_node_pool_full() {
        let mut pool = NodePool::new(1);

        let info1 = NodeInfo::new(NodeId::new("1"), "localhost", 8080);
        pool.add(Node::new(info1)).unwrap();

        let info2 = NodeInfo::new(NodeId::new("2"), "localhost", 8081);
        let result = pool.add(Node::new(info2));
        assert!(matches!(result, Err(FleetError::ClusterFull { .. })));
    }
}
