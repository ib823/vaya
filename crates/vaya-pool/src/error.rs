//! Error types for vaya-pool

use std::fmt;

/// Result type for pool operations
pub type PoolResult<T> = Result<T, PoolError>;

/// Errors that can occur during pool operations
#[derive(Debug, Clone)]
pub enum PoolError {
    // === Pool State Errors ===
    /// Pool not found
    PoolNotFound(String),
    /// Pool already exists
    PoolExists(String),
    /// Invalid pool status transition
    InvalidStateTransition { from: String, to: String },
    /// Pool is not joinable (full, locked, or closed)
    PoolNotJoinable(String),
    /// Pool is locked and cannot be modified
    PoolLocked,
    /// Pool has expired
    PoolExpired,
    /// Pool is already complete
    PoolCompleted,

    // === Member Errors ===
    /// Member not found in pool
    MemberNotFound(String),
    /// User is already a member
    AlreadyMember,
    /// User is not a member
    NotAMember,
    /// Cannot leave pool (locked or own contribution)
    CannotLeave(String),
    /// Member limit reached
    MemberLimitReached,
    /// Minimum members not reached
    MinMembersNotReached { required: u32, current: u32 },

    // === Contribution Errors ===
    /// Invalid contribution amount
    InvalidContribution(String),
    /// Contribution not found
    ContributionNotFound(String),
    /// Insufficient contribution
    InsufficientContribution { required: i64, provided: i64 },
    /// Contribution deadline passed
    ContributionDeadlinePassed,
    /// Contribution already processed
    ContributionAlreadyProcessed,

    // === Pricing Errors ===
    /// Tier not available
    TierNotAvailable(String),
    /// Price changed since pool creation
    PriceChanged,
    /// Offer no longer valid
    OfferInvalid,

    // === Validation Errors ===
    /// Invalid pool configuration
    InvalidConfig(String),
    /// Missing required field
    MissingField(String),
    /// Invalid date range
    InvalidDateRange,

    // === Concurrency Errors ===
    /// Concurrent modification detected
    ConcurrentModification,
    /// Lock acquisition failed
    LockFailed,

    // === System Errors ===
    /// Internal error
    Internal(String),
    /// Serialization error
    SerializationError(String),
}

impl fmt::Display for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Pool State
            PoolError::PoolNotFound(id) => write!(f, "Pool not found: {}", id),
            PoolError::PoolExists(id) => write!(f, "Pool already exists: {}", id),
            PoolError::InvalidStateTransition { from, to } => {
                write!(f, "Invalid pool state transition from {} to {}", from, to)
            }
            PoolError::PoolNotJoinable(reason) => write!(f, "Pool not joinable: {}", reason),
            PoolError::PoolLocked => write!(f, "Pool is locked"),
            PoolError::PoolExpired => write!(f, "Pool has expired"),
            PoolError::PoolCompleted => write!(f, "Pool is already completed"),

            // Member
            PoolError::MemberNotFound(id) => write!(f, "Member not found: {}", id),
            PoolError::AlreadyMember => write!(f, "User is already a member of this pool"),
            PoolError::NotAMember => write!(f, "User is not a member of this pool"),
            PoolError::CannotLeave(reason) => write!(f, "Cannot leave pool: {}", reason),
            PoolError::MemberLimitReached => write!(f, "Pool member limit reached"),
            PoolError::MinMembersNotReached { required, current } => {
                write!(f, "Minimum {} members required, only {} joined", required, current)
            }

            // Contribution
            PoolError::InvalidContribution(msg) => write!(f, "Invalid contribution: {}", msg),
            PoolError::ContributionNotFound(id) => write!(f, "Contribution not found: {}", id),
            PoolError::InsufficientContribution { required, provided } => {
                write!(f, "Insufficient contribution: required {}, provided {}", required, provided)
            }
            PoolError::ContributionDeadlinePassed => write!(f, "Contribution deadline has passed"),
            PoolError::ContributionAlreadyProcessed => write!(f, "Contribution already processed"),

            // Pricing
            PoolError::TierNotAvailable(tier) => write!(f, "Pricing tier not available: {}", tier),
            PoolError::PriceChanged => write!(f, "Price has changed since pool creation"),
            PoolError::OfferInvalid => write!(f, "Offer is no longer valid"),

            // Validation
            PoolError::InvalidConfig(msg) => write!(f, "Invalid pool configuration: {}", msg),
            PoolError::MissingField(field) => write!(f, "Missing required field: {}", field),
            PoolError::InvalidDateRange => write!(f, "Invalid date range"),

            // Concurrency
            PoolError::ConcurrentModification => write!(f, "Concurrent modification detected"),
            PoolError::LockFailed => write!(f, "Failed to acquire lock"),

            // System
            PoolError::Internal(msg) => write!(f, "Internal error: {}", msg),
            PoolError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for PoolError {}

impl PoolError {
    /// Check if error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            PoolError::LockFailed
                | PoolError::ConcurrentModification
                | PoolError::Internal(_)
        )
    }

    /// Check if error is a validation error
    pub fn is_validation(&self) -> bool {
        matches!(
            self,
            PoolError::InvalidConfig(_)
                | PoolError::MissingField(_)
                | PoolError::InvalidDateRange
                | PoolError::InvalidContribution(_)
        )
    }

    /// Check if error is a state error
    pub fn is_state_error(&self) -> bool {
        matches!(
            self,
            PoolError::InvalidStateTransition { .. }
                | PoolError::PoolLocked
                | PoolError::PoolExpired
                | PoolError::PoolCompleted
                | PoolError::PoolNotJoinable(_)
        )
    }
}
