//! Raft consensus implementation

use std::collections::{HashMap, HashSet};
use time::OffsetDateTime;

use crate::{FleetError, FleetResult, NodeId};

/// Raft node state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftState {
    /// Follower - accepts commands from leader
    Follower,
    /// Candidate - seeking election
    Candidate,
    /// Leader - replicates log to followers
    Leader,
}

/// Raft configuration
#[derive(Debug, Clone)]
pub struct RaftConfig {
    /// Election timeout range (min, max) in milliseconds
    pub election_timeout: (u64, u64),
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval: u64,
    /// Maximum log entries per batch
    pub max_batch_size: usize,
}

impl Default for RaftConfig {
    fn default() -> Self {
        Self {
            election_timeout: (150, 300),
            heartbeat_interval: 50,
            max_batch_size: 100,
        }
    }
}

/// Log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Term when entry was created
    pub term: u64,
    /// Entry index
    pub index: u64,
    /// Command data
    pub command: Vec<u8>,
}

/// Vote request
#[derive(Debug, Clone)]
pub struct VoteRequest {
    /// Candidate's term
    pub term: u64,
    /// Candidate ID
    pub candidate_id: NodeId,
    /// Candidate's last log index
    pub last_log_index: u64,
    /// Candidate's last log term
    pub last_log_term: u64,
}

/// Vote response
#[derive(Debug, Clone)]
pub struct VoteResponse {
    /// Current term
    pub term: u64,
    /// Vote granted
    pub vote_granted: bool,
}

/// Append entries request
#[derive(Debug, Clone)]
pub struct AppendRequest {
    /// Leader's term
    pub term: u64,
    /// Leader ID
    pub leader_id: NodeId,
    /// Previous log index
    pub prev_log_index: u64,
    /// Previous log term
    pub prev_log_term: u64,
    /// Entries to append
    pub entries: Vec<LogEntry>,
    /// Leader's commit index
    pub leader_commit: u64,
}

/// Append entries response
#[derive(Debug, Clone)]
pub struct AppendResponse {
    /// Current term
    pub term: u64,
    /// Success
    pub success: bool,
    /// Match index (for faster log catchup)
    pub match_index: u64,
}

/// Raft node
#[derive(Debug)]
pub struct RaftNode {
    /// Node ID
    id: NodeId,
    /// Current state
    state: RaftState,
    /// Current term
    current_term: u64,
    /// Who we voted for in current term
    voted_for: Option<NodeId>,
    /// Log entries
    log: Vec<LogEntry>,
    /// Commit index
    commit_index: u64,
    /// Last applied index
    last_applied: u64,
    /// Current leader
    leader_id: Option<NodeId>,
    /// Configuration
    config: RaftConfig,
    /// Cluster members
    members: HashSet<NodeId>,
    /// Next index for each follower (leader only)
    next_index: HashMap<NodeId, u64>,
    /// Match index for each follower (leader only)
    match_index: HashMap<NodeId, u64>,
    /// Votes received (candidate only)
    votes_received: HashSet<NodeId>,
    /// Last heartbeat time
    last_heartbeat: i64,
}

