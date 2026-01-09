//! Task scheduling and distribution

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use time::OffsetDateTime;

use crate::{FleetError, FleetResult, NodeId, NodePool};

/// Unique task identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(pub String);

impl TaskId {
    /// Create from string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate new unique ID
    pub fn generate() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let timestamp = OffsetDateTime::now_utc().unix_timestamp_nanos() as u64;
        let counter = COUNTER.fetch_add(1, AtomicOrdering::SeqCst);
        Self(format!("task-{:x}-{:04x}", timestamp, counter))
    }

    /// Get as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TaskPriority {
    /// Low priority
    Low = 0,
    /// Normal priority
    #[default]
    Normal = 1,
    /// High priority
    High = 2,
    /// Critical priority
    Critical = 3,
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is pending
    Pending,
    /// Task is queued
    Queued,
    /// Task is running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
    /// Task timed out
    TimedOut,
}

impl TaskStatus {
    /// Check if task is terminal
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskStatus::Completed
                | TaskStatus::Failed
                | TaskStatus::Cancelled
                | TaskStatus::TimedOut
        )
    }

    /// Check if task is active
    pub fn is_active(&self) -> bool {
        matches!(self, TaskStatus::Queued | TaskStatus::Running)
    }
}

/// Task result
#[derive(Debug, Clone)]
pub struct TaskResult {
    /// Task ID
    pub task_id: TaskId,
    /// Final status
    pub status: TaskStatus,
    /// Result data
    pub data: Option<Vec<u8>>,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// A task to execute
#[derive(Debug, Clone)]
pub struct Task {
    /// Task ID
    pub id: TaskId,
    /// Task type/name
    pub task_type: String,
    /// Task payload
    pub payload: Vec<u8>,
    /// Priority
    pub priority: TaskPriority,
    /// Status
    pub status: TaskStatus,
    /// Assigned node
    pub assigned_node: Option<NodeId>,
    /// Created timestamp
    pub created_at: i64,
    /// Started timestamp
    pub started_at: Option<i64>,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Retry count
    pub retries: u32,
    /// Max retries
    pub max_retries: u32,
    /// Dependencies (tasks that must complete first)
    pub dependencies: Vec<TaskId>,
}

impl Task {
    /// Create new task
    pub fn new(task_type: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            id: TaskId::generate(),
            task_type: task_type.into(),
            payload,
            priority: TaskPriority::Normal,
            status: TaskStatus::Pending,
            assigned_node: None,
            created_at: OffsetDateTime::now_utc().unix_timestamp(),
            started_at: None,
            timeout_ms: 30000,
            retries: 0,
            max_retries: 3,
            dependencies: Vec::new(),
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set max retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Add dependency
    pub fn with_dependency(mut self, task_id: TaskId) -> Self {
        self.dependencies.push(task_id);
        self
    }

    /// Check if task can run (dependencies satisfied)
    pub fn can_run(&self, completed_tasks: &HashMap<TaskId, TaskResult>) -> bool {
        self.dependencies.iter().all(|dep| {
            completed_tasks
                .get(dep)
                .map(|r| r.status == TaskStatus::Completed)
                .unwrap_or(false)
        })
    }

    /// Check if timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(started) = self.started_at {
            let now = OffsetDateTime::now_utc().unix_timestamp_nanos() as u64 / 1_000_000;
            let started_ms = started as u64 * 1000;
            now - started_ms > self.timeout_ms
        } else {
            false
        }
    }
}

/// Wrapper for priority queue ordering
#[derive(Debug)]
struct PriorityTask {
    priority: TaskPriority,
    created_at: i64,
    task_id: TaskId,
}

impl PartialEq for PriorityTask {
    fn eq(&self, other: &Self) -> bool {
        self.task_id == other.task_id
    }
}

impl Eq for PriorityTask {}

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then older tasks first
        match (self.priority as u8).cmp(&(other.priority as u8)) {
            Ordering::Equal => other.created_at.cmp(&self.created_at),
            ord => ord,
        }
    }
}

