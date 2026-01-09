//! Fleet error types

use std::fmt;

/// Result type for fleet operations
pub type FleetResult<T> = Result<T, FleetError>;

/// Fleet errors
#[derive(Debug, Clone)]
pub enum FleetError {
    /// Node not found
    NodeNotFound(String),
    /// Node unreachable
    NodeUnreachable(String),
    /// Consensus error
    ConsensusError(String),
    /// Not leader
    NotLeader { leader_id: Option<String> },
    /// Election failed
    ElectionFailed(String),
    /// Task failed
    TaskFailed { task_id: String, reason: String },
    /// Task timeout
    TaskTimeout(String),
    /// Service not found
    ServiceNotFound(String),
    /// Configuration error
    ConfigError(String),
    /// Network error
    NetworkError(String),
    /// Cluster full
    ClusterFull { max_nodes: usize },
}

impl fmt::Display for FleetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FleetError::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            FleetError::NodeUnreachable(id) => write!(f, "Node unreachable: {}", id),
            FleetError::ConsensusError(msg) => write!(f, "Consensus error: {}", msg),
            FleetError::NotLeader { leader_id } => match leader_id {
                Some(id) => write!(f, "Not leader, current leader: {}", id),
                None => write!(f, "Not leader, no leader elected"),
            },
            FleetError::ElectionFailed(msg) => write!(f, "Election failed: {}", msg),
            FleetError::TaskFailed { task_id, reason } => {
                write!(f, "Task {} failed: {}", task_id, reason)
            }
            FleetError::TaskTimeout(id) => write!(f, "Task timeout: {}", id),
            FleetError::ServiceNotFound(name) => write!(f, "Service not found: {}", name),
            FleetError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            FleetError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            FleetError::ClusterFull { max_nodes } => {
                write!(f, "Cluster full, max nodes: {}", max_nodes)
            }
        }
    }
}

impl std::error::Error for FleetError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = FleetError::NodeNotFound("node-1".into());
        assert!(err.to_string().contains("node-1"));
    }

    #[test]
    fn test_not_leader_with_leader() {
        let err = FleetError::NotLeader {
            leader_id: Some("leader-1".into()),
        };
        assert!(err.to_string().contains("leader-1"));
    }

    #[test]
    fn test_not_leader_no_leader() {
        let err = FleetError::NotLeader { leader_id: None };
        assert!(err.to_string().contains("no leader"));
    }
}