impl RaftNode {
    /// Create new Raft node
    pub fn new(id: NodeId, config: RaftConfig) -> Self {
        Self {
            id: id.clone(),
            state: RaftState::Follower,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            leader_id: None,
            config,
            members: HashSet::from([id]),
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            votes_received: HashSet::new(),
            last_heartbeat: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    /// Get node ID
    pub fn id(&self) -> &NodeId {
        &self.id
    }

    /// Get current state
    pub fn state(&self) -> RaftState {
        self.state
    }

    /// Get current term
    pub fn current_term(&self) -> u64 {
        self.current_term
    }

    /// Check if leader
    pub fn is_leader(&self) -> bool {
        self.state == RaftState::Leader
    }

    /// Get leader ID
    pub fn leader_id(&self) -> Option<&NodeId> {
        self.leader_id.as_ref()
    }

    /// Add member to cluster
    pub fn add_member(&mut self, id: NodeId) {
        self.members.insert(id);
    }

    /// Remove member from cluster
    pub fn remove_member(&mut self, id: &NodeId) {
        self.members.remove(id);
        self.next_index.remove(id);
        self.match_index.remove(id);
    }

    /// Start election
    pub fn start_election(&mut self) -> VoteRequest {
        self.state = RaftState::Candidate;
        self.current_term += 1;
        self.voted_for = Some(self.id.clone());
        self.votes_received.clear();
        self.votes_received.insert(self.id.clone());

        let (last_log_index, last_log_term) = self.last_log_info();

        VoteRequest {
            term: self.current_term,
            candidate_id: self.id.clone(),
            last_log_index,
            last_log_term,
        }
    }

    /// Handle vote request
    pub fn handle_vote_request(&mut self, req: VoteRequest) -> VoteResponse {
        // Update term if needed
        if req.term > self.current_term {
            self.become_follower(req.term);
        }

        let vote_granted = req.term >= self.current_term
            && self.voted_for.as_ref().map(|v| v == &req.candidate_id).unwrap_or(true)
            && self.is_log_up_to_date(req.last_log_index, req.last_log_term);

        if vote_granted {
            self.voted_for = Some(req.candidate_id);
            self.last_heartbeat = OffsetDateTime::now_utc().unix_timestamp();
        }

        VoteResponse {
            term: self.current_term,
            vote_granted,
        }
    }

    /// Handle vote response
    pub fn handle_vote_response(&mut self, from: NodeId, resp: VoteResponse) -> bool {
        if resp.term > self.current_term {
            self.become_follower(resp.term);
            return false;
        }

        if self.state != RaftState::Candidate || resp.term != self.current_term {
            return false;
        }

        if resp.vote_granted {
            self.votes_received.insert(from);

            // Check if we have majority
            if self.votes_received.len() > self.members.len() / 2 {
                self.become_leader();
                return true;
            }
        }

        false
    }

    /// Become leader
    fn become_leader(&mut self) {
        self.state = RaftState::Leader;
        self.leader_id = Some(self.id.clone());

        // Initialize leader state
        let last_index = self.log.len() as u64;
        for member in &self.members {
            if member != &self.id {
                self.next_index.insert(member.clone(), last_index + 1);
                self.match_index.insert(member.clone(), 0);
            }
        }

        tracing::info!("Node {} became leader for term {}", self.id.as_str(), self.current_term);
    }

    /// Become follower
    fn become_follower(&mut self, term: u64) {
        self.state = RaftState::Follower;
        self.current_term = term;
        self.voted_for = None;
        self.votes_received.clear();
    }

    /// Handle append entries request
    pub fn handle_append(&mut self, req: AppendRequest) -> AppendResponse {
        if req.term > self.current_term {
            self.become_follower(req.term);
        }

        self.last_heartbeat = OffsetDateTime::now_utc().unix_timestamp();

        if req.term < self.current_term {
            return AppendResponse {
                term: self.current_term,
                success: false,
                match_index: 0,
            };
        }

        self.leader_id = Some(req.leader_id);

        // Check log consistency
        if req.prev_log_index > 0 {
            if req.prev_log_index as usize > self.log.len() {
                return AppendResponse {
                    term: self.current_term,
                    success: false,
                    match_index: self.log.len() as u64,
                };
            }

            if let Some(entry) = self.log.get(req.prev_log_index as usize - 1) {
                if entry.term != req.prev_log_term {
                    self.log.truncate(req.prev_log_index as usize - 1);
                    return AppendResponse {
                        term: self.current_term,
                        success: false,
                        match_index: self.log.len() as u64,
                    };
                }
            }
        }

        // Append new entries
        for entry in req.entries {
            if entry.index as usize > self.log.len() {
                self.log.push(entry);
            }
        }

        // Update commit index
        if req.leader_commit > self.commit_index {
            self.commit_index = req.leader_commit.min(self.log.len() as u64);
        }

        AppendResponse {
            term: self.current_term,
            success: true,
            match_index: self.log.len() as u64,
        }
    }

    /// Append command (leader only)
    pub fn append_command(&mut self, command: Vec<u8>) -> FleetResult<u64> {
        if !self.is_leader() {
            return Err(FleetError::NotLeader {
                leader_id: self.leader_id.as_ref().map(|id| id.as_str().to_string()),
            });
        }

        let index = self.log.len() as u64 + 1;
        let entry = LogEntry {
            term: self.current_term,
            index,
            command,
        };
        self.log.push(entry);

        Ok(index)
    }

    fn last_log_info(&self) -> (u64, u64) {
        self.log
            .last()
            .map(|e| (e.index, e.term))
            .unwrap_or((0, 0))
    }

    fn is_log_up_to_date(&self, index: u64, term: u64) -> bool {
        let (our_index, our_term) = self.last_log_info();
        term > our_term || (term == our_term && index >= our_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raft_initial_state() {
        let node = RaftNode::new(NodeId::new("node-1"), RaftConfig::default());
        assert_eq!(node.state(), RaftState::Follower);
        assert_eq!(node.current_term(), 0);
    }

    #[test]
    fn test_start_election() {
        let mut node = RaftNode::new(NodeId::new("node-1"), RaftConfig::default());
        let req = node.start_election();

        assert_eq!(node.state(), RaftState::Candidate);
        assert_eq!(node.current_term(), 1);
        assert_eq!(req.term, 1);
    }

    #[test]
    fn test_vote_grant() {
        let mut node = RaftNode::new(NodeId::new("node-1"), RaftConfig::default());
        let req = VoteRequest {
            term: 1,
            candidate_id: NodeId::new("node-2"),
            last_log_index: 0,
            last_log_term: 0,
        };

        let resp = node.handle_vote_request(req);
        assert!(resp.vote_granted);
    }

    #[test]
    fn test_become_leader() {
        let mut node = RaftNode::new(NodeId::new("node-1"), RaftConfig::default());
        node.add_member(NodeId::new("node-2"));
        node.add_member(NodeId::new("node-3"));

        node.start_election();

        // Simulate receiving votes
        let resp = VoteResponse {
            term: 1,
            vote_granted: true,
        };

        node.handle_vote_response(NodeId::new("node-2"), resp.clone());
        assert!(node.is_leader());
    }
}