/// Scheduler configuration
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Maximum concurrent tasks per node
    pub max_tasks_per_node: usize,
    /// Default task timeout
    pub default_timeout_ms: u64,
    /// Enable work stealing
    pub work_stealing: bool,
    /// Queue capacity
    pub queue_capacity: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_tasks_per_node: 10,
            default_timeout_ms: 30000,
            work_stealing: true,
            queue_capacity: 10000,
        }
    }
}

/// Task scheduler
#[derive(Debug)]
pub struct Scheduler {
    /// Configuration
    config: SchedulerConfig,
    /// All tasks
    tasks: HashMap<TaskId, Task>,
    /// Priority queue
    priority_queue: BinaryHeap<PriorityTask>,
    /// Ready queue (for round-robin when priorities equal)
    ready_queue: VecDeque<TaskId>,
    /// Completed task results
    completed: HashMap<TaskId, TaskResult>,
    /// Tasks by node
    node_tasks: HashMap<NodeId, Vec<TaskId>>,
}

impl Scheduler {
    /// Create new scheduler
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            config,
            tasks: HashMap::new(),
            priority_queue: BinaryHeap::new(),
            ready_queue: VecDeque::new(),
            completed: HashMap::new(),
            node_tasks: HashMap::new(),
        }
    }

    /// Submit task
    pub fn submit(&mut self, task: Task) -> TaskId {
        let id = task.id.clone();
        let priority_task = PriorityTask {
            priority: task.priority,
            created_at: task.created_at,
            task_id: id.clone(),
        };

        self.tasks.insert(id.clone(), task);
        self.priority_queue.push(priority_task);

        tracing::debug!("Task {} submitted", id.as_str());
        id
    }

    /// Get next task to execute
    pub fn next_task(&mut self) -> Option<&Task> {
        // First check ready queue
        while let Some(task_id) = self.ready_queue.pop_front() {
            if let Some(task) = self.tasks.get(&task_id) {
                if task.status == TaskStatus::Pending && task.can_run(&self.completed) {
                    return Some(task);
                }
            }
        }

        // Then check priority queue
        while let Some(pt) = self.priority_queue.pop() {
            if let Some(task) = self.tasks.get(&pt.task_id) {
                if task.status == TaskStatus::Pending && task.can_run(&self.completed) {
                    return Some(task);
                }
            }
        }

        None
    }

    /// Assign task to node
    pub fn assign(&mut self, task_id: &TaskId, node_id: NodeId) -> FleetResult<()> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| FleetError::TaskFailed {
                task_id: task_id.as_str().to_string(),
                reason: "Task not found".into(),
            })?;

        if task.status != TaskStatus::Pending && task.status != TaskStatus::Queued {
            return Err(FleetError::TaskFailed {
                task_id: task_id.as_str().to_string(),
                reason: format!("Task in invalid state: {:?}", task.status),
            });
        }

        task.status = TaskStatus::Running;
        task.assigned_node = Some(node_id.clone());
        task.started_at = Some(OffsetDateTime::now_utc().unix_timestamp());

        self.node_tasks
            .entry(node_id)
            .or_default()
            .push(task_id.clone());

        tracing::info!("Task {} assigned to node", task_id.as_str());
        Ok(())
    }

    /// Complete task
    pub fn complete(&mut self, task_id: &TaskId, result: TaskResult) -> FleetResult<()> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| FleetError::TaskFailed {
                task_id: task_id.as_str().to_string(),
                reason: "Task not found".into(),
            })?;

        task.status = result.status;

        // Remove from node tasks
        if let Some(node_id) = &task.assigned_node {
            if let Some(tasks) = self.node_tasks.get_mut(node_id) {
                tasks.retain(|t| t != task_id);
            }
        }

        self.completed.insert(task_id.clone(), result);

        tracing::info!(
            "Task {} completed with status {:?}",
            task_id.as_str(),
            task.status
        );
        Ok(())
    }

    /// Retry failed task
    pub fn retry(&mut self, task_id: &TaskId) -> FleetResult<bool> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| FleetError::TaskFailed {
                task_id: task_id.as_str().to_string(),
                reason: "Task not found".into(),
            })?;

        if task.retries >= task.max_retries {
            return Ok(false);
        }

        task.retries += 1;
        task.status = TaskStatus::Pending;
        task.assigned_node = None;
        task.started_at = None;

        // Re-queue
        self.ready_queue.push_back(task_id.clone());

        tracing::info!(
            "Task {} queued for retry ({}/{})",
            task_id.as_str(),
            task.retries,
            task.max_retries
        );
        Ok(true)
    }

    /// Get task
    pub fn get_task(&self, id: &TaskId) -> Option<&Task> {
        self.tasks.get(id)
    }

    /// Get task result
    pub fn get_result(&self, id: &TaskId) -> Option<&TaskResult> {
        self.completed.get(id)
    }

    /// Get pending task count
    pub fn pending_count(&self) -> usize {
        self.tasks
            .values()
            .filter(|t| t.status == TaskStatus::Pending)
            .count()
    }

    /// Get running task count
    pub fn running_count(&self) -> usize {
        self.tasks
            .values()
            .filter(|t| t.status == TaskStatus::Running)
            .count()
    }

    /// Get tasks for node
    pub fn tasks_for_node(&self, node_id: &NodeId) -> Vec<&Task> {
        self.node_tasks
            .get(node_id)
            .map(|ids| ids.iter().filter_map(|id| self.tasks.get(id)).collect())
            .unwrap_or_default()
    }

    /// Check for timed out tasks
    pub fn check_timeouts(&mut self) -> Vec<TaskId> {
        let mut timed_out = Vec::new();

        for (id, task) in &self.tasks {
            if task.status == TaskStatus::Running && task.is_timed_out() {
                timed_out.push(id.clone());
            }
        }

        for id in &timed_out {
            if let Some(task) = self.tasks.get_mut(id) {
                task.status = TaskStatus::TimedOut;
            }
        }

        timed_out
    }

    /// Schedule tasks to available nodes
    pub fn schedule(&mut self, pool: &mut NodePool) -> Vec<(TaskId, NodeId)> {
        let mut assignments = Vec::new();

        while let Some(task) = self.next_task() {
            let task_id = task.id.clone();

            // Find best node
            if let Some(node) = pool.least_loaded() {
                let node_id = node.info.id.clone();

                // Check capacity
                let current_tasks = self.node_tasks.get(&node_id).map(|t| t.len()).unwrap_or(0);

                if current_tasks < self.config.max_tasks_per_node {
                    if self.assign(&task_id, node_id.clone()).is_ok() {
                        if let Some(node) = pool.get_mut(&node_id) {
                            node.assign_task(task_id.as_str().to_string());
                        }
                        assignments.push((task_id, node_id));
                    }
                } else {
                    // No capacity, put back in queue
                    self.ready_queue.push_back(task_id);
                    break;
                }
            } else {
                // No available nodes
                self.ready_queue.push_back(task_id);
                break;
            }
        }

        assignments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_generate() {
        let id1 = TaskId::generate();
        let id2 = TaskId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new("test", vec![1, 2, 3]);
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, TaskPriority::Normal);
    }

    #[test]
    fn test_task_priority() {
        let task = Task::new("test", vec![]).with_priority(TaskPriority::Critical);
        assert_eq!(task.priority, TaskPriority::Critical);
    }

    #[test]
    fn test_scheduler_submit() {
        let mut scheduler = Scheduler::new(SchedulerConfig::default());
        let task = Task::new("test", vec![]);
        let id = scheduler.submit(task);

        assert!(scheduler.get_task(&id).is_some());
        assert_eq!(scheduler.pending_count(), 1);
    }

    #[test]
    fn test_task_status() {
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(!TaskStatus::Running.is_terminal());
        assert!(TaskStatus::Running.is_active());
    }
}
