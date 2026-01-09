//! Error types for vaya-book

use std::fmt;

/// Result type for booking operations
pub type BookResult<T> = Result<T, BookError>;

/// Errors that can occur during booking operations
#[derive(Debug, Clone)]
pub enum BookError {
    // === Validation Errors ===
    /// Invalid passenger data
    InvalidPassenger(String),
    /// Invalid contact information
    InvalidContact(String),
    /// Invalid payment data
    InvalidPayment(String),
    /// Missing required field
    MissingField(String),
    /// Passenger count mismatch
    PassengerCountMismatch { expected: u8, got: u8 },

    // === State Errors ===
    /// Booking not found
    BookingNotFound(String),
    /// Invalid state transition
    InvalidStateTransition { from: String, to: String },
    /// Booking already exists
    BookingExists(String),
    /// Booking expired
    BookingExpired,
    /// Offer expired
    OfferExpired,
    /// Offer no longer available
    OfferUnavailable,

    // === Payment Errors ===
    /// Payment failed
    PaymentFailed(String),
    /// Payment already processed
    PaymentAlreadyProcessed,
    /// Refund failed
    RefundFailed(String),
    /// Insufficient funds
    InsufficientFunds,
    /// Payment timeout
    PaymentTimeout,

    // === Cancellation Errors ===
    /// Booking not cancellable
    NotCancellable(String),
    /// Cancellation deadline passed
    CancellationDeadlinePassed,
    /// Partial cancellation not allowed
    PartialCancellationNotAllowed,

    // === Ticketing Errors ===
    /// Ticketing failed
    TicketingFailed(String),
    /// Already ticketed
    AlreadyTicketed,
    /// Ticket not found
    TicketNotFound(String),
    /// Void deadline passed
    VoidDeadlinePassed,

    // === Concurrency Errors ===
    /// Concurrent modification
    ConcurrentModification,
    /// Lock acquisition failed
    LockFailed,
    /// Operation timeout
    Timeout,

    // === System Errors ===
    /// Internal error
    Internal(String),
    /// Provider error
    ProviderError(String),
    /// Serialization error
    SerializationError(String),
}

impl fmt::Display for BookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Validation
            BookError::InvalidPassenger(msg) => write!(f, "Invalid passenger: {}", msg),
            BookError::InvalidContact(msg) => write!(f, "Invalid contact: {}", msg),
            BookError::InvalidPayment(msg) => write!(f, "Invalid payment: {}", msg),
            BookError::MissingField(field) => write!(f, "Missing required field: {}", field),
            BookError::PassengerCountMismatch { expected, got } => {
                write!(f, "Passenger count mismatch: expected {}, got {}", expected, got)
            }

            // State
            BookError::BookingNotFound(id) => write!(f, "Booking not found: {}", id),
            BookError::InvalidStateTransition { from, to } => {
                write!(f, "Invalid state transition from {} to {}", from, to)
            }
            BookError::BookingExists(id) => write!(f, "Booking already exists: {}", id),
            BookError::BookingExpired => write!(f, "Booking has expired"),
            BookError::OfferExpired => write!(f, "Offer has expired"),
            BookError::OfferUnavailable => write!(f, "Offer is no longer available"),

            // Payment
            BookError::PaymentFailed(msg) => write!(f, "Payment failed: {}", msg),
            BookError::PaymentAlreadyProcessed => write!(f, "Payment already processed"),
            BookError::RefundFailed(msg) => write!(f, "Refund failed: {}", msg),
            BookError::InsufficientFunds => write!(f, "Insufficient funds"),
            BookError::PaymentTimeout => write!(f, "Payment timeout"),

            // Cancellation
            BookError::NotCancellable(reason) => write!(f, "Booking not cancellable: {}", reason),
            BookError::CancellationDeadlinePassed => write!(f, "Cancellation deadline passed"),
            BookError::PartialCancellationNotAllowed => write!(f, "Partial cancellation not allowed"),

            // Ticketing
            BookError::TicketingFailed(msg) => write!(f, "Ticketing failed: {}", msg),
            BookError::AlreadyTicketed => write!(f, "Already ticketed"),
            BookError::TicketNotFound(id) => write!(f, "Ticket not found: {}", id),
            BookError::VoidDeadlinePassed => write!(f, "Void deadline passed"),

            // Concurrency
            BookError::ConcurrentModification => write!(f, "Concurrent modification detected"),
            BookError::LockFailed => write!(f, "Failed to acquire lock"),
            BookError::Timeout => write!(f, "Operation timeout"),

            // System
            BookError::Internal(msg) => write!(f, "Internal error: {}", msg),
            BookError::ProviderError(msg) => write!(f, "Provider error: {}", msg),
            BookError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for BookError {}

impl BookError {
    /// Check if error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            BookError::Timeout
                | BookError::LockFailed
                | BookError::PaymentTimeout
                | BookError::ProviderError(_)
        )
    }

    /// Check if error is a validation error
    pub fn is_validation(&self) -> bool {
        matches!(
            self,
            BookError::InvalidPassenger(_)
                | BookError::InvalidContact(_)
                | BookError::InvalidPayment(_)
                | BookError::MissingField(_)
                | BookError::PassengerCountMismatch { .. }
        )
    }

    /// Check if error is a state error
    pub fn is_state_error(&self) -> bool {
        matches!(
            self,
            BookError::InvalidStateTransition { .. }
                | BookError::BookingExpired
                | BookError::OfferExpired
        )
    }
}
