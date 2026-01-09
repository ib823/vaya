//! API Handlers - All 71 REST API endpoint handlers
//!
//! Organized by domain:
//! - auth: Authentication and session management (8 handlers)
//! - search: Flight search and suggestions (6 handlers)
//! - oracle: Price predictions (4 handlers)
//! - booking: Booking management (8 handlers)
//! - pool: Group buying pools (10 handlers)
//! - alert: Price alerts (6 handlers)
//! - user: User profile and settings (11 handlers)
//! - traveler: Traveler profiles (5 handlers)
//! - payment: Payment processing (6 handlers)
//! - trip: Trip management (6 handlers)
//! - notification: Notifications (4 handlers)
//! - support: Customer support tickets (4 handlers)
//! - admin: Admin operations (8 handlers)

pub mod admin;
pub mod alert;
pub mod auth;
pub mod booking;
pub mod notification;
pub mod oracle;
pub mod payment;
pub mod pool;
pub mod search;
pub mod support;
pub mod traveler;
pub mod trip;
pub mod user;

pub use admin::*;
pub use alert::*;
pub use auth::*;
pub use booking::*;
pub use notification::*;
pub use oracle::*;
pub use payment::*;
pub use pool::*;
pub use search::*;
pub use support::*;
pub use traveler::*;
pub use trip::*;
pub use user::*;

/// Total number of API handlers
pub const HANDLER_COUNT: usize = 71;
